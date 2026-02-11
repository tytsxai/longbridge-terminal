use std::sync::atomic::{AtomicU8, Ordering};

use bevy_ecs::{prelude::Component, system::Resource};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Resource, Component)]
pub struct Loading {
    index: AtomicU8,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LoadingWidget {
    index: u8,
}

impl From<&Loading> for LoadingWidget {
    fn from(loading: &Loading) -> Self {
        Self {
            index: loading.index.fetch_add(1, Ordering::Acquire),
        }
    }
}

impl Widget for LoadingWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let spinner = match self.index % 4 {
            0 => "◰",
            1 => "◳",
            2 => "◲",
            _ => "◱",
        };
        let text = format!("{spinner} {}", t!("Loading.General"));
        let area = crate::ui::rect::centered(16, 1, area);
        Paragraph::new(text).render(area, buf);
    }
}
