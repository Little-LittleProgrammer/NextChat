use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::State;
use std::thread;
#[cfg(unix)]
use libc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub r#type: Option<String>, // "stdio" or "sse"
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub status: Option<String>,
    // SSE 模式配置
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub status: String,
    #[serde(rename = "errorMsg")]
    pub error_msg: Option<String>,
}

struct McpProcess {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>, // 使用 BufReader 但手动控制读取
    stderr: Arc<Mutex<String>>, // 存储 stderr 输出
    initialized: Arc<Mutex<bool>>,
}

pub struct McpState {
    processes: Arc<Mutex<HashMap<String, McpProcess>>>,
    config: Arc<Mutex<Option<McpConfig>>>,
}

impl McpState {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg(unix)]
fn poll_read_ready(fd: i32, timeout_ms: i32) -> Result<bool, String> {
    unsafe {
        use libc::{poll, pollfd, POLLIN};

        let mut fds = [pollfd {
            fd,
            events: POLLIN,
            revents: 0,
        }];

        let result = poll(fds.as_mut_ptr(), 1, timeout_ms);

        if result == -1 {
            return Err(format!("poll failed: {}", std::io::Error::last_os_error()));
        }

        Ok(result > 0 && (fds[0].revents & POLLIN) != 0)
    }
}

#[cfg(unix)]
fn set_nonblocking(fd: i32, nonblocking: bool) -> Result<(), String> {
    unsafe {
        use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

        let flags = fcntl(fd, F_GETFL);
        if flags == -1 {
            return Err(format!("fcntl F_GETFL failed: {}", std::io::Error::last_os_error()));
        }

        let new_flags = if nonblocking {
            flags | O_NONBLOCK
        } else {
            flags & !O_NONBLOCK
        };

        if fcntl(fd, F_SETFL, new_flags) == -1 {
            return Err(format!("fcntl F_SETFL failed: {}", std::io::Error::last_os_error()));
        }

        Ok(())
    }
}

