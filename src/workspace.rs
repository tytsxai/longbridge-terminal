use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::data::{Counter, KlineType};

static STARTUP_SELECTED_COUNTER: std::sync::LazyLock<Mutex<Option<Counter>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WorkspaceSnapshot {
    pub version: u8,
    pub saved_at_unix: i64,
    pub last_state: Option<String>,
    pub watchlist_group_id: Option<u64>,
    pub watchlist_sort_by: (u8, u8, bool),
    pub watchlist_hidden: bool,
    pub selected_counter: Option<Counter>,
    pub stock_detail_counter: Option<Counter>,
    pub kline_type: KlineType,
    pub kline_index: usize,
    pub log_panel_visible: bool,
}

impl WorkspaceSnapshot {
    #[must_use]
    pub fn empty_now() -> Self {
        Self {
            version: 1,
            saved_at_unix: now_unix(),
            watchlist_sort_by: (0, 0, false),
            kline_type: KlineType::PerDay,
            ..Self::default()
        }
    }
}

#[must_use]
pub fn workspace_file_path() -> PathBuf {
    if let Some(mut path) =
        crate::path_env::dir_override("CHANGQIAO_DATA_DIR", "LONGBRIDGE_DATA_DIR")
    {
        path.push("workspace.json");
        return path;
    }

    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::home_dir()
            .or_else(dirs::data_local_dir)
            .unwrap_or_else(std::env::temp_dir);
        path.push("Library/Application Support/ChangQiao");
        path.push("workspace.json");
        path
    }
    #[cfg(target_os = "windows")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(std::env::temp_dir);
        path.push("ChangQiao");
        path.push("workspace.json");
        path
    }
    #[cfg(target_os = "linux")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(std::env::temp_dir);
        path.push("changqiao");
        path.push("workspace.json");
        path
    }
}

pub fn load() -> Option<WorkspaceSnapshot> {
    let path = workspace_file_path();
    let bytes = std::fs::read(&path).ok()?;
    match serde_json::from_slice::<WorkspaceSnapshot>(&bytes) {
        Ok(snapshot) => Some(snapshot),
        Err(err) => {
            tracing::warn!(
                path = %path.display(),
                error = %err,
                "工作区文件解析失败，将备份后重置"
            );
            backup_corrupted_file(&path, &bytes);
            None
        }
    }
}

pub fn save(snapshot: &WorkspaceSnapshot) -> std::io::Result<()> {
    let path = workspace_file_path();
    save_to_path(snapshot, &path)
}

pub fn set_startup_selected_counter(counter: Option<Counter>) {
    *STARTUP_SELECTED_COUNTER.lock().expect("poison") = counter;
}

pub fn take_startup_selected_counter() -> Option<Counter> {
    STARTUP_SELECTED_COUNTER.lock().expect("poison").take()
}

fn save_to_path(snapshot: &WorkspaceSnapshot, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)?;

    let data = serde_json::to_vec_pretty(snapshot).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("序列化工作区快照失败：{err}"),
        )
    })?;

    file.write_all(&data)?;
    file.flush()?;
    drop(file);
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

fn backup_corrupted_file(path: &Path, bytes: &[u8]) {
    let backup_path = path.with_extension(format!("json.corrupt.{}.bak", now_unix()));
    if let Some(parent) = backup_path.parent() {
        _ = std::fs::create_dir_all(parent);
    }
    _ = std::fs::write(backup_path, bytes);
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{save_to_path, WorkspaceSnapshot};
    use crate::data::Counter;
    use std::path::PathBuf;

    struct TempFileGuard {
        path: PathBuf,
    }

    impl TempFileGuard {
        fn new(path: PathBuf) -> Self {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("failed to create parent dir");
            }
            Self { path }
        }
    }

    impl Drop for TempFileGuard {
        fn drop(&mut self) {
            _ = std::fs::remove_file(&self.path);
        }
    }

    #[test]
    fn saves_snapshot_as_json() {
        let path = std::env::temp_dir().join("changqiao_workspace_save_test.json");
        let _guard = TempFileGuard::new(path.clone());

        let mut snapshot = WorkspaceSnapshot::empty_now();
        snapshot.selected_counter = Some(Counter::new("AAPL.US"));
        save_to_path(&snapshot, &path).expect("failed to save snapshot");

        let data = std::fs::read_to_string(&path).expect("failed to read snapshot");
        assert!(data.contains("AAPL.US"));
    }
}
