use std::path::{Path, PathBuf};

pub struct InstanceGuard {
    _guard: crate::os::FileGuard,
}

pub fn lock_file_path() -> PathBuf {
    if let Some(mut path) =
        crate::path_env::dir_override("CHANGQIAO_DATA_DIR", "LONGBRIDGE_DATA_DIR")
    {
        path.push("changqiao.lock");
        return path;
    }

    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
        path.push("Library/Application Support/ChangQiao");
        path.push("changqiao.lock");
        path
    }
    #[cfg(target_os = "windows")]
    {
        let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
        path.push("ChangQiao");
        path.push("changqiao.lock");
        path
    }
    #[cfg(target_os = "linux")]
    {
        let mut path = dirs::data_local_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .unwrap_or_else(std::env::temp_dir);
        path.push("changqiao");
        path.push("changqiao.lock");
        path
    }
}

pub fn acquire() -> std::io::Result<InstanceGuard> {
    let lock_path = lock_file_path();
    create_parent_dir(&lock_path)?;
    let guard = crate::os::flock(&lock_path)?;
    Ok(InstanceGuard { _guard: guard })
}

fn create_parent_dir(path: &Path) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Lock file path has no parent directory",
        )
    })?;
    std::fs::create_dir_all(parent)
}

#[cfg(test)]
mod tests {
    use super::lock_file_path;

    #[test]
    fn lock_file_path_has_filename() {
        let path = lock_file_path();
        assert!(
            path.file_name()
                .is_some_and(|name| name == "changqiao.lock"),
            "lock path should end with changqiao.lock"
        );
    }
}
