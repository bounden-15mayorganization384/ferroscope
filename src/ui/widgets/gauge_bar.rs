use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

#[derive(Debug, Clone)]
pub struct GaugeBar {
    pub label: String,
    pub value: f64,
    pub color: Color,
}

impl GaugeBar {
    pub fn new(label: impl Into<String>, value: f64, color: Color) -> Self {
        Self {
            label: label.into(),
            value,
            color,
        }
    }

    pub fn clamped_value(&self) -> f64 {
        self.value.clamp(0.0, 1.0)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let ratio = self.clamped_value();
        let pct = (ratio * 100.0) as u16;
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(self.label.as_str())
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(self.color))
            .percent(pct);
        frame.render_widget(gauge, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_clamped_value_normal() {
        let g = GaugeBar::new("test", 0.5, Color::Green);
        assert_eq!(g.clamped_value(), 0.5);
    }

    #[test]
    fn test_clamped_value_below_zero() {
        let g = GaugeBar::new("test", -1.0, Color::Green);
        assert_eq!(g.clamped_value(), 0.0);
    }

    #[test]
    fn test_clamped_value_above_one() {
        let g = GaugeBar::new("test", 2.0, Color::Green);
        assert_eq!(g.clamped_value(), 1.0);
    }

    #[test]
    fn test_clamped_value_at_bounds() {
        let g0 = GaugeBar::new("test", 0.0, Color::Green);
        assert_eq!(g0.clamped_value(), 0.0);
        let g1 = GaugeBar::new("test", 1.0, Color::Green);
        assert_eq!(g1.clamped_value(), 1.0);
    }

    #[test]
    fn test_render() {
        let g = GaugeBar::new("Memory", 0.75, Color::Blue);
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| g.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_zero() {
        let g = GaugeBar::new("CPU", 0.0, Color::Red);
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| g.render(f, f.area())).unwrap();
    }
}