/// 查找命令的完整路径
/// 在打包后的应用中，系统 PATH 可能不可用，需要显式查找命令路径
fn find_command_path(command: &str) -> Result<String, String> {
    log::info!("[find_command_path] Looking for command: {}", command);
    
    // 如果命令已经是绝对路径，直接返回
    if std::path::Path::new(command).is_absolute() {
        log::debug!("[find_command_path] Command is absolute path: {}", command);
        if std::path::Path::new(command).exists() {
            log::info!("[find_command_path] Found command at absolute path: {}", command);
            return Ok(command.to_string());
        } else {
            log::error!("[find_command_path] Command not found at absolute path: {}", command);
            return Err(format!("Command not found at absolute path: {}", command));
        }
    }
    
    log::debug!("[find_command_path] Command is relative, searching in PATH...");

    // 使用 which/where 命令查找
    #[cfg(unix)]
    {
        log::debug!("[find_command_path] Trying 'which' command...");
        let output = Command::new("which")
            .arg(command)
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    log::info!("[find_command_path] Found command '{}' via 'which' at: {}", command, path);
                    return Ok(path);
                } else {
                    log::debug!("[find_command_path] 'which' returned empty result");
                }
            }
            Ok(output) => {
                log::debug!("[find_command_path] 'which' failed with status: {:?}, stderr: {}", 
                    output.status, String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                log::debug!("[find_command_path] Failed to execute 'which': {}", e);
            }
        }
    }

    #[cfg(windows)]
    {
        log::debug!("[find_command_path] Trying 'where' command...");
        let output = Command::new("where")
            .arg(command)
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if !path.is_empty() {
                    log::info!("[find_command_path] Found command '{}' via 'where' at: {}", command, path);
                    return Ok(path);
                } else {
                    log::debug!("[find_command_path] 'where' returned empty result");
                }
            }
            Ok(output) => {
                log::debug!("[find_command_path] 'where' failed with status: {:?}, stderr: {}", 
                    output.status, String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                log::debug!("[find_command_path] Failed to execute 'where': {}", e);
            }
        }
    }

    // 尝试从环境变量 PATH 中查找
    log::debug!("[find_command_path] Searching in PATH environment variable...");
    let mut search_paths: Vec<String> = Vec::new();
    
    if let Ok(path_var) = std::env::var("PATH") {
        let paths: Vec<&str> = if cfg!(windows) {
            path_var.split(';').collect()
        } else {
            path_var.split(':').collect()
        };
        
        log::info!("[find_command_path] PATH environment variable: {} paths", paths.len());
        if log::log_enabled!(log::Level::Debug) {
            for (idx, path) in paths.iter().enumerate() {
                log::debug!("[find_command_path] PATH[{}]: {}", idx, path);
            }
        }
        search_paths.extend(paths.iter().map(|s| s.to_string()));
    } else {
        log::warn!("[find_command_path] PATH environment variable not available");
    }
    
    // 添加常见的安装路径作为回退
    #[cfg(unix)]
    {
        // macOS Homebrew 路径和其他常见路径
        if let Ok(home) = std::env::var("HOME") {
            let homebrew_paths = vec![
                format!("{}/.local/bin", home),
                format!("{}/.yarn/bin", home),
                format!("{}/.config/yarn/global/node_modules/.bin", home),
                "/opt/homebrew/bin".to_string(), // Apple Silicon Homebrew
                "/usr/local/bin".to_string(),     // Intel Homebrew / 标准安装
                "/usr/local/opt/node/bin".to_string(), // Homebrew node
                "/usr/bin".to_string(),
                "/bin".to_string(),
            ];
            log::debug!("[find_command_path] Adding {} common Unix paths", homebrew_paths.len());
            search_paths.extend(homebrew_paths);
            
            // 尝试查找 nvm 安装的 node（需要遍历版本目录）
            let nvm_base = format!("{}/.nvm/versions/node", home);
            log::debug!("[find_command_path] Checking for nvm installations at: {}", nvm_base);
            if let Ok(entries) = std::fs::read_dir(&nvm_base) {
                let mut nvm_count = 0;
                for entry in entries.flatten() {
                    let node_bin = entry.path().join("bin");
                    if node_bin.exists() {
                        let bin_path = node_bin.to_string_lossy().to_string();
                        log::debug!("[find_command_path] Found nvm node installation: {}", bin_path);
                        search_paths.push(bin_path);
                        nvm_count += 1;
                    }
                }
                if nvm_count > 0 {
                    log::info!("[find_command_path] Found {} nvm node installations", nvm_count);
                }
            } else {
                log::debug!("[find_command_path] No nvm installations found at: {}", nvm_base);
            }

            // 尝试查找 Volta 安装
            let volta_bin = format!("{}/.volta/bin", home);
            if std::path::Path::new(&volta_bin).exists() {
                log::debug!("[find_command_path] Found Volta installation at: {}", volta_bin);
                search_paths.push(volta_bin);
            }

            // 尝试查找 Bun 安装
            let bun_bin = format!("{}/.bun/bin", home);
            if std::path::Path::new(&bun_bin).exists() {
                log::debug!("[find_command_path] Found Bun installation at: {}", bun_bin);
                search_paths.push(bun_bin);
            }
            
            // 尝试查找 fnm (Fast Node Manager) 安装
            let fnm_base = format!("{}/.fnm_multishells", home);
            if std::path::Path::new(&fnm_base).exists() {
                log::debug!("[find_command_path] Checking for fnm installations");
                // fnm 通常通过 shell 初始化脚本设置，这里只记录
            }
        } else {
            log::warn!("[find_command_path] HOME environment variable not available");
        }
        
        // 尝试通过 shell 查找（使用 login shell 来加载用户环境）
        // macOS 打包应用可能无法访问用户的 shell 环境，尝试多种方式
        let shell_commands = vec![
            ("/bin/bash", true),  // login shell，会加载 ~/.bash_profile
            ("/bin/zsh", true),  // login zsh，会加载 ~/.zshrc
            ("/bin/sh", false),       // 基本 shell
        ];
        
        for (shell, use_login) in shell_commands {
            log::debug!("[find_command_path] Trying shell lookup with '{} {} \"command -v {}\"'...", 
                shell, if use_login { "-l -c" } else { "-c" }, command);
            
            let mut cmd = Command::new(shell);
            if use_login {
                cmd.arg("-l").arg("-c").arg(format!("command -v {}", command));
            } else {
                cmd.arg("-c").arg(format!("command -v {}", command));
            }
            
            match cmd.output() {
                Ok(output) => {
                    let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    
                    log::debug!("[find_command_path] Shell '{}' status: {:?}, stdout: '{}', stderr: '{}'", 
                        shell, output.status, stdout_str, stderr_str);
                    
                    if output.status.success() && !stdout_str.is_empty() {
                        if std::path::Path::new(&stdout_str).exists() {
                            log::info!("[find_command_path] Found command '{}' via shell '{}' at: {}", command, shell, stdout_str);
                            return Ok(stdout_str);
                        } else {
                            log::warn!("[find_command_path] Shell '{}' returned path '{}' but file does not exist", shell, stdout_str);
                        }
                    }
                }
                Err(e) => {
                    log::debug!("[find_command_path] Failed to execute shell '{}': {}", shell, e);
                }
            }
        }
        
        // 也尝试使用 which 通过 shell
        log::debug!("[find_command_path] Trying shell 'which' command lookup...");
        match Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("which {}", command))
            .output()
        {
            Ok(output) => {
                let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                log::debug!("[find_command_path] Shell 'which' status: {:?}, stdout: '{}'", 
                    output.status, stdout_str);
                
                if output.status.success() && !stdout_str.is_empty() && std::path::Path::new(&stdout_str).exists() {
                    log::info!("[find_command_path] Found command '{}' via shell 'which' at: {}", command, stdout_str);
                    return Ok(stdout_str);
                }
            }
            Err(e) => {
                log::debug!("[find_command_path] Failed to execute shell 'which': {}", e);
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows 常见路径
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            let common_paths = vec![
                format!("{}\\nodejs", program_files),
                format!("{}\\nodejs\\node_modules\\npm\\bin", program_files),
            ];
            search_paths.extend(common_paths);
        }
        
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            let common_paths = vec![
                format!("{}\\nodejs", program_files_x86),
            ];
            search_paths.extend(common_paths);
        }
        
        // 尝试通过 PowerShell 查找
        log::debug!("[find_command_path] Trying PowerShell lookup...");
        if let Ok(output) = Command::new("powershell")
            .arg("-Command")
            .arg(format!("Get-Command {} -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source", command))
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() && std::path::Path::new(&path).exists() {
                    log::info!("[find_command_path] Found command '{}' via PowerShell at: {}", command, path);
                    return Ok(path);
                }
            }
        }
    }
    
    // 在收集的路径中搜索
    log::info!("[find_command_path] Searching in {} total paths", search_paths.len());
    if log::log_enabled!(log::Level::Debug) {
        for (idx, path) in search_paths.iter().enumerate() {
            log::debug!("[find_command_path] Search path[{}]: {}", idx, path);
        }
    }
    
    let exe_ext = if cfg!(windows) { ".exe" } else { "" };
    let command_with_ext = format!("{}{}", command, exe_ext);
    
    for (idx, path_str) in search_paths.iter().enumerate() {
        // 检查路径目录是否存在
        let path_dir = std::path::Path::new(path_str);
        if !path_dir.exists() {
            log::debug!("[find_command_path] Search path[{}] '{}' does not exist, skipping", idx, path_str);
            continue;
        }
        
        let full_path = path_dir.join(&command_with_ext);
        log::debug!("[find_command_path] Checking path[{}]: {}", idx, full_path.display());
        
        if full_path.exists() && full_path.is_file() {
            log::info!("[find_command_path] Found command '{}' in search path[{}] at: {}", 
                command, idx, full_path.display());
            return Ok(full_path.to_string_lossy().to_string());
        }
        
        // 也尝试不带扩展名的版本（Unix）
        #[cfg(unix)]
        {
            let full_path_no_ext = path_dir.join(command);
            if full_path_no_ext.exists() && full_path_no_ext.is_file() {
                log::info!("[find_command_path] Found command '{}' in search path[{}] at: {}", 
                    command, idx, full_path_no_ext.display());
                return Ok(full_path_no_ext.to_string_lossy().to_string());
            }
        }
    }
    log::warn!("[find_command_path] Command '{}' not found in any of {} search paths", command, search_paths.len());

    // 如果都找不到，返回详细错误信息
    log::error!("[find_command_path] Failed to find command '{}' after all search methods", command);
    
    let suggestion = match command {
        "npx" | "node" => "Please install Node.js from https://nodejs.org/ and ensure it's added to your PATH.",
        "python" | "python3" => "Please install Python from https://www.python.org/ and ensure it's added to your PATH.",
        "uvx" => "Please install uv from https://github.com/astral-sh/uv and ensure it's added to your PATH.",
        _ => "Please ensure the command is installed and added to your PATH environment variable.",
    };
    
    Err(format!(
        "Failed to find command '{}': Command not found in PATH.\n\
        {}\n\
        Note: In packaged applications, the system PATH may not be accessible. \
        You may need to use the full path to the command instead.",
        command, suggestion
    ))
}

