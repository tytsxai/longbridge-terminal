use anyhow::Result;
use std::sync::{Arc, OnceLock};

use super::wrapper::{RateLimitedQuoteContext, RateLimitedTradeContext};

/// Global `QuoteContext`
pub static QUOTE_CTX: OnceLock<longport::quote::QuoteContext> = OnceLock::new();

/// Global `TradeContext`
pub static TRADE_CTX: OnceLock<longport::trade::TradeContext> = OnceLock::new();

/// Global rate-limited `QuoteContext` wrapper
pub static RATE_LIMITED_QUOTE_CTX: OnceLock<RateLimitedQuoteContext> = OnceLock::new();

/// Global rate-limited `TradeContext` wrapper
pub static RATE_LIMITED_TRADE_CTX: OnceLock<RateLimitedTradeContext> = OnceLock::new();

/// Get API language based on current UI locale
/// Maps UI locale to API-supported languages: en, zh-CN, zh-HK
/// Defaults to "en" if locale is not supported
fn get_api_language() -> &'static str {
    match std::env::var("LONGBRIDGE_LOCALE").ok().as_deref() {
        Some("zh-CN") => "zh-CN",
        Some("zh-HK" | "zh-TW") => "zh-HK",
        _ => "en",
    }
}

#[must_use]
pub fn missing_required_env() -> Vec<&'static str> {
    [
        "LONGPORT_APP_KEY",
        "LONGPORT_APP_SECRET",
        "LONGPORT_ACCESS_TOKEN",
    ]
    .into_iter()
    .filter(|key| {
        std::env::var(key)
            .ok()
            .is_none_or(|value| value.trim().is_empty())
    })
    .collect()
}

/// Initialize contexts (should be called once at app startup)
/// Returns quote receiver for caller to handle WebSocket events
pub async fn init_contexts(
) -> Result<impl tokio_stream::Stream<Item = longport::quote::PushEvent> + Send + Unpin> {
    // Set language based on current UI locale
    std::env::set_var("LONGPORT_LANGUAGE", get_api_language());
    std::env::set_var("LONGPORT_PRINT_QUOTE_PACKAGES", "false");

    // Load config from environment variables
    let config = Arc::new(longport::Config::from_env()?);

    // Create QuoteContext and TradeContext
    let (quote_ctx, quote_receiver) =
        longport::quote::QuoteContext::try_new(Arc::clone(&config)).await?;
    let (trade_ctx, _trade_receiver) =
        longport::trade::TradeContext::try_new(Arc::clone(&config)).await?;

    // Store in global variables
    QUOTE_CTX
        .set(quote_ctx)
        .map_err(|_| anyhow::anyhow!("QuoteContext already initialized"))?;
    TRADE_CTX
        .set(trade_ctx)
        .map_err(|_| anyhow::anyhow!("TradeContext already initialized"))?;

    // Initialize rate-limited wrappers
    let quote_ref = QUOTE_CTX.get().expect("QuoteContext just initialized");
    let trade_ref = TRADE_CTX.get().expect("TradeContext just initialized");

    RATE_LIMITED_QUOTE_CTX
        .set(RateLimitedQuoteContext::new(quote_ref))
        .map_err(|_| anyhow::anyhow!("RateLimitedQuoteContext already initialized"))?;
    RATE_LIMITED_TRADE_CTX
        .set(RateLimitedTradeContext::new(trade_ref))
        .map_err(|_| anyhow::anyhow!("RateLimitedTradeContext already initialized"))?;

    tracing::info!("Rate limiter initialized: 10 requests/second, burst capacity: 20");

    // Wrap as Stream
    Ok(tokio_stream::wrappers::UnboundedReceiverStream::new(
        quote_receiver,
    ))
}

/// Get global `QuoteContext`
pub fn quote() -> &'static longport::quote::QuoteContext {
    QUOTE_CTX
        .get()
        .expect("QuoteContext not initialized, please call init_contexts() first")
}

/// Get global `TradeContext`
pub fn trade() -> &'static longport::trade::TradeContext {
    TRADE_CTX
        .get()
        .expect("TradeContext not initialized, please call init_contexts() first")
}

/// Get rate-limited `QuoteContext` (recommended for all API calls)
pub fn quote_limited() -> &'static RateLimitedQuoteContext {
    RATE_LIMITED_QUOTE_CTX
        .get()
        .expect("RateLimitedQuoteContext not initialized, please call init_contexts() first")
}

/// Get rate-limited `TradeContext` (recommended for all API calls)
pub fn trade_limited() -> &'static RateLimitedTradeContext {
    RATE_LIMITED_TRADE_CTX
        .get()
        .expect("RateLimitedTradeContext not initialized, please call init_contexts() first")
}

/// Display config guide (when config loading fails)
pub fn print_config_guide() {
    eprintln!("配置错误：缺少必需环境变量");
    eprintln!();
    eprintln!("请先配置以下环境变量：");
    eprintln!("  LONGPORT_APP_KEY=<your_app_key>");
    eprintln!("  LONGPORT_APP_SECRET=<your_app_secret>");
    eprintln!("  LONGPORT_ACCESS_TOKEN=<your_access_token>");
    eprintln!();
    eprintln!("可选：通过 LONGPORT_HTTP_URL 与 LONGPORT_QUOTE_WS_URL 指定自定义服务地址");
    eprintln!("可选：通过 LONGBRIDGE_LOCALE 指定界面语言（如 zh-CN / en）");
    eprintln!("可选：通过 LONGBRIDGE_LOG 调整日志过滤（如 error,longbridge=info）");
    eprintln!();
    eprintln!("获取 Token: https://open.longbridge.com");
    eprintln!();
    eprintln!("提示：你可以在项目根目录创建 .env 文件来管理这些变量");
}

#[cfg(test)]
mod tests {
    use super::{get_api_language, missing_required_env};

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = std::env::var(key).ok();
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn detects_missing_required_environment_variables() {
        let _app_key = EnvGuard::set("LONGPORT_APP_KEY", Some(""));
        let _app_secret = EnvGuard::set("LONGPORT_APP_SECRET", None);
        let _access_token = EnvGuard::set("LONGPORT_ACCESS_TOKEN", Some("token"));

        let missing = missing_required_env();
        assert!(missing.contains(&"LONGPORT_APP_KEY"));
        assert!(missing.contains(&"LONGPORT_APP_SECRET"));
        assert!(!missing.contains(&"LONGPORT_ACCESS_TOKEN"));
    }

    #[test]
    fn maps_locale_to_supported_api_language() {
        let _locale = EnvGuard::set("LONGBRIDGE_LOCALE", Some("zh-HK"));
        assert_eq!(get_api_language(), "zh-HK");

        let _locale = EnvGuard::set("LONGBRIDGE_LOCALE", Some("en-US"));
        assert_eq!(get_api_language(), "en");

        let _locale = EnvGuard::set("LONGBRIDGE_LOCALE", Some("unknown"));
        assert_eq!(get_api_language(), "en");
    }
}
