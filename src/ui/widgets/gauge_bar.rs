use std::f64::consts::PI;

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
    /// Current tick count — used to drive a subtle breathing pulse effect.
    pub tick: u64,
}

impl GaugeBar {
    pub fn new(label: impl Into<String>, value: f64, color: Color) -> Self {
        Self {
            label: label.into(),
            value,
            color,
            tick: 0,
        }
    }

    pub fn clamped_value(&self) -> f64 {
        self.value.clamp(0.0, 1.0)
    }

    /// Returns the display ratio with a gentle breathing pulse applied.
    pub fn pulsed_value(&self) -> f64 {
        let pulse = (self.tick as f64 * PI / 30.0).sin() * 0.03;
        (self.clamped_value() + pulse).clamp(0.0, 1.0)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let ratio = self.pulsed_value();
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
    fn test_pulsed_value_in_bounds() {
        // pulsed value must always stay in [0, 1]
        for tick in [0u64, 15, 30, 45, 60, 90, 120] {
            let mut g = GaugeBar::new("test", 0.5, Color::Green);
            g.tick = tick;
            let v = g.pulsed_value();
            assert!((0.0..=1.0).contains(&v), "tick={tick} pulsed_value={v}");
        }
    }

    #[test]
    fn test_pulsed_value_zero_base() {
        let mut g = GaugeBar::new("test", 0.0, Color::Green);
        g.tick = 0;
        // At tick=0, sin(0)=0 → pulse=0 → clamped value stays 0
        assert_eq!(g.pulsed_value(), 0.0);
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

    #[test]
    fn test_render_with_tick() {
        let mut g = GaugeBar::new("CPU", 0.5, Color::Cyan);
        g.tick = 47;
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| g.render(f, f.area())).unwrap();
    }
}
