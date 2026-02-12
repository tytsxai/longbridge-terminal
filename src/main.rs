use crate::widgets::Terminal;
use std::io::{IsTerminal, Write};

#[macro_use]
mod macros;

pub mod api;
pub mod app;
pub mod cli;
pub mod data;
pub mod helper;
pub mod instance_lock;
pub mod kline;
pub mod logger;
pub mod openapi;
#[cfg_attr(target_family = "windows", path = "os/windows.rs")]
#[cfg_attr(target_family = "unix", path = "os/unix.rs")]
pub mod os;
pub mod render;
pub mod system;
pub mod ui;
pub mod widgets;

mod views;

#[macro_use]
extern crate rust_i18n;
i18n!("locales");

pub use cli::Args;

#[tokio::main]
async fn main() {
    let bin_name = std::env::args()
        .next()
        .unwrap_or_else(|| "changqiao".to_string());

    let command = match cli::parse_args(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("{}", err.message);
            std::process::exit(err.code);
        }
    };

    let args = match command {
        cli::Command::Help => {
            println!("{}", cli::help_text(&bin_name));
            return;
        }
        cli::Command::Version => {
            println!("{}", cli::version_text());
            return;
        }
        cli::Command::Run(args) => args,
    };

    dotenvy::dotenv().ok();

    if !std::io::stdout().is_terminal() {
        eprintln!("长桥终端 需要在交互式终端（TTY）中运行。");
        std::process::exit(1);
    }

    let _instance_lock = match instance_lock::acquire() {
        Ok(lock) => lock,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::WouldBlock {
                eprintln!("已有 changqiao 进程在运行，请先关闭后再启动。");
            } else {
                eprintln!("获取进程锁失败：{err}");
            }
            std::process::exit(3);
        }
    };

    // Set default locale to Chinese
    let locale = std::env::var("CHANGQIAO_LOCALE")
        .or_else(|_| std::env::var("LONGBRIDGE_LOCALE"))
        .unwrap_or_else(|_| "zh-CN".to_string());
    rust_i18n::set_locale(&locale);

    // Initialize logger
    let _guard = logger::init();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        pid = std::process::id(),
        locale = %locale,
        log_dir = %logger::active_log_dir().display(),
        "应用启动"
    );

    let missing_env = openapi::missing_required_env();
    if !missing_env.is_empty() {
        openapi::print_config_guide();
        eprintln!("\n缺少必需环境变量：{}", missing_env.join(", "));
        std::process::exit(2);
    }

    // Initialize OpenAPI first (before entering fullscreen mode, so SDK outputs stay in main screen)
    let quote_receiver = match openapi::init_contexts().await {
        Ok(receiver) => receiver,
        Err(e) => {
            let sanitized = sanitize_startup_error(&e.to_string());
            eprintln!("\nOpenAPI 初始化失败：{sanitized}");
            tracing::error!(error = %sanitized, "OpenAPI 初始化失败");
            std::process::exit(2);
        }
    };

    tracing::info!("OpenAPI 初始化成功");

    // Set up panic hook to restore terminal
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        Terminal::exit_full_screen();
        hook(info);
    }));

    // Clean terminal state to ensure no residual output

    let _ = std::io::stdout().write_all(b"\n");
    let _ = std::io::stdout().flush();

    // Now enter fullscreen mode (SDK is initialized, alternate screen is clean)
    Terminal::enter_full_screen();
    tokio::select! {
        _ = app::run(args, quote_receiver) => {
            tracing::info!("应用主循环已退出");
        }
        _ = wait_for_shutdown_signal() => {
            tracing::warn!("收到退出信号，正在退出");
        }
    }
    Terminal::exit_full_screen();
}

fn sanitize_startup_error(message: &str) -> String {
    let mut sanitized = message.to_string();
    for key in [
        "LONGPORT_APP_KEY",
        "LONGPORT_APP_SECRET",
        "LONGPORT_ACCESS_TOKEN",
    ] {
        if let Ok(secret) = std::env::var(key) {
            if !secret.is_empty() {
                sanitized = sanitized.replace(&secret, "***");
            }
        }
    }
    sanitized
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() {
    use std::future::pending;
    use tokio::signal::unix::{signal, Signal, SignalKind};

    async fn recv_or_pending(signal: Option<Signal>) {
        let mut signal = signal;
        if let Some(sig) = signal.as_mut() {
            let _ = sig.recv().await;
            return;
        }
        pending::<()>().await;
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = recv_or_pending(signal(SignalKind::terminate()).ok()) => {}
        _ = recv_or_pending(signal(SignalKind::hangup()).ok()) => {}
        _ = recv_or_pending(signal(SignalKind::interrupt()).ok()) => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
