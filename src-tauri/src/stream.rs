//
//

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

static REQUEST_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamResponse {
    request_id: u32,
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
}

#[derive(Clone, serde::Serialize)]
pub struct EndPayload {
    request_id: u32,
    status: u16,
}

#[derive(Clone, serde::Serialize)]
pub struct ChunkPayload {
    request_id: u32,
    chunk: bytes::Bytes,
}

#[tauri::command]
pub async fn stream_fetch(
    window: tauri::Window,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> Result<StreamResponse, String> {
    let event_name = "stream-response";
    let request_id = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);

    let mut _headers = HeaderMap::new();
    for (key, value) in &headers {
        _headers.insert(key.parse::<HeaderName>().unwrap(), value.parse().unwrap());
    }

    // 打印请求信息
    log::info!("[Stream HTTP Request] Request ID: {}", request_id);
    log::info!("[Stream HTTP Request] Method: {}", method);
    log::info!("[Stream HTTP Request] URL: {}", url);
    log::debug!("[Stream HTTP Request] Headers: {:?}", headers);
    if !body.is_empty() {
        let body_str = String::from_utf8_lossy(&body);
        log::debug!(
            "[Stream HTTP Request] Body: {} ({} bytes)",
            body_str,
            body.len()
        );
    }

    let method = method
        .parse::<reqwest::Method>()
        .map_err(|err| format!("failed to parse method: {}", err))?;
    let client = Client::builder()
        .default_headers(_headers)
        .redirect(reqwest::redirect::Policy::limited(3))
        .connect_timeout(Duration::new(3, 0))
        .build()
        .map_err(|err| format!("failed to generate client: {}", err))?;

    let mut request = client.request(
        method.clone(),
        url.parse::<reqwest::Url>()
            .map_err(|err| format!("failed to parse url: {}", err))?,
    );

    if method == reqwest::Method::POST
        || method == reqwest::Method::PUT
        || method == reqwest::Method::PATCH
    {
        let body = bytes::Bytes::from(body);
        request = request.body(body);
    }

    let response_future = request.send();

    let res = response_future.await;
    let response = match res {
        Ok(res) => {
            // get response and emit to client
            let mut headers = HashMap::new();
            for (name, value) in res.headers() {
                headers.insert(
                    name.as_str().to_string(),
                    std::str::from_utf8(value.as_bytes()).unwrap().to_string(),
                );
            }
            let status = res.status().as_u16();
            let status_text = res
                .status()
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string();

            // 打印响应信息
            log::info!("[Stream HTTP Response] Request ID: {}", request_id);
            log::info!("[Stream HTTP Response] Status: {} {}", status, status_text);
            log::debug!("[Stream HTTP Response] Headers: {:?}", headers);
            log::info!("[Stream HTTP Response] Stream started (chunks will be emitted)");

            tauri::async_runtime::spawn(async move {
                let mut stream = res.bytes_stream();
                let mut total_bytes = 0usize;
                let mut chunk_count = 0u32;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            chunk_count += 1;
                            total_bytes += bytes.len();
                            log::debug!("[Stream HTTP Response] Request ID: {}, Chunk #{}: {} bytes (total: {} bytes)", 
                                request_id, chunk_count, bytes.len(), total_bytes);

                            if let Err(e) = window.emit(
                                event_name,
                                ChunkPayload {
                                    request_id,
                                    chunk: bytes,
                                },
                            ) {
                                log::error!("Failed to emit chunk payload: {:?}", e);
                            }
                        }
                        Err(err) => {
                            log::error!(
                                "[Stream HTTP Response] Request ID: {}, Error chunk: {:?}",
                                request_id, err
                            );
                        }
                    }
                }

                log::info!("[Stream HTTP Response] Request ID: {}, Stream ended (total chunks: {}, total bytes: {})", 
                    request_id, chunk_count, total_bytes);

                if let Err(e) = window.emit(
                    event_name,
                    EndPayload {
                        request_id,
                        status: 0,
                    },
                ) {
                    log::error!("Failed to emit end payload: {:?}", e);
                }
            });

            StreamResponse {
                request_id,
                status,
                status_text: "OK".to_string(),
                headers,
            }
        }
        Err(err) => {
            let error: String = err
                .source()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error occurred".to_string());
            log::error!(
                "[Stream HTTP Response] Request ID: {}, Error: {}",
                request_id, error
            );
            tauri::async_runtime::spawn(async move {
                if let Err(e) = window.emit(
                    event_name,
                    ChunkPayload {
                        request_id,
                        chunk: error.into(),
                    },
                ) {
                    log::error!("Failed to emit chunk payload: {:?}", e);
                }
                if let Err(e) = window.emit(
                    event_name,
                    EndPayload {
                        request_id,
                        status: 0,
                    },
                ) {
                    log::error!("Failed to emit end payload: {:?}", e);
                }
            });
            StreamResponse {
                request_id,
                status: 599,
                status_text: "Error".to_string(),
                headers: HashMap::new(),
            }
        }
    };
    Ok(response)
}