/// 从 BufReader<ChildStdout> 读取完整的 JSON 消息
/// 使用 fill_buf() 和 consume() 来避免 BufReader 内部缓冲区的阻塞
fn read_json_message(
    reader: &mut BufReader<ChildStdout>,
    max_size: usize,
    timeout_ms: u64,
) -> Result<String, String> {
    let mut buffer = String::with_capacity(1024);
    let start_time = Instant::now();
    let timeout_duration = if timeout_ms > 0 {
        Some(Duration::from_millis(timeout_ms))
    } else {
        None
    };

    loop {
        // 检查总超时
        if let Some(duration) = timeout_duration {
            if start_time.elapsed() > duration {
                log::debug!("Timeout! Buffer so far (first 500 chars): {:?}", &buffer[..std::cmp::min(500, buffer.len())]);
                return Err(format!("Timeout after {}ms", timeout_ms));
            }
        }

        // 使用 poll 检查是否有数据可用
        // 注意：poll 检查的是底层文件描述符，但 BufReader 可能有内部缓冲的数据
        // 由于 stdout 已设置为非阻塞模式，fill_buf 不会阻塞，但 poll 仍然有用
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = reader.get_ref().as_raw_fd();
            match poll_read_ready(fd, 100) {
                Ok(true) => {
                    log::debug!("poll: data available on fd {}", fd);
                }
                Ok(false) => {
                    log::debug!("poll: no data after 100ms, continuing");
                    continue;
                }
                Err(e) => {
                    log::debug!("poll failed: {}, falling back to sleep", e);
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        }
        #[cfg(not(unix))]
        {
            log::debug!("non-unix: sleeping 100ms before fill_buf");
            std::thread::sleep(Duration::from_millis(100));
        }

        // 现在检查 BufReader 内部缓冲区是否有数据
        // 由于 stdout 已设置为非阻塞模式，fill_buf 不会阻塞
        log::debug!("Calling fill_buf()...");
        let available = match reader.fill_buf() {
            Ok(buf) => buf,
            Err(e) => {
                // fill_buf 失败，可能是底层错误
                let err_msg = format!("Failed to fill buffer: {}", e);
                log::debug!("{}", err_msg);
                // 检查是否是 EOF（在非阻塞模式下，这通常意味着连接关闭）
                if buffer.is_empty() {
                    return Err("No data received from server".to_string());
                } else {
                    return Err("Incomplete JSON response".to_string());
                }
            }
        };
        let n = available.len();
        log::debug!("fill_buf returned {} bytes", n);

        // 如果 BufReader 内部缓冲区为空，继续循环等待
        // 在非阻塞模式下，这通常意味着底层没有数据，需要继续等待
        if n == 0 {
            continue;
        }

        // 处理可用数据
        let data_str = String::from_utf8_lossy(&available[..n]);

        // 第一个数据块特殊处理
        if buffer.is_empty() {
            log::debug!("First read: {} bytes, data: {:?}", n, data_str);
        }

        // 每 5 秒打印进度
        if !buffer.is_empty() && start_time.elapsed().as_secs() % 5 == 0 {
            log::debug!("Progress: buffer len={}", buffer.len());
        }

        // 检查缓冲区大小
        if buffer.len() + n > max_size {
            return Err(format!("Response too large (max {} bytes)", max_size));
        }

        buffer.push_str(&data_str);

        // 消耗已读取的数据
        reader.consume(n);

        // 解析已收到的数据，检查是否是完整 JSON
        if let Ok(_) = serde_json::from_str::<serde_json::Value>(&buffer) {
            // 验证括号匹配
            let json_str = buffer.trim();
            let mut brace_count = 0;
            let mut bracket_count = 0;
            let mut in_string = false;
            let mut escape = false;

            for ch in json_str.chars() {
                if !in_string {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 && bracket_count == 0 {
                                log::debug!("Found complete JSON at buffer len {}", buffer.len());
                                return Ok(buffer);
                            }
                        }
                        '[' => bracket_count += 1,
                        ']' => {
                            bracket_count -= 1;
                            if brace_count == 0 && bracket_count == 0 {
                                log::debug!("Found complete JSON array at buffer len {}", buffer.len());
                                return Ok(buffer);
                            }
                        }
                        '"' => in_string = true,
                        _ => {}
                    }
                } else {
                    if escape {
                        escape = false;
                    } else if ch == '\\' {
                        escape = true;
                    } else if ch == '"' {
                        in_string = false;
                    }
                }
            }
        }
    }
}

/// 读取 JSON-RPC 响应，跳过 notification 消息
/// MCP 服务器可能在响应前发送 notifications（如 notifications/initialized）
fn read_jsonrpc_response(
    reader: &mut BufReader<ChildStdout>,
    max_size: usize,
    timeout_ms: u64,
) -> Result<String, String> {
    log::debug!("[read_jsonrpc_response] Called with max_size: {}, timeout: {}ms", max_size, timeout_ms);
    let mut notification_count = 0;

    loop {
        log::debug!("[read_jsonrpc_response] Reading JSON message (attempt #{})", notification_count + 1);
        let response = read_json_message(reader, max_size, timeout_ms)
            .map_err(|e| {
                log::error!("[read_jsonrpc_response] Failed to read JSON message: {}", e);
                e
            })?;
        log::debug!("[read_jsonrpc_response] Received message: {} bytes", response.len());

        // 尝试解析为 JSON-RPC 消息
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response) {
            // 检查是否是 notification（没有 id 字段但有 method 字段）
            if value.get("id").is_none() && value.get("method").is_some() {
                // 这是 notification，跳过并继续读取
                notification_count += 1;
                let method = value.get("method").and_then(|m| m.as_str()).unwrap_or("unknown");
                log::info!("[read_jsonrpc_response] Skipping notification #{}: {}", notification_count, method);
                continue;
            }
            
            // 检查是否是响应（有 id 或 result/error 字段）
            if value.get("id").is_some() || value.get("result").is_some() || value.get("error").is_some() {
                log::info!("[read_jsonrpc_response] Found valid JSON-RPC response (skipped {} notifications)", notification_count);
                return Ok(response);
            }
        } else {
            log::warn!("[read_jsonrpc_response] Received message is not valid JSON, returning as-is");
        }

        // 返回响应
        log::info!("[read_jsonrpc_response] Returning response (skipped {} notifications)", notification_count);
        return Ok(response);
    }
}

