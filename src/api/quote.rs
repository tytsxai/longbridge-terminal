// OpenAPI SDK automatically manages device connections, no need to manually call online API
// Keep this file for compatibility with existing code references

use crate::openapi;
use anyhow::Result;

/// Fetch stock static information
pub async fn fetch_static_info(
    symbols: &[String],
) -> Result<Vec<longport::quote::SecurityStaticInfo>> {
    openapi::helpers::get_static_info(symbols).await
}

/// Fetch recent trades for a symbol
pub async fn fetch_trades(symbol: &str, count: usize) -> Result<Vec<longport::quote::Trade>> {
    openapi::helpers::get_trades(symbol, count).await
}
