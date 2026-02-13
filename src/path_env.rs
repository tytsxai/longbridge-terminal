use std::path::PathBuf;

/// 从环境变量读取目录覆盖配置（优先新变量，回退旧变量）。
#[must_use]
pub fn dir_override(primary_key: &str, legacy_key: &str) -> Option<PathBuf> {
    path_from_env(primary_key).or_else(|| path_from_env(legacy_key))
}

fn path_from_env(key: &str) -> Option<PathBuf> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::dir_override;

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
    fn uses_primary_key_first() {
        let _primary = EnvGuard::set("CHANGQIAO_PATH_ENV_TEST_PRIMARY", Some("/tmp/primary"));
        let _legacy = EnvGuard::set("CHANGQIAO_PATH_ENV_TEST_LEGACY", Some("/tmp/legacy"));

        let result = dir_override(
            "CHANGQIAO_PATH_ENV_TEST_PRIMARY",
            "CHANGQIAO_PATH_ENV_TEST_LEGACY",
        );
        assert_eq!(result, Some(std::path::PathBuf::from("/tmp/primary")));
    }

    #[test]
    fn falls_back_to_legacy_key() {
        let _primary = EnvGuard::set("CHANGQIAO_PATH_ENV_TEST_PRIMARY", None);
        let _legacy = EnvGuard::set("CHANGQIAO_PATH_ENV_TEST_LEGACY", Some("/tmp/legacy"));

        let result = dir_override(
            "CHANGQIAO_PATH_ENV_TEST_PRIMARY",
            "CHANGQIAO_PATH_ENV_TEST_LEGACY",
        );
        assert_eq!(result, Some(std::path::PathBuf::from("/tmp/legacy")));
    }

    #[test]
    fn ignores_empty_values() {
        let _primary = EnvGuard::set("CHANGQIAO_PATH_ENV_TEST_PRIMARY", Some("   "));
        let _legacy = EnvGuard::set("CHANGQIAO_PATH_ENV_TEST_LEGACY", Some(""));

        let result = dir_override(
            "CHANGQIAO_PATH_ENV_TEST_PRIMARY",
            "CHANGQIAO_PATH_ENV_TEST_LEGACY",
        );
        assert!(result.is_none());
    }
}
