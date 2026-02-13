use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::data::Stock;

pub static ALERT_STORE: std::sync::LazyLock<RwLock<AlertStore>> =
    std::sync::LazyLock::new(|| RwLock::new(AlertStore::default()));

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertRuleKind {
    PriceAbove,
    PriceBelow,
    ChangePercentAbove,
    ChangePercentBelow,
    VolumeAbove,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AlertRuleStatus {
    #[default]
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub symbol: String,
    pub kind: AlertRuleKind,
    pub threshold: Decimal,
    pub status: AlertRuleStatus,
    pub cooldown_seconds: u64,
    pub last_triggered_at_unix: Option<i64>,
    pub created_at_unix: i64,
    pub updated_at_unix: i64,
}

impl AlertRule {
    fn should_skip_by_cooldown(&self, now_unix: i64) -> bool {
        let Some(last) = self.last_triggered_at_unix else {
            return false;
        };
        let cooldown = i64::try_from(self.cooldown_seconds).unwrap_or(i64::MAX);
        now_unix.saturating_sub(last) < cooldown
    }

    fn is_triggered_by(&self, stock: &Stock) -> bool {
        let Some(last_done) = stock.quote.last_done else {
            return false;
        };

        match self.kind {
            AlertRuleKind::PriceAbove => last_done >= self.threshold,
            AlertRuleKind::PriceBelow => last_done <= self.threshold,
            AlertRuleKind::ChangePercentAbove => {
                let Some(prev_close) = stock.quote.prev_close else {
                    return false;
                };
                if prev_close <= Decimal::ZERO {
                    return false;
                }
                let pct = (last_done - prev_close) / prev_close * Decimal::from(100);
                pct >= self.threshold
            }
            AlertRuleKind::ChangePercentBelow => {
                let Some(prev_close) = stock.quote.prev_close else {
                    return false;
                };
                if prev_close <= Decimal::ZERO {
                    return false;
                }
                let pct = (last_done - prev_close) / prev_close * Decimal::from(100);
                pct <= self.threshold
            }
            AlertRuleKind::VolumeAbove => Decimal::from(stock.quote.volume) >= self.threshold,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertStore {
    pub version: u8,
    pub rules: Vec<AlertRule>,
}

impl Default for AlertStore {
    fn default() -> Self {
        Self {
            version: 1,
            rules: Vec::new(),
        }
    }
}

#[must_use]
pub fn alert_store_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::home_dir()
            .or_else(dirs::data_local_dir)
            .unwrap_or_else(std::env::temp_dir);
        path.push("Library/Application Support/ChangQiao");
        path.push("alerts.json");
        path
    }
    #[cfg(target_os = "windows")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(std::env::temp_dir);
        path.push("ChangQiao");
        path.push("alerts.json");
        path
    }
    #[cfg(target_os = "linux")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(std::env::temp_dir);
        path.push("changqiao");
        path.push("alerts.json");
        path
    }
}

pub fn load_from_disk() -> std::io::Result<usize> {
    let path = alert_store_path();
    let store = load_store_from_path(&path)?;
    let count = store.rules.len();
    *ALERT_STORE.write().expect("poison") = store;
    Ok(count)
}

pub fn save_to_disk() -> std::io::Result<()> {
    let path = alert_store_path();
    let store = ALERT_STORE.read().expect("poison").clone();
    save_store_to_path(&store, &path)
}

pub fn evaluate_quote(symbol: &str, stock: &Stock) {
    let now = now_unix();
    let mut triggered_logs = Vec::new();
    let mut mutated = false;

    {
        let mut store = ALERT_STORE.write().expect("poison");
        for rule in &mut store.rules {
            if rule.status != AlertRuleStatus::Enabled {
                continue;
            }
            if rule.symbol != symbol {
                continue;
            }
            if rule.should_skip_by_cooldown(now) {
                continue;
            }
            if !rule.is_triggered_by(stock) {
                continue;
            }

            rule.last_triggered_at_unix = Some(now);
            rule.updated_at_unix = now;
            mutated = true;
            triggered_logs.push(format!(
                "规则命中：id={}, symbol={}, kind={:?}, threshold={}",
                rule.id, rule.symbol, rule.kind, rule.threshold
            ));
        }
    }

    for log in triggered_logs {
        tracing::warn!(event = "alert.triggered", "{log}");
    }

    if mutated {
        if let Err(err) = save_to_disk() {
            tracing::warn!(error = %err, "预警规则状态保存失败");
        }
    }
}

fn load_store_from_path(path: &Path) -> std::io::Result<AlertStore> {
    if !path.exists() {
        return Ok(AlertStore::default());
    }

    let bytes = std::fs::read(path)?;
    match serde_json::from_slice::<AlertStore>(&bytes) {
        Ok(store) => Ok(store),
        Err(err) => {
            tracing::warn!(
                path = %path.display(),
                error = %err,
                "预警规则文件解析失败，将备份并重置为空"
            );
            backup_corrupted_file(path, &bytes);
            Ok(AlertStore::default())
        }
    }
}

fn save_store_to_path(store: &AlertStore, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)?;

    let data = serde_json::to_vec_pretty(store).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("序列化预警规则失败：{err}"),
        )
    })?;

    file.write_all(&data)?;
    file.flush()?;
    drop(file);
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

fn backup_corrupted_file(path: &Path, bytes: &[u8]) {
    let backup = path.with_extension(format!("json.corrupt.{}.bak", now_unix()));
    if let Some(parent) = backup.parent() {
        _ = std::fs::create_dir_all(parent);
    }
    _ = std::fs::write(backup, bytes);
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{AlertRule, AlertRuleKind, AlertRuleStatus, AlertStore};
    use crate::data::{Counter, Stock};
    use rust_decimal::Decimal;

    #[test]
    fn price_above_rule_triggers_correctly() {
        let mut stock = Stock::new(Counter::new("AAPL.US"));
        stock.quote.last_done = Some(Decimal::from(101));

        let rule = AlertRule {
            id: "r1".to_string(),
            symbol: "AAPL.US".to_string(),
            kind: AlertRuleKind::PriceAbove,
            threshold: Decimal::from(100),
            status: AlertRuleStatus::Enabled,
            cooldown_seconds: 0,
            last_triggered_at_unix: None,
            created_at_unix: 0,
            updated_at_unix: 0,
        };

        assert!(rule.is_triggered_by(&stock));
    }

    #[test]
    fn store_default_is_empty() {
        let store = AlertStore::default();
        assert_eq!(store.version, 1);
        assert!(store.rules.is_empty());
    }
}