/// 从捕获的 stderr 缓冲区中提取 JSON-RPC 响应
fn extract_jsonrpc_from_stderr(stderr_buffer: &str) -> Option<String> {
    log::debug!("[extract_jsonrpc_from_stderr] Checking stderr buffer ({} bytes)", stderr_buffer.len());
    
    if stderr_buffer.is_empty() {
        log::debug!("[extract_jsonrpc_from_stderr] stderr buffer is empty");
        return None;
    }

    // 尝试解析整个 buffer
    log::debug!("[extract_jsonrpc_from_stderr] Attempting to parse entire buffer as JSON");
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(stderr_buffer) {
        // 检查是否是有效的 JSON-RPC 响应
        if value.get("id").is_some() || value.get("result").is_some() || value.get("error").is_some() {
            log::info!("[extract_jsonrpc_from_stderr] Found valid JSON-RPC response in entire stderr buffer");
            return Some(stderr_buffer.to_string());
        } else {
            log::debug!("[extract_jsonrpc_from_stderr] Buffer is valid JSON but not a JSON-RPC response");
        }
    } else {
        log::debug!("[extract_jsonrpc_from_stderr] Buffer is not valid JSON, trying line-by-line");
    }

    // 尝试从多行中提取完整的 JSON
    let lines: Vec<&str> = stderr_buffer.lines().collect();
    log::debug!("[extract_jsonrpc_from_stderr] Checking {} lines for JSON-RPC", lines.len());
    
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        log::debug!("[extract_jsonrpc_from_stderr] Checking line {}: {} chars", idx, trimmed.len());
        
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if value.get("id").is_some() || value.get("result").is_some() || value.get("error").is_some() {
                log::info!("[extract_jsonrpc_from_stderr] Found valid JSON-RPC response in line {}", idx);
                return Some(trimmed.to_string());
            }
        }
    }

    log::debug!("[extract_jsonrpc_from_stderr] No valid JSON-RPC response found in stderr");
    None
}

/// 发送 MCP 初始化请求
fn initialize_mcp_server(
    reader: &mut BufReader<ChildStdout>,
    writer: &mut ChildStdin,
    timeout_ms: u64,
) -> Result<(), String> {
    log::info!("[initialize_mcp_server] Starting MCP server initialization (timeout: {}ms)", timeout_ms);
    
    // 构建初始化请求
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "nextchat-tauri",
                "version": "1.0.0"
            }
        }
    });

    log::debug!("[initialize_mcp_server] Serializing initialization request");
    let request_str = serde_json::to_string(&init_request)
        .map_err(|e| {
            log::error!("[initialize_mcp_server] Failed to serialize init request: {}", e);
            format!("Failed to serialize init request: {}", e)
        })?;

    log::info!("[initialize_mcp_server] Sending initialize request...");
    log::debug!("[initialize_mcp_server] Initialize request: {}", request_str);
    
    // 发送请求
    writeln!(writer, "{}", request_str)
        .map_err(|e| {
            log::error!("[initialize_mcp_server] Failed to write init request: {}", e);
            format!("Failed to write init request: {}", e)
        })?;
    writer
        .flush()
        .map_err(|e| {
            log::error!("[initialize_mcp_server] Failed to flush init request: {}", e);
            format!("Failed to flush init request: {}", e)
        })?;
    log::info!("[initialize_mcp_server] Initialize request sent and flushed, waiting for response...");

    // 读取响应（可能需要跳过 notification 消息）
    const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
    log::debug!("[initialize_mcp_server] Reading response (max size: {} bytes)", MAX_MESSAGE_SIZE);
    let response_str = read_jsonrpc_response(reader, MAX_MESSAGE_SIZE, timeout_ms)
        .map_err(|e| {
            log::error!("[initialize_mcp_server] Failed to read init response: {}", e);
            format!("Failed to read init response: {}", e)
        })?;
    log::info!("[initialize_mcp_server] Initialize response received ({} bytes): {}", 
        response_str.len(), response_str);

    log::debug!("[initialize_mcp_server] Parsing response JSON");
    let response: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| {
            log::error!("[initialize_mcp_server] Failed to parse init response: {}", e);
            log::error!("[initialize_mcp_server] Response content: {}", response_str);
            format!("Failed to parse init response: {}", e)
        })?;

    // 检查响应
    if let Some(error) = response.get("error") {
        log::error!("[initialize_mcp_server] Server returned error: {}", error);
        return Err(format!("MCP server initialization failed: {}", error));
    }

    log::info!("[initialize_mcp_server] MCP server initialization completed successfully");
    Ok(())
}

#[tauri::command]
pub async fn mcp_read_config(app_handle: tauri::AppHandle) -> Result<McpConfig, String> {
    log::info!("[mcp_read_config] Reading MCP configuration");
    
    let app_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| {
            log::error!("[mcp_read_config] Failed to get app data directory");
            "Failed to get app data dir".to_string()
        })?;

    let config_path = app_dir.join("mcp_config.json");
    log::debug!("[mcp_read_config] Config path: {:?}", config_path);

    if !config_path.exists() {
        log::info!("[mcp_read_config] Config file does not exist, returning default empty config");
        // 返回默认配置
        return Ok(McpConfig {
            mcp_servers: HashMap::new(),
        });
    }

    log::debug!("[mcp_read_config] Reading config file content");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| {
            log::error!("[mcp_read_config] Failed to read config file: {}", e);
            format!("Failed to read config file: {}", e)
        })?;
    
    log::debug!("[mcp_read_config] Config file content length: {} bytes", content.len());

    let config: McpConfig = serde_json::from_str(&content)
        .map_err(|e| {
            log::error!("[mcp_read_config] Failed to parse config file: {}", e);
            log::error!("[mcp_read_config] Config content: {}", content);
            format!("Failed to parse config file: {}", e)
        })?;

    log::info!("[mcp_read_config] Successfully loaded config with {} servers", config.mcp_servers.len());
    Ok(config)
}

