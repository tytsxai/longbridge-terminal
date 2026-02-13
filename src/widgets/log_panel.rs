use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 判断文件名是否为 changqiao/longbridge 日志文件
fn is_log_file_name(name: &str) -> bool {
    (name.starts_with("changqiao") || name.starts_with("longbridge"))
        && Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("log"))
}

/// Get the path to the latest log file
fn get_latest_log_file() -> Option<PathBuf> {
    let log_dir = crate::logger::active_log_dir();

    // Find all log files in the directory
    let mut log_files: Vec<PathBuf> = fs::read_dir(&log_dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(is_log_file_name)
        })
        .collect();

    // Sort by modification time (newest first)
    log_files.sort_by(|a, b| {
        let time_a = fs::metadata(a).and_then(|m| m.modified()).ok();
        let time_b = fs::metadata(b).and_then(|m| m.modified()).ok();

        match (time_a, time_b) {
            (Some(ta), Some(tb)) => tb.cmp(&ta), // Reverse order (newest first)
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    log_files.into_iter().next()
}

/// Read the last N lines from the log file
fn read_last_lines(path: &Path, count: usize) -> Vec<String> {
    let max_lines = count.max(1);
    let Ok(file) = std::fs::File::open(path) else {
        return vec![];
    };
    let reader = BufReader::new(file);
    let mut tail = VecDeque::with_capacity(max_lines);

    for line in reader.lines().map_while(Result::ok) {
        if tail.len() == max_lines {
            tail.pop_front();
        }
        tail.push_back(line);
    }

    tail.into_iter().collect()
}

/// Log panel widget
pub struct LogPanel {
    lines: Vec<String>,
    visible: bool,
    last_log_file: Option<PathBuf>,
    last_modified: Option<SystemTime>,
    last_size: u64,
}

impl LogPanel {
    /// Create a new log panel
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            visible: false,
            last_log_file: None,
            last_modified: None,
            last_size: 0,
        }
    }

    /// Refresh log content from file
    pub fn refresh(&mut self) {
        if let Some(log_file) = get_latest_log_file() {
            let metadata = fs::metadata(&log_file).ok();
            let modified = metadata.as_ref().and_then(|m| m.modified().ok());
            let size = metadata.as_ref().map_or(0, std::fs::Metadata::len);
            let unchanged = self.last_log_file.as_ref() == Some(&log_file)
                && self.last_modified == modified
                && self.last_size == size;

            if unchanged {
                return;
            }

            self.lines = read_last_lines(&log_file, 100);
            self.last_log_file = Some(log_file);
            self.last_modified = modified;
            self.last_size = size;
            return;
        }

        self.lines.clear();
        self.last_log_file = None;
        self.last_modified = None;
        self.last_size = 0;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if visible {
            self.refresh();
        }
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.set_visible(!self.visible);
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the log panel as a floating overlay
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Refresh log content every time we render (auto-refresh)
        self.refresh();

        // Clear the area behind the log panel to block background content
        frame.render_widget(Clear, area);

        // Render log panel with background
        let block = Block::default()
            .title(format!(" {} ", t!("Keyboard.Console")))
            .bg(Color::Black)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(Color::Black));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Prepare log lines for display
        let display_lines: Vec<Line> = self
            .lines
            .iter()
            .rev()
            .take(inner_area.height as usize)
            .rev()
            .map(|line| {
                // Colorize log levels
                if line.contains("ERROR") {
                    Line::from(Span::styled(line.clone(), Style::default().fg(Color::Red)))
                } else if line.contains("WARN") {
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::Yellow),
                    ))
                } else if line.contains("INFO") {
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::Green),
                    ))
                } else if line.contains("DEBUG") {
                    Line::from(Span::styled(line.clone(), Style::default().fg(Color::Cyan)))
                } else {
                    Line::from(line.clone())
                }
            })
            .collect();

        let paragraph = Paragraph::new(display_lines).style(Style::default().bg(Color::Black));
        frame.render_widget(paragraph, inner_area);
    }
}

impl Default for LogPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{is_log_file_name, read_last_lines};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempFileGuard {
        path: PathBuf,
    }

    impl TempFileGuard {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir().join(format!("changqiao-log-panel-{unique}.log"));
            Self { path }
        }
    }

    impl Drop for TempFileGuard {
        fn drop(&mut self) {
            _ = std::fs::remove_file(&self.path);
        }
    }

    #[test]
    fn reads_last_lines_from_log_file() {
        let guard = TempFileGuard::new();
        std::fs::write(&guard.path, "line1\nline2\nline3\nline4\nline5\n")
            .expect("failed to write temp log");

        let lines = read_last_lines(&guard.path, 2);
        assert_eq!(lines, vec!["line4".to_string(), "line5".to_string()]);
    }

    #[test]
    fn recognizes_expected_log_file_name() {
        assert!(is_log_file_name("changqiao.log"));
        assert!(is_log_file_name("longbridge.2026-02-13.log"));
        assert!(!is_log_file_name("changqiao.txt"));
        assert!(!is_log_file_name("random.log"));
    }
}
