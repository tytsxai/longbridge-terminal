use std::cmp::Ordering;

pub trait Sign {
    fn positive(&self) -> bool;
    fn negative(&self) -> bool;
    fn zero(&self) -> bool;
    fn sign(&self) -> Ordering;
}

impl Sign for str {
    fn positive(&self) -> bool {
        !(self.negative() || self.zero())
    }

    fn negative(&self) -> bool {
        self.starts_with('-')
    }

    fn zero(&self) -> bool {
        self.chars().all(|c| matches!(c, '0' | '.' | '+' | '-'))
    }

    fn sign(&self) -> Ordering {
        if self.negative() {
            Ordering::Less
        } else if self.zero() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Sign for rust_decimal::Decimal {
    fn positive(&self) -> bool {
        self.is_sign_positive() && !self.is_zero()
    }

    fn negative(&self) -> bool {
        self.is_sign_negative()
    }

    fn zero(&self) -> bool {
        self.is_zero()
    }

    fn sign(&self) -> Ordering {
        if self.is_sign_negative() {
            Ordering::Less
        } else if self.is_zero() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

/// Format volume to short format
/// Example: 1234567 → 1.23M (en) / 123.46万 (zh-CN)
pub fn format_volume(volume: u64) -> String {
    if volume == 0 {
        return "--".to_string();
    }

    #[allow(clippy::cast_precision_loss)]
    let volume_f = volume as f64;
    let locale = rust_i18n::locale();

    if locale.starts_with("zh") {
        let (wan, yi, wan_yi) = match locale.as_str() {
            "zh-HK" | "zh-TW" => ("萬", "億", "萬億"),
            _ => ("万", "亿", "万亿"),
        };

        if volume >= 1_000_000_000_000 {
            format!("{:.2}{wan_yi}", volume_f / 1_000_000_000_000.0)
        } else if volume >= 100_000_000 {
            format!("{:.2}{yi}", volume_f / 100_000_000.0)
        } else if volume >= 10_000 {
            format!("{:.2}{wan}", volume_f / 10_000.0)
        } else {
            volume.to_string()
        }
    } else if volume >= 1_000_000_000 {
        format!("{:.2}B", volume_f / 1_000_000_000.0)
    } else if volume >= 1_000_000 {
        format!("{:.2}M", volume_f / 1_000_000.0)
    } else if volume >= 1_000 {
        format!("{:.2}K", volume_f / 1_000.0)
    } else {
        volume.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::format_volume;

    #[test]
    fn format_volume_in_english() {
        let _lock = crate::helper::TEST_LOCALE_LOCK.lock().expect("poison");
        rust_i18n::set_locale("en");
        assert_eq!(format_volume(0), "--");
        assert_eq!(format_volume(532), "532");
        assert_eq!(format_volume(12_300), "12.30K");
        assert_eq!(format_volume(3_456_700), "3.46M");
        assert_eq!(format_volume(9_876_543_210), "9.88B");
    }

    #[test]
    fn format_volume_in_chinese() {
        let _lock = crate::helper::TEST_LOCALE_LOCK.lock().expect("poison");
        rust_i18n::set_locale("zh-CN");
        assert_eq!(format_volume(5_320), "5320");
        assert_eq!(format_volume(12_300), "1.23万");
        assert_eq!(format_volume(345_670_000), "3.46亿");
        assert_eq!(format_volume(9_876_543_210_000), "9.88万亿");
    }

    #[test]
    fn format_volume_in_hk_chinese() {
        let _lock = crate::helper::TEST_LOCALE_LOCK.lock().expect("poison");
        rust_i18n::set_locale("zh-HK");
        assert_eq!(format_volume(12_300), "1.23萬");
        assert_eq!(format_volume(345_670_000), "3.46億");
        assert_eq!(format_volume(9_876_543_210_000), "9.88萬億");
    }

    #[test]
    fn format_volume_boundary_values() {
        let _lock = crate::helper::TEST_LOCALE_LOCK.lock().expect("poison");
        rust_i18n::set_locale("en");
        assert_eq!(format_volume(999), "999");
        assert_eq!(format_volume(1_000), "1.00K");
        assert_eq!(format_volume(999_999), "1000.00K");
        assert_eq!(format_volume(1_000_000), "1.00M");

        rust_i18n::set_locale("zh-CN");
        assert_eq!(format_volume(9_999), "9999");
        assert_eq!(format_volume(10_000), "1.00万");
        assert_eq!(format_volume(99_999_999), "10000.00万");
        assert_eq!(format_volume(100_000_000), "1.00亿");
    }
}
