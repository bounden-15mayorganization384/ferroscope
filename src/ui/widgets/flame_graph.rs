use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct FlameGraph {
    pub frames: Vec<(String, f64)>,
    pub color: Color,
}

impl FlameGraph {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            color: Color::Yellow,
        }
    }

    pub fn push_frame(&mut self, label: impl Into<String>, proportion: f64) {
        let p = proportion.clamp(0.0, 1.0);
        self.frames.push((label.into(), p));
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("Flame Graph").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.frames.is_empty() || inner.height == 0 || inner.width == 0 {
            return;
        }

        let constraints: Vec<Constraint> =
            self.frames.iter().map(|_| Constraint::Length(1)).collect();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        for (i, (label, proportion)) in self.frames.iter().enumerate() {
            if i >= rows.len() {
                break;
            }
            let bar_width = (rows[i].width as f64 * proportion) as usize;
            let bar_width = bar_width.min(rows[i].width as usize);
            let bar: String = "█".repeat(bar_width);
            let padding = rows[i].width as usize - bar_width;
            let pad: String = " ".repeat(padding);
            let pct = (proportion * 100.0) as u8;
            let text = format!("{:<12} {}{} {:3}%", label, bar, pad, pct);
            let line = Line::from(Span::styled(text, Style::default().fg(self.color)));
            frame.render_widget(Paragraph::new(line), rows[i]);
        }
    }
}

impl Default for FlameGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_new() {
        let fg = FlameGraph::new();
        assert!(fg.frames.is_empty());
    }

    #[test]
    fn test_push_frame() {
        let mut fg = FlameGraph::new();
        fg.push_frame("sort()", 0.4);
        fg.push_frame("alloc()", 0.2);
        assert_eq!(fg.frames.len(), 2);
        assert!((fg.frames[0].1 - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_push_frame_clamps() {
        let mut fg = FlameGraph::new();
        fg.push_frame("over", 1.5);
        fg.push_frame("under", -0.5);
        assert!((fg.frames[0].1 - 1.0).abs() < 1e-6);
        assert!((fg.frames[1].1 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut fg = FlameGraph::new();
        fg.push_frame("a", 0.5);
        fg.clear();
        assert!(fg.frames.is_empty());
    }

    #[test]
    fn test_render_empty() {
        let fg = FlameGraph::new();
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| fg.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_with_frames() {
        let mut fg = FlameGraph::new();
        fg.push_frame("sort()", 0.5);
        fg.push_frame("alloc()", 0.3);
        fg.push_frame("memcpy()", 0.2);
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| fg.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_default() {
        let fg = FlameGraph::default();
        assert!(fg.frames.is_empty());
    }
}