#[tauri::command]
pub async fn mcp_write_config(
    app_handle: tauri::AppHandle,
    config: McpConfig,
    state: State<'_, McpState>,
) -> Result<(), String> {
    log::info!("[mcp_write_config] Writing MCP configuration with {} servers", config.mcp_servers.len());
    
    let app_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| {
            log::error!("[mcp_write_config] Failed to get app data directory");
            "Failed to get app data dir".to_string()
        })?;

    // 确保目录存在
    log::debug!("[mcp_write_config] Creating app data directory if not exists: {:?}", app_dir);
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| {
            log::error!("[mcp_write_config] Failed to create app data dir: {}", e);
            format!("Failed to create app data dir: {}", e)
        })?;

    let config_path = app_dir.join("mcp_config.json");
    log::debug!("[mcp_write_config] Config path: {:?}", config_path);

    log::debug!("[mcp_write_config] Serializing config to JSON");
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| {
            log::error!("[mcp_write_config] Failed to serialize config: {}", e);
            format!("Failed to serialize config: {}", e)
        })?;
    
    log::debug!("[mcp_write_config] Serialized config length: {} bytes", content.len());

    log::debug!("[mcp_write_config] Writing config to file");
    std::fs::write(&config_path, content)
        .map_err(|e| {
            log::error!("[mcp_write_config] Failed to write config file: {}", e);
            format!("Failed to write config file: {}", e)
        })?;
    
    log::info!("[mcp_write_config] Config file written successfully");

    // 更新状态前，比较新旧配置，停止已删除的服务器进程
    let mut processes = state.processes.lock().unwrap();
    let mut state_config = state.config.lock().unwrap();

    // 获取旧配置
    let old_config = state_config.as_ref();

    // 如果存在旧配置，停止已删除的服务器
    if let Some(old_cfg) = old_config {
        log::debug!("[mcp_write_config] Comparing with old config ({} servers)", old_cfg.mcp_servers.len());
        for old_client_id in old_cfg.mcp_servers.keys() {
            if !config.mcp_servers.contains_key(old_client_id) {
                log::info!("[mcp_write_config] Server '{}' removed from config, stopping process", old_client_id);
                // 服务器已从配置中删除，停止进程
                if let Some(mut process) = processes.remove(old_client_id) {
                    let pid = process.child.id();
                    if let Err(e) = process.child.kill() {
                        log::warn!("[mcp_write_config] Failed to kill process (PID {}) for removed server '{}': {}", 
                            pid, old_client_id, e);
                    } else {
                        log::info!("[mcp_write_config] Killed process (PID {}) for removed server '{}'", 
                            pid, old_client_id);
                    }
                }
            }
        }
    } else {
        log::debug!("[mcp_write_config] No old config to compare");
    }

    // 更新状态
    log::debug!("[mcp_write_config] Updating state config");
    *state_config = Some(config);
    
    log::info!("[mcp_write_config] Configuration updated successfully");
    Ok(())
}

// 从旧版 NextChat 路径读取用户配置
#[tauri::command]
pub async fn mcp_read_user_config() -> Result<McpConfig, String> {
    // 旧版 NextChat 路径: ~/Library/Application Support/com.yida.chatgpt.next.web/mcp_config.json
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Failed to get home directory".to_string())?;

    let old_config_path = home_dir
        .join("Library")
        .join("Application Support")
        .join("com.yida.chatgpt.next.web")
        .join("mcp_config.json");

    if !old_config_path.exists() {
        // 返回默认配置（没有服务器）
        return Ok(McpConfig {
            mcp_servers: HashMap::new(),
        });
    }

    let content = std::fs::read_to_string(&old_config_path)
        .map_err(|e| format!("Failed to read old config file: {}", e))?;

    let config: McpConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse old config file: {}", e))?;

    Ok(config)
}

// 导入用户配置到当前配置
#[tauri::command]
pub async fn mcp_import_user_config(
    app_handle: tauri::AppHandle,
    state: State<'_, McpState>,
) -> Result<McpConfig, String> {
    let user_config = mcp_read_user_config().await?;
    let current_config = mcp_read_config(app_handle.clone()).await?;

    // 合并配置：只添加不在当前配置中的服务器
    let mut merged_config = current_config;
    let mut imported_count = 0;

    for (client_id, server_config) in user_config.mcp_servers {
        if !merged_config.mcp_servers.contains_key(&client_id) {
            merged_config.mcp_servers.insert(client_id, server_config);
            imported_count += 1;
        }
    }

    if imported_count > 0 {
        log::info!("Imported {} servers from old config", imported_count);
        // 保存合并后的配置
        mcp_write_config(app_handle, merged_config.clone(), state).await?;
    }

    Ok(merged_config)
}

