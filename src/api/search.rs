use crate::data::Counter;
use crate::openapi;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StockItem {
    /// Stock code
    pub code: String,
    /// Stock `counter_id`
    pub counter_id: Counter,
    /// Trading currency
    pub currency: String,
    /// Stock market
    pub market: String,
    /// Stock name
    pub name: String,
    /// Stock product
    pub product: String,
    /// Search score
    pub score: f64,
    /// Stock type
    #[serde(rename = "type")]
    pub product_type: String,
}

impl PartialEq for StockItem {
    fn eq(&self, other: &Self) -> bool {
        self.counter_id == other.counter_id
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct StockResult {
    pub product_list: Vec<StockItem>,
    pub recommend_list: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StockQuery {
    #[serde(rename = "k")]
    pub keyword: String,
    pub market: String,
    pub product: String,
    pub account_channel: String,
}

/// Search stocks
pub async fn fetch_stock(query: &StockQuery) -> Result<StockResult> {
    // Use rate-limited wrapper to avoid API burst and 429 errors
    let symbols = openapi::helpers::get_static_info([query.keyword.as_str()]).await?;
    let locale = rust_i18n::locale();

    let product_list = symbols
        .iter()
        .map(|info| {
            // Parse market and code from symbol (e.g. AAPL.US, .DJI.US)
            let (code, market) = info
                .symbol
                .rsplit_once('.')
                .map_or((info.symbol.clone(), String::new()), |(code, market)| {
                    (code.to_string(), market.to_string())
                });

            let display_name = if locale.starts_with("zh") {
                info.name_cn.clone()
            } else {
                info.name_en.clone()
            };

            StockItem {
                code: code.clone(),
                counter_id: Counter::new(&info.symbol),
                currency: String::new(), // longport's static_info may not provide currency
                market,
                name: display_name,
                product: String::new(),
                score: 1.0,
                product_type: String::new(),
            }
        })
        .collect();

    Ok(StockResult {
        product_list,
        recommend_list: None,
    })
}
