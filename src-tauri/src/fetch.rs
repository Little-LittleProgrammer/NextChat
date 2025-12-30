//
// 普通 HTTP 请求处理模块
//

use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

static REQUEST_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, serde::Serialize)]
pub struct FetchResponse {
    request_id: u32,
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorResponse {
    request_id: u32,
    status: u16,
    status_text: String,
    error: String,
}

#[tauri::command]
pub async fn http_fetch(
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> Result<FetchResponse, String> {
    let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);

    let mut _headers = HeaderMap::new();
    for (key, value) in &headers {
        match key.parse::<HeaderName>() {
            Ok(header_name) => match value.parse() {
                Ok(header_value) => {
                    _headers.insert(header_name, header_value);
                }
                Err(err) => {
                    return Err(format!("failed to parse header value '{}': {}", value, err));
                }
            },
            Err(err) => {
                return Err(format!("failed to parse header name '{}': {}", key, err));
            }
        }
    }

    // 解析 HTTP 方法
    let method = method
        .parse::<reqwest::Method>()
        .map_err(|err| format!("failed to parse method: {}", err))?;

    // 创建客户端
    let client = Client::builder()
        .default_headers(_headers)
        .redirect(reqwest::redirect::Policy::limited(3))
        .connect_timeout(Duration::new(10, 0))
        .timeout(Duration::new(30, 0))
        .build()
        .map_err(|err| format!("failed to create client: {}", err))?;

    // 构建请求
    let url_str = url.clone();
    let mut request = client.request(
        method.clone(),
        url.parse::<reqwest::Url>()
            .map_err(|err| format!("failed to parse url: {}", err))?,
    );

    // 打印请求信息
    log::info!("[HTTP Request] Request ID: {}", request_id);
    log::info!("[HTTP Request] Method: {}", method);
    log::info!("[HTTP Request] URL: {}", url_str);
    log::debug!("[HTTP Request] Headers: {:?}", headers);
    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);
        log::debug!("[HTTP Request] Body: {} ({} bytes)", body_str, body.len());
    }

    // 对于需要 body 的请求方法，添加请求体
    if method == reqwest::Method::POST
        || method == reqwest::Method::PUT
        || method == reqwest::Method::PATCH
        || method == reqwest::Method::DELETE
    {
        if !body.is_empty() {
            let body_bytes = bytes::Bytes::from(body);
            request = request.body(body_bytes);
        }
    }

    // 发送请求
    let response = request.send().await.map_err(|err| {
        let error_msg = err
            .source()
            .map(|e| e.to_string())
            .unwrap_or_else(|| err.to_string());
        format!("request failed: {}", error_msg)
    })?;

    // 获取响应状态和头部
    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    let mut response_headers = HashMap::new();
    for (name, value) in response.headers() {
        response_headers.insert(
            name.as_str().to_string(),
            std::str::from_utf8(value.as_bytes())
                .unwrap_or("<invalid utf8>")
                .to_string(),
        );
    }

    // 读取响应体
    let response_body = response
        .bytes()
        .await
        .map_err(|err| format!("failed to read response body: {}", err))?;

    // 打印响应信息
    log::info!("[HTTP Response] Request ID: {}", request_id);
    log::info!("[HTTP Response] Status: {} {}", status, status_text);
    log::debug!("[HTTP Response] Headers: {:?}", response_headers);
    let body_preview = if response_body.len() > 500 {
        format!(
            "{}... ({} bytes total)",
            String::from_utf8_lossy(&response_body[..500]),
            response_body.len()
        )
    } else {
        String::from_utf8_lossy(&response_body).to_string()
    };
    log::debug!("[HTTP Response] Body: {}", body_preview);

    Ok(FetchResponse {
        request_id,
        status,
        status_text,
        headers: response_headers,
        body: response_body.to_vec(),
    })
}

#[tauri::command]
pub async fn http_fetch_text(
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: String,
) -> Result<String, String> {
    // 将字符串 body 转换为字节
    let body_bytes = body.into_bytes();

    // 调用主要的 fetch 方法
    let response = http_fetch(method, url, headers, body_bytes).await?;

    // 将响应体转换为字符串
    let response_text = String::from_utf8(response.body)
        .map_err(|err| format!("failed to convert response to text: {}", err))?;

    Ok(response_text)
}

#[tauri::command]
pub async fn http_fetch_json(
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // 将 JSON 转换为字符串再转换为字节
    let body_string = serde_json::to_string(&body)
        .map_err(|err| format!("failed to serialize JSON body: {}", err))?;
    let body_bytes = body_string.into_bytes();

    // 确保设置了正确的 Content-Type
    let mut json_headers = headers;
    if !json_headers.contains_key("content-type") && !json_headers.contains_key("Content-Type") {
        json_headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    // 调用主要的 fetch 方法
    let response = http_fetch(method, url, json_headers, body_bytes).await?;

    // 将响应体解析为 JSON
    let response_json: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|err| format!("failed to parse response as JSON: {}", err))?;

    Ok(response_json)
}