#[tauri::command]
pub async fn mcp_start_server(
    client_id: String,
    config: ServerConfig,
    state: State<'_, McpState>,
) -> Result<(), String> {
    // 检查服务器类型
    let server_type = config.r#type.as_deref().unwrap_or("stdio");
    
    // SSE 类型的服务器不需要启动进程，直接返回成功
    if server_type == "sse" {
        log::info!("[mcp_start_server] SSE server {} does not require process startup", client_id);
        return Ok(());
    }
    
    log::info!("[mcp_start_server] Starting server: client_id={}, command={:?}, args={:?}", 
        client_id, config.command, config.args);
    
    // SSE 类型不需要 command，但 stdio 类型需要
    let command = config.command.ok_or_else(|| {
        format!("Stdio transport requires 'command' in config for server {}", client_id)
    })?;
    
    let mut processes = state.processes.lock().unwrap();

    // 如果进程已存在，先停止
    if let Some(mut process) = processes.remove(&client_id) {
        log::info!("[mcp_start_server] Stopping existing process for server {}", client_id);
        let pid = process.child.id();
        if let Err(e) = process.child.kill() {
            log::warn!("[mcp_start_server] Failed to kill existing process (PID {}): {}", pid, e);
        } else {
            log::info!("[mcp_start_server] Killed existing process (PID {})", pid);
        }
        if let Err(e) = process.child.wait() {
            log::warn!("[mcp_start_server] Error waiting for process (PID {}): {}", pid, e);
        }
    }

    // 验证环境变量名和值
    log::debug!("[mcp_start_server] Validating {} environment variables", config.env.len());
    for (key, value) in &config.env {
        if key.is_empty() {
            log::error!("[mcp_start_server] Environment variable name is empty");
            return Err("Environment variable name cannot be empty".to_string());
        }
        if key.contains('\0') || value.contains('\0') {
            log::error!("[mcp_start_server] Environment variable '{}' contains null character", key);
            return Err("Environment variable contains null character".to_string());
        }
    }
    log::debug!("[mcp_start_server] Environment variables validated successfully");

    // 查找命令的完整路径（打包后的应用可能无法访问系统 PATH）
    log::info!("[mcp_start_server] Looking up command path for: {}", command);
    let command_path = find_command_path(&command)
        .map_err(|e| {
            log::error!("[mcp_start_server] Failed to find command '{}': {}", command, e);
            e
        })?;
    log::info!("[mcp_start_server] Using command path: {}", command_path);
    
    // 启动新进程
    log::debug!("[mcp_start_server] Preparing command: {} with args: {:?}", command_path, config.args);
    let mut cmd = Command::new(&command_path);
    cmd.args(&config.args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    // 使用 pipe 处理 stderr，避免进程阻塞
    cmd.stderr(Stdio::piped());

    // 设置环境变量
    log::debug!("[mcp_start_server] Setting {} environment variables", config.env.len());
    for (key, value) in &config.env {
        cmd.env(key, value);
        log::debug!("[mcp_start_server] Set env: {}={}", key, 
            if value.len() > 50 { format!("{}...", &value[..50]) } else { value.clone() });
    }
    
    // 确保 PATH 环境变量可用（打包后的应用可能无法访问系统 PATH）
    // 如果用户没有设置 PATH，尝试从系统获取
    if !config.env.contains_key("PATH") {
        log::debug!("[mcp_start_server] PATH not in config, trying to get from system");
        if let Ok(path_var) = std::env::var("PATH") {
            cmd.env("PATH", &path_var);
            log::info!("[mcp_start_server] Using system PATH ({} chars)", path_var.len());
        } else {
            log::warn!("[mcp_start_server] System PATH not available, using fallback");
            // 如果无法获取系统 PATH，尝试使用常见的路径
            #[cfg(unix)]
            {
                let common_paths = vec![
                    "/usr/local/bin",
                    "/usr/bin",
                    "/bin",
                    "/opt/homebrew/bin", // macOS Homebrew
                    "/usr/local/sbin",
                    "/usr/sbin",
                    "/sbin",
                ];
                let fallback_path = common_paths.join(":");
                cmd.env("PATH", &fallback_path);
                log::warn!("System PATH not available, using fallback PATH: {}", fallback_path);
            }
            #[cfg(windows)]
            {
                // Windows 常见路径
                if let Ok(program_files) = std::env::var("ProgramFiles") {
                    let common_paths = vec![
                        format!("{}\\nodejs", program_files),
                        format!("{}\\Python", program_files),
                        "C:\\Windows\\System32",
                        "C:\\Windows",
                        "C:\\Windows\\System32\\WindowsPowerShell\\v1.0",
                    ];
                    let fallback_path = common_paths.join(";");
                    cmd.env("PATH", &fallback_path);
                    log::warn!("System PATH not available, using fallback PATH: {}", fallback_path);
                }
            }
        }
    }

    log::info!("[mcp_start_server] Spawning process: {} {:?}", command_path, config.args);
    let mut child = cmd
        .spawn()
        .map_err(|e| {
            log::error!("[mcp_start_server] Failed to spawn process: {}", e);
            format!("Failed to start MCP server '{}': {}", command, e)
        })?;

    let pid = child.id();
    log::info!("[mcp_start_server] Process spawned successfully: client_id={}, PID={}", client_id, pid);

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to get stdin".to_string())?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to get stdout".to_string())?;

    // 设置 stdout 为非阻塞模式，避免 fill_buf 阻塞
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = stdout.as_raw_fd();
        if let Err(e) = set_nonblocking(fd, true) {
            log::warn!("Failed to set stdout non-blocking: {}, continuing anyway", e);
        } else {
            log::debug!("Set stdout (fd {}) to non-blocking mode", fd);
        }
    }

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to get stderr".to_string())?;

    let stderr_capture = Arc::new(Mutex::new(String::new()));

    // 启动后台线程处理 stderr 输出
    {
        let client_id_clone = client_id.clone();
        let stderr_capture_clone = stderr_capture.clone();
        thread::spawn(move || {
            let mut stderr_buf = BufReader::new(stderr);
            let mut line = String::new();
            let mut line_count = 0;
            while let Ok(n) = stderr_buf.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                line_count += 1;
                // 捕获每一行
                let trimmed = line.trim_end().to_string();
                {
                    let mut captured = stderr_capture_clone.lock().unwrap();
                    if !captured.is_empty() {
                        captured.push('\n');
                    }
                    captured.push_str(&trimmed);
                }
                // 打印到控制台
                log::info!("[MCP {}] {}", client_id_clone, trimmed);
                line.clear();
            }
            log::info!("[MCP {}] stderr stream ended ({} lines)", client_id_clone, line_count);
        });
    }

    // 等待一小段时间，检查进程是否立即退出
    std::thread::sleep(Duration::from_millis(100));
    
    // 检查进程是否还在运行
    match child.try_wait() {
        Ok(Some(status)) => {
            // 进程已经退出
            let exit_code = status.code().unwrap_or(-1);
            log::error!(
                "[mcp_start_server] Process exited immediately after start: client_id={}, PID={}, exit_code={}",
                client_id, pid, exit_code
            );
            
            // 读取 stderr 输出以获取错误信息
            let stderr_content = stderr_capture.lock().unwrap().clone();
            let error_msg = if !stderr_content.is_empty() {
                format!(
                    "Process exited immediately with code {}. stderr: {}",
                    exit_code,
                    if stderr_content.len() > 500 {
                        format!("{}...", &stderr_content[..500])
                    } else {
                        stderr_content
                    }
                )
            } else {
                format!("Process exited immediately with code {}", exit_code)
            };
            
            return Err(format!(
                "Failed to start MCP server '{}': {}",
                client_id, error_msg
            ));
        }
        Ok(None) => {
            // 进程仍在运行，正常情况
            log::info!("[mcp_start_server] Process is running: client_id={}, PID={}", client_id, pid);
        }
        Err(e) => {
            log::warn!("[mcp_start_server] Failed to check process status: {}, assuming it's running", e);
        }
    }

    processes.insert(
        client_id.clone(),
        McpProcess {
            child,
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            stderr: stderr_capture,
            initialized: Arc::new(Mutex::new(false)),
        },
    );

    log::info!(
        "[mcp_start_server] Server started successfully: client_id={}, command={}, args={:?}, PID={}",
        client_id,
        command,
        config.args,
        pid
    );
    Ok(())
}

