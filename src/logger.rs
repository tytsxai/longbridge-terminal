use std::any::Any;
use std::path::PathBuf;

use std::sync::OnceLock;

static ACTIVE_LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn default_log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::home_dir()
            .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(std::env::temp_dir));
        path.push("Library/Logs/ChangQiao");
        path
    }
    #[cfg(target_os = "windows")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(std::env::temp_dir);
        path.push("ChangQiao\\Logs");
        path
    }
    #[cfg(target_os = "linux")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(std::env::temp_dir);
        path.push("changqiao/logs");
        path
    }
}

fn fallback_log_dir() -> PathBuf {
    std::env::temp_dir().join("changqiao").join("logs")
}

#[must_use]
pub fn active_log_dir() -> PathBuf {
    ACTIVE_LOG_DIR
        .get()
        .cloned()
        .unwrap_or_else(default_log_dir)
}

fn local_offset() -> time::UtcOffset {
    time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
}

#[must_use]
pub fn init() -> impl Any {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let primary_log_dir = default_log_dir();
    let log_dir = if std::fs::create_dir_all(&primary_log_dir).is_ok() {
        primary_log_dir
    } else {
        let fallback = fallback_log_dir();
        let _ = std::fs::create_dir_all(&fallback);
        fallback
    };

    let _ = ACTIVE_LOG_DIR.set(log_dir.clone());

    let writer = match RollingFileAppender::builder()
        .filename_prefix("changqiao")
        .filename_suffix("log")
        .max_log_files(5)
        .rotation(Rotation::DAILY)
        .build(&log_dir)
    {
        Ok(writer) => writer,
        Err(err) => {
            eprintln!("日志初始化失败（目录: {}）：{}", log_dir.display(), err);
            std::process::exit(1);
        }
    };
    let (writer, guard) = tracing_appender::non_blocking(writer);

    let timer = fmt::time::OffsetTime::new(
        local_offset(),
        time::format_description::well_known::Rfc3339,
    );
    let file_line = cfg!(debug_assertions);

    let subscriber = fmt::layer()
        .with_ansi(false)
        .with_timer(timer)
        .with_thread_ids(true)
        .with_file(file_line)
        .with_line_number(file_line)
        .with_writer(writer);

    let dirs = "error,changqiao=debug";
    let dirs = std::env::var("CHANGQIAO_LOG")
        .or_else(|_| std::env::var("LONGBRIDGE_LOG"))
        .unwrap_or_else(|_| dirs.to_string());
    let subscriber = subscriber.with_filter(tracing_subscriber::EnvFilter::new(dirs));

    tracing_subscriber::registry().with(subscriber).init();
    guard
}
