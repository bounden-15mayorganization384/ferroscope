use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme;

#[derive(Debug, Clone)]
pub struct CodeLine {
    pub content: String,
    pub highlight: bool,
}

#[derive(Debug, Clone)]
pub struct CodePanel {
    pub title: String,
    pub lines: Vec<CodeLine>,
}

impl CodePanel {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
        }
    }

    pub fn push_line(&mut self, content: impl Into<String>, highlight: bool) {
        self.lines.push(CodeLine {
            content: content.into(),
            highlight,
        });
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let ratatui_lines: Vec<Line> = self
            .lines
            .iter()
            .map(|l| {
                if l.highlight {
                    Line::from(Span::styled(
                        format!("▶ {}", l.content),
                        Style::default()
                            .fg(theme::SAFE_GREEN)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(format!("  {}", l.content), theme::dim_style()))
                }
            })
            .collect();

        let para = Paragraph::new(ratatui_lines).block(
            Block::default()
                .title(self.title.as_str())
                .borders(Borders::ALL),
        );
        frame.render_widget(para, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_new() {
        let p = CodePanel::new("Rust Code");
        assert_eq!(p.title, "Rust Code");
        assert!(p.lines.is_empty());
    }

    #[test]
    fn test_push_line() {
        let mut p = CodePanel::new("test");
        p.push_line("let x = 5;", false);
        p.push_line("let y = x;", true);
        assert_eq!(p.lines.len(), 2);
        assert!(!p.lines[0].highlight);
        assert!(p.lines[1].highlight);
    }

    #[test]
    fn test_clear() {
        let mut p = CodePanel::new("test");
        p.push_line("line", false);
        p.clear();
        assert!(p.lines.is_empty());
    }

    #[test]
    fn test_render_empty() {
        let p = CodePanel::new("empty");
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| p.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_with_lines() {
        let mut p = CodePanel::new("Code");
        p.push_line("fn main() {", false);
        p.push_line("    let x = 5;", true);
        p.push_line("}", false);
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| p.render(f, f.area())).unwrap();
    }
}