#[tauri::command]
pub async fn mcp_stop_server(client_id: String, state: State<'_, McpState>) -> Result<(), String> {
    log::info!("[mcp_stop_server] Stopping server: {}", client_id);
    
    let mut processes = state.processes.lock().unwrap();

    if let Some(mut process) = processes.remove(&client_id) {
        let pid = process.child.id();
        log::info!("[mcp_stop_server] Found process (PID {}) for server {}", pid, client_id);
        
        process.child.kill().map_err(|e| {
            log::error!(
                "[mcp_stop_server] Failed to kill process (PID {}) for server {}: {}",
                pid, client_id, e
            );
            format!("Failed to kill process: {}", e)
        })?;
        
        log::info!("[mcp_stop_server] Process (PID {}) killed, waiting for exit", pid);
        if let Err(e) = process.child.wait() {
            log::warn!("[mcp_stop_server] Error waiting for process (PID {}) exit: {}", pid, e);
        }
        
        log::info!("[mcp_stop_server] Server {} stopped successfully (PID {})", client_id, pid);
    } else {
        log::warn!("[mcp_stop_server] Server {} not found in running processes", client_id);
    }

    Ok(())
}

/// 检查进程是否仍在运行
fn check_process_alive(child: &mut Child) -> bool {
    // 尝试获取进程退出状态（非阻塞）
    match child.try_wait() {
        Ok(Some(status)) => {
            log::warn!("Process exited with status: {:?}", status);
            false
        }
        Ok(None) => {
            // 进程仍在运行
            true
        }
        Err(e) => {
            log::warn!("Failed to check process status: {}", e);
            // 如果无法检查，假设进程仍在运行
            true
        }
    }
}

#[tauri::command]
pub async fn mcp_execute_command(
    client_id: String,
    request: serde_json::Value,
    state: State<'_, McpState>,
) -> Result<serde_json::Value, String> {
    log::info!("[mcp_execute_command] Executing command for server: {}", client_id);
    log::debug!("[mcp_execute_command] Request: {}", request);
    
    // 检查服务器类型
    let server_type = {
        let config_guard = state.config.lock().unwrap();
        config_guard
            .as_ref()
            .and_then(|cfg| cfg.mcp_servers.get(&client_id))
            .and_then(|cfg| cfg.r#type.as_deref())
            .unwrap_or("stdio")
            .to_string()
    };
    
    // SSE 类型的服务器应该在前端处理，不在 Rust 后端处理
    if server_type == "sse" {
        return Err(format!(
            "SSE server '{}' should be handled in the frontend, not in Rust backend",
            client_id
        ));
    }
    
    let mut processes = state.processes.lock().unwrap();

    // 检查进程是否存在，如果不存在则尝试模糊匹配
    let actual_client_id = if processes.contains_key(&client_id) {
        client_id.clone()
    } else {
        // 尝试模糊匹配：查找包含 client_id 的服务器名称
        let mut matched_server: Option<String> = None;
        
        // 首先尝试精确匹配配置中的服务器名称
        let config_guard = state.config.lock().unwrap();
        if let Some(config) = config_guard.as_ref() {
            for server_name in config.mcp_servers.keys() {
                // 如果 client_id 是 server_name 的子串，或者 server_name 包含 client_id
                if server_name.contains(&client_id) || client_id.contains(server_name.as_str()) {
                    if processes.contains_key(server_name) {
                        matched_server = Some(server_name.clone());
                        log::info!(
                            "[mcp_execute_command] Matched server '{}' for requested '{}'",
                            server_name, client_id
                        );
                        break;
                    }
                }
            }
        }
        drop(config_guard);
        
        if let Some(matched) = matched_server {
            matched
        } else {
            // 如果找不到匹配的服务器，返回详细错误信息
            log::error!("[mcp_execute_command] Server '{}' not found in processes", client_id);
            
            // 获取所有可用的服务器名称
            let available_servers: Vec<String> = processes.keys().cloned().collect();
            let config_guard = state.config.lock().unwrap();
            let configured_servers: Vec<String> = config_guard
                .as_ref()
                .map(|cfg| cfg.mcp_servers.keys().cloned().collect())
                .unwrap_or_default();
            
            let mut error_msg = format!(
                "MCP server '{}' not found. ",
                client_id
            );
            
            if !available_servers.is_empty() {
                error_msg.push_str(&format!(
                    "Available running servers: {}. ",
                    available_servers.join(", ")
                ));
            }
            
            if !configured_servers.is_empty() {
                error_msg.push_str(&format!(
                    "Configured servers: {}. ",
                    configured_servers.join(", ")
                ));
            }
            
            // 检查是否有相似的服务器名称（可能是拼写错误）
            let similar_servers: Vec<String> = configured_servers
                .iter()
                .filter(|s| s.contains(&client_id) || client_id.contains(s.as_str()))
                .cloned()
                .collect();
            
            if !similar_servers.is_empty() {
                error_msg.push_str(&format!(
                    "Did you mean: {}? ",
                    similar_servers.join(", ")
                ));
            }
            
            error_msg.push_str("Please check the server name and ensure the server is started.");
            
            return Err(error_msg);
        }
    };
    
    // 使用实际匹配到的 client_id
    let client_id = actual_client_id;
    
    // 检查进程是否还在运行
    {
        let process = processes.get_mut(&client_id).unwrap();
        if !check_process_alive(&mut process.child) {
            log::error!("[mcp_execute_command] Server '{}' process has exited", client_id);
            // 从 processes 中移除已退出的进程
            processes.remove(&client_id);
            return Err(format!(
                "MCP server '{}' process has exited. Please restart the server.",
                client_id
            ));
        }
    }
    
    // 重新获取 process 引用（在检查后）
    let process = processes
        .get(&client_id)
        .ok_or_else(|| {
            log::error!("[mcp_execute_command] Server '{}' disappeared after check", client_id);
            format!("MCP server {} disappeared", client_id)
        })?;
    
    log::debug!("[mcp_execute_command] Found server process for: {} (PID: {})", 
        client_id, process.child.id());

    // 检查并执行初始化（在持有 initialized 锁期间完成）
    // 使用原子模式避免竞态：只有获得锁的线程才能执行初始化
    let needs_init = {
        let mut initialized = process.initialized.lock().unwrap();
        if *initialized {
            false
        } else {
            *initialized = true; // 立即标记为已初始化，防止其他线程进入
            true
        }
    };

    // 只有获得初始化资格的线程才执行初始化
    if needs_init {
        log::info!("[mcp_execute_command] Server {} needs initialization, starting init...", client_id);
        let mut stdin = process.stdin.lock().unwrap();
        let mut stdout = process.stdout.lock().unwrap();
        log::debug!("[mcp_execute_command] Acquired stdin/stdout locks for initialization");

        const INIT_TIMEOUT_MS: u64 = 10_000; // 10秒初始化超时
        log::debug!("[mcp_execute_command] Calling initialize_mcp_server with timeout: {}ms", INIT_TIMEOUT_MS);
        initialize_mcp_server(&mut *stdout, &mut *stdin, INIT_TIMEOUT_MS)
            .map_err(|e| {
                log::error!("[mcp_execute_command] Initialization failed: {}", e);
                // 初始化失败时重置状态，允许重试
                let mut initialized = process.initialized.lock().unwrap();
                *initialized = false;
                format!("MCP server initialization failed: {}", e)
            })?;

        log::info!("[mcp_execute_command] Server {} initialized successfully", client_id);
    } else {
        log::debug!("[mcp_execute_command] Server {} already initialized, skipping init", client_id);
    }

    // 确保请求包含必需的 JSON-RPC 字段
    let mut request_obj = request.as_object()
        .ok_or_else(|| "Request must be a JSON object".to_string())?
        .clone();
    
    // 添加 jsonrpc 字段（如果缺失）
    if !request_obj.contains_key("jsonrpc") {
        request_obj.insert("jsonrpc".to_string(), serde_json::Value::String("2.0".to_string()));
    }
    
    // 添加 id 字段（如果缺失）
    if !request_obj.contains_key("id") {
        // 使用时间戳作为 id
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        request_obj.insert("id".to_string(), serde_json::Value::Number(id.into()));
    }
    
    let final_request = serde_json::Value::Object(request_obj);

    // 将请求写入进程的 stdin
    log::debug!("[mcp_execute_command] Serializing request to JSON string");
    let request_str = serde_json::to_string(&final_request)
        .map_err(|e| {
            log::error!("[mcp_execute_command] Failed to serialize request: {}", e);
            format!("Failed to serialize request: {}", e)
        })?;
    log::debug!("[mcp_execute_command] Serialized request length: {} bytes", request_str.len());

    {
        log::debug!("[mcp_execute_command] Acquiring stdin lock to send request");
        let mut stdin = process.stdin.lock().unwrap();
        log::info!("[mcp_execute_command] Sending request to server {}: {}", client_id, final_request);
        log::debug!("[mcp_execute_command] Request string ({} bytes): {}", request_str.len(), request_str);
        
        writeln!(stdin, "{}", request_str)
            .map_err(|e| {
                log::error!("[mcp_execute_command] Failed to write to stdin: {}", e);
                format!("Failed to write to stdin: {}", e)
            })?;
        
        stdin
            .flush()
            .map_err(|e| {
                log::error!("[mcp_execute_command] Failed to flush stdin: {}", e);
                format!("Failed to flush stdin: {}", e)
            })?;
        log::info!("[mcp_execute_command] Request sent and flushed successfully");
    }

    // 从 stdout 读取响应（跳过 notification 消息）
    log::debug!("mcp_execute_command: about to acquire stdout lock...");
    let mut stdout = process.stdout.lock().unwrap();
    log::debug!("mcp_execute_command: acquired stdout lock");

    // 读取完整的 JSON 消息（支持多行）
    const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    const RESPONSE_TIMEOUT_MS: u64 = 10_000; // 10秒超时（缩短以便更快回退到 stderr）

    log::info!("Waiting for response from server {} (timeout: {}s)...", client_id, RESPONSE_TIMEOUT_MS / 1000);

    let response_str = match read_jsonrpc_response(&mut *stdout, MAX_MESSAGE_SIZE, RESPONSE_TIMEOUT_MS) {
        Ok(s) => {
            log::info!("Received response from server {} ({} bytes) via stdout", client_id, s.len());
            s
        }
        Err(e) => {
            log::warn!("stdout timeout/error: {}, checking stderr...", e);

            // 检查 stderr 是否有响应
            let stderr_buffer = process.stderr.lock().unwrap();
            log::debug!("stderr buffer: {} bytes", stderr_buffer.len());
            if !stderr_buffer.is_empty() {
                log::debug!("stderr content (first 500 chars): {}", 
                    &stderr_buffer[..std::cmp::min(500, stderr_buffer.len())]);
            }

            if let Some(response) = extract_jsonrpc_from_stderr(&stderr_buffer) {
                log::info!("Found response in stderr ({} bytes)", response.len());
                drop(stderr_buffer);
                response
            } else {
                drop(stderr_buffer);
                return Err(format!("Failed to read response from stdout: {}. Server may not be responding. Please check if the MCP server is running correctly.", e));
            }
        }
    };

    log::debug!("[mcp_execute_command] Parsing response JSON ({} bytes)", response_str.len());
    let response: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| {
            log::error!("[mcp_execute_command] Failed to parse response JSON: {}", e);
            log::error!("[mcp_execute_command] Response content: {}", response_str);
            format!("Failed to parse response: {}", e)
        })?;

    log::info!(
        "[mcp_execute_command] Successfully received and parsed response from server {}: {}",
        client_id, response
    );
    Ok(response)
}

