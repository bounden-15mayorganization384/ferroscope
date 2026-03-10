use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Sparkline},
    Frame,
};

#[derive(Debug, Clone)]
pub struct SparklineExt {
    pub data: Vec<u64>,
    pub label: String,
    pub max: u64,
    pub color: Color,
}

impl SparklineExt {
    pub fn new(label: impl Into<String>, max: u64, color: Color) -> Self {
        Self {
            data: Vec::new(),
            label: label.into(),
            max,
            color,
        }
    }

    pub fn push(&mut self, val: u64) {
        self.data.push(val);
        // Keep last 64 values
        if self.data.len() > 64 {
            self.data.remove(0);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(self.label.as_str())
                    .borders(Borders::ALL),
            )
            .data(&self.data)
            .max(if self.max == 0 { 1 } else { self.max })
            .style(Style::default().fg(self.color));
        frame.render_widget(sparkline, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_new() {
        let s = SparklineExt::new("CPU", 100, Color::Cyan);
        assert_eq!(s.label, "CPU");
        assert_eq!(s.max, 100);
        assert!(s.data.is_empty());
    }

    #[test]
    fn test_push_and_cap() {
        let mut s = SparklineExt::new("test", 100, Color::Cyan);
        for i in 0..70 {
            s.push(i);
        }
        assert_eq!(s.data.len(), 64);
        // Last 64 values: 6..=69
        assert_eq!(s.data[0], 6);
        assert_eq!(s.data[63], 69);
    }

    #[test]
    fn test_render_empty() {
        let s = SparklineExt::new("test", 100, Color::Cyan);
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| s.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_with_data() {
        let mut s = SparklineExt::new("test", 100, Color::Cyan);
        s.push(10);
        s.push(50);
        s.push(90);
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| s.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_max_zero() {
        let s = SparklineExt::new("test", 0, Color::Cyan);
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| s.render(f, f.area())).unwrap();
    }
}
