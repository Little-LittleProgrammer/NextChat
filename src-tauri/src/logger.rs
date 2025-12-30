use log::{LevelFilter, SetLoggerError};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::path::PathBuf;

/// 初始化日志系统，同时输出到控制台和文件
pub fn init_logger(app_data_dir: Option<PathBuf>) -> Result<(), SetLoggerError> {
    // 确定日志文件路径
    let log_file_path = if let Some(app_dir) = app_data_dir {
        app_dir.join("nextchat.log")
    } else {
        // 如果无法获取应用数据目录，使用当前目录
        PathBuf::from("nextchat.log")
    };

    // 创建日志文件（如果不存在）
    let log_file = File::create(&log_file_path).unwrap_or_else(|e| {
        eprintln!("Failed to create log file at {:?}: {}", log_file_path, e);
        // 如果创建失败，尝试使用临时文件
        std::fs::File::create("nextchat.log").expect("Failed to create fallback log file")
    });

    // 使用默认配置
    let config = Config::default();

    // 创建组合日志记录器：同时输出到终端和文件
    CombinedLogger::init(vec![
        // 终端日志记录器
        TermLogger::new(
            LevelFilter::Debug,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // 文件日志记录器
        WriteLogger::new(
            LevelFilter::Debug,
            config,
            log_file,
        ),
    ])
}