#[tauri::command]
pub async fn mcp_get_server_status(
    state: State<'_, McpState>,
) -> Result<HashMap<String, ServerStatusResponse>, String> {
    let mut processes = state.processes.lock().unwrap();
    let config_guard = state.config.lock().unwrap();

    let mut result = HashMap::new();
    let mut dead_processes = Vec::new();

    if let Some(config) = config_guard.as_ref() {
        for (client_id, server_config) in &config.mcp_servers {
            let server_type = server_config.r#type.as_deref().unwrap_or("stdio");
            
            let status = if server_config.status.as_deref() == Some("paused") {
                ServerStatusResponse {
                    status: "paused".to_string(),
                    error_msg: None,
                }
            } else if server_type == "sse" {
                // SSE 类型的服务器不需要进程，直接返回 active
                ServerStatusResponse {
                    status: "active".to_string(),
                    error_msg: None,
                }
            } else if let Some(process) = processes.get_mut(client_id) {
                // stdio 类型：检查进程是否还在运行
                if check_process_alive(&mut process.child) {
                    ServerStatusResponse {
                        status: "active".to_string(),
                        error_msg: None,
                    }
                } else {
                    // 进程已退出，标记为待清理
                    log::warn!("[mcp_get_server_status] Server '{}' process has exited", client_id);
                    dead_processes.push(client_id.clone());
                    ServerStatusResponse {
                        status: "undefined".to_string(),
                        error_msg: Some("Process has exited".to_string()),
                    }
                }
            } else {
                ServerStatusResponse {
                    status: "undefined".to_string(),
                    error_msg: None,
                }
            };

            result.insert(client_id.clone(), status);
        }
    }

    // 清理已退出的进程
    for client_id in dead_processes {
        log::info!("[mcp_get_server_status] Removing dead process: {}", client_id);
        processes.remove(&client_id);
    }

    Ok(result)
}



