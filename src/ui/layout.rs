use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, Copy)]
pub struct AppLayout {
    pub header: Rect,
    pub nav: Rect,
    pub content: Rect,
    pub footer: Rect,
}

pub fn app_layout(area: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    AppLayout {
        header: chunks[0],
        nav: chunks[1],
        content: chunks[2],
        footer: chunks[3],
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let px = percent_x.min(100);
    let py = percent_y.min(100);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - py) / 2),
            Constraint::Percentage(py),
            Constraint::Percentage((100 - py) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - px) / 2),
            Constraint::Percentage(px),
            Constraint::Percentage((100 - px) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Returns (left_area, right_area) where right_area is `percent` wide.
pub fn right_panel(percent: u16, r: Rect) -> (Rect, Rect) {
    let p = percent.min(100);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100 - p),
            Constraint::Percentage(p),
        ])
        .split(r);
    (chunks[0], chunks[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: u16, h: u16) -> Rect {
        Rect::new(0, 0, w, h)
    }

    #[test]
    fn test_app_layout_normal() {
        let layout = app_layout(rect(80, 24));
        assert_eq!(layout.header.height, 3);
        assert_eq!(layout.nav.height, 3);
        assert_eq!(layout.footer.height, 1);
        // content takes remaining = 24 - 3 - 3 - 1 = 17
        assert_eq!(layout.content.height, 17);
        // all same width
        assert_eq!(layout.header.width, 80);
        assert_eq!(layout.nav.width, 80);
        assert_eq!(layout.content.width, 80);
        assert_eq!(layout.footer.width, 80);
    }

    #[test]
    fn test_app_layout_small() {
        let layout = app_layout(rect(40, 10));
        // 10 - 3 - 3 - 1 = 3
        assert_eq!(layout.content.height, 3);
    }

    #[test]
    fn test_app_layout_very_small() {
        // Should not panic even with tiny area
        let _layout = app_layout(rect(10, 4));
    }

    #[test]
    fn test_centered_rect() {
        let area = rect(100, 50);
        let centered = centered_rect(60, 40, area);
        // Should be roughly 60% wide, 40% tall
        assert!(centered.width <= 100);
        assert!(centered.height <= 50);
    }

    #[test]
    fn test_centered_rect_clamps_100() {
        let area = rect(80, 24);
        let centered = centered_rect(120, 120, area); // over 100%
        assert!(centered.width <= 80);
        assert!(centered.height <= 24);
    }

    #[test]
    fn test_right_panel() {
        let area = rect(100, 20);
        let (left, right) = right_panel(30, area);
        assert_eq!(left.width + right.width, 100);
        assert_eq!(right.width, 30);
        assert_eq!(left.width, 70);
    }

    #[test]
    fn test_right_panel_clamps_100() {
        let area = rect(100, 20);
        let (left, right) = right_panel(150, area);
        assert_eq!(right.width, 100);
        assert_eq!(left.width, 0);
    }
}
