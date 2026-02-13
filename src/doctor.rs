use std::io::{IsTerminal, Write};
use std::net::ToSocketAddrs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    #[must_use]
    fn marker(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}

#[derive(Clone, Debug)]
struct CheckItem {
    name: &'static str,
    status: CheckStatus,
    detail: String,
}

impl CheckItem {
    #[must_use]
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Pass,
            detail: detail.into(),
        }
    }

    #[must_use]
    fn warn(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            detail: detail.into(),
        }
    }

    #[must_use]
    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Fail,
            detail: detail.into(),
        }
    }
}

#[must_use]
pub fn run() -> i32 {
    let checks = vec![
        check_tty(),
        check_required_env(),
        check_log_dir(),
        check_dns_resolution(),
        check_instance_lock(),
    ];

    println!("长桥终端 诊断报告");
    println!("============================================================");
    for item in &checks {
        println!(
            "[{}] {:<20} {}",
            item.status.marker(),
            item.name,
            item.detail
        );
    }
    println!("============================================================");

    let has_fail = checks.iter().any(|item| item.status == CheckStatus::Fail);
    let has_warn = checks.iter().any(|item| item.status == CheckStatus::Warn);

    if has_fail {
        println!("结论：存在阻塞项，请先修复 FAIL 项后再启动 changqiao。");
        return 2;
    }

    if has_warn {
        println!("结论：可继续使用，但建议处理 WARN 项以降低运行风险。");
    } else {
        println!("结论：诊断通过，可以启动 changqiao。");
    }
    0
}

fn check_tty() -> CheckItem {
    if std::io::stdout().is_terminal() {
        CheckItem::pass("交互式终端", "stdout 是 TTY。")
    } else {
        CheckItem::warn(
            "交互式终端",
            "stdout 不是 TTY；此命令可运行，但 changqiao 主程序会拒绝启动。",
        )
    }
}

fn check_required_env() -> CheckItem {
    let missing = crate::openapi::missing_required_env();
    if missing.is_empty() {
        return CheckItem::pass(
            "必需环境变量",
            "LONGPORT_APP_KEY / SECRET / ACCESS_TOKEN 已配置。",
        );
    }

    CheckItem::fail("必需环境变量", format!("缺失：{}", missing.join(", ")))
}

fn check_log_dir() -> CheckItem {
    let primary = crate::logger::default_log_dir();
    if let Err(err) = ensure_writable_dir(&primary) {
        let fallback = std::env::temp_dir().join("changqiao/logs");
        if let Err(fallback_err) = ensure_writable_dir(&fallback) {
            return CheckItem::fail(
                "日志目录写入",
                format!("默认目录不可写（{err}）；临时目录也不可写（{fallback_err}）。"),
            );
        }

        return CheckItem::warn(
            "日志目录写入",
            format!(
                "默认目录不可写（{}），将降级到临时目录：{}。",
                err,
                fallback.display()
            ),
        );
    }

    CheckItem::pass("日志目录写入", format!("可写：{}。", primary.display()))
}

fn check_dns_resolution() -> CheckItem {
    let targets = dns_targets();
    let mut failures = Vec::new();
    let mut resolved = Vec::new();

    for target in targets {
        match target.to_socket_addrs() {
            Ok(mut addrs) => {
                if addrs.next().is_some() {
                    resolved.push(target);
                } else {
                    failures.push(format!("{target}（无可用地址）"));
                }
            }
            Err(err) => failures.push(format!("{target}（{err}）")),
        }
    }

    if resolved.is_empty() {
        return CheckItem::warn(
            "网络 DNS",
            format!("全部解析失败：{}。", failures.join("；")),
        );
    }

    if failures.is_empty() {
        return CheckItem::pass("网络 DNS", format!("解析成功：{}。", resolved.join(", ")));
    }

    CheckItem::warn(
        "网络 DNS",
        format!(
            "部分解析成功（{}）；失败项：{}。",
            resolved.join(", "),
            failures.join("；")
        ),
    )
}

fn check_instance_lock() -> CheckItem {
    match crate::instance_lock::acquire() {
        Ok(_guard) => CheckItem::pass("单实例锁", "可成功获取锁。"),
        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => CheckItem::warn(
            "单实例锁",
            "检测到已有 changqiao 进程在运行，主程序直接启动会失败。",
        ),
        Err(err) => CheckItem::fail("单实例锁", format!("无法获取锁：{err}")),
    }
}

fn ensure_writable_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let probe_path = path.join(format!("doctor_write_probe_{}.tmp", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)?;
    file.write_all(b"probe")?;
    file.flush()?;
    std::fs::remove_file(probe_path)?;
    Ok(())
}

fn dns_targets() -> Vec<String> {
    let mut targets = vec!["open.longbridge.com:443".to_string()];

    for (key, default_port) in [
        ("LONGPORT_HTTP_URL", 443_u16),
        ("LONGPORT_QUOTE_WS_URL", 443),
    ] {
        if let Ok(raw) = std::env::var(key) {
            if let Some(target) = endpoint_to_host_port(&raw, default_port) {
                targets.push(target);
            }
        }
    }

    targets.sort();
    targets.dedup();
    targets
}

fn endpoint_to_host_port(raw: &str, default_port: u16) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_scheme = if let Some((_, rest)) = trimmed.split_once("://") {
        rest
    } else {
        trimmed
    };
    let authority = without_scheme.split('/').next()?.rsplit('@').next()?;
    if authority.is_empty() {
        return None;
    }

    if authority.starts_with('[') {
        let end = authority.find(']')?;
        let host = &authority[..=end];
        let rest = authority.get(end + 1..).unwrap_or_default();
        if let Some(port) = rest.strip_prefix(':') {
            return Some(format!("{host}:{port}"));
        }
        return Some(format!("{host}:{default_port}"));
    }

    if authority.contains(':') {
        return Some(authority.to_string());
    }

    Some(format!("{authority}:{default_port}"))
}

#[cfg(test)]
mod tests {
    use super::endpoint_to_host_port;

    #[test]
    fn parses_https_endpoint() {
        assert_eq!(
            endpoint_to_host_port("https://api.example.com/v1", 443),
            Some("api.example.com:443".to_string())
        );
    }

    #[test]
    fn keeps_existing_port() {
        assert_eq!(
            endpoint_to_host_port("wss://quote.example.com:18080/stream", 443),
            Some("quote.example.com:18080".to_string())
        );
    }

    #[test]
    fn handles_ipv6() {
        assert_eq!(
            endpoint_to_host_port("https://[2001:db8::1]:8443/ws", 443),
            Some("[2001:db8::1]:8443".to_string())
        );
    }
}
