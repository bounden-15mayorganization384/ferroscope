use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app::App, theme};

pub fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let paused_indicator = if app.paused {
        Span::styled(" ⏸ PAUSED", Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("", Style::default())
    };

    let title = Line::from(vec![
        Span::styled("🦀 ", Style::default()),
        Span::styled("FERROSCOPE", Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled("  │  Rust Capabilities Explorer  │  v0.1.0", theme::dim_style()),
        Span::styled(format!("  │  tick:{}", app.tick_count), theme::dim_style()),
        paused_indicator,
    ]);

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme::RUST_ORANGE)));

    frame.render_widget(header, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn make_app(paused: bool) -> App {
        let mut app = App::new(12);
        app.paused = paused;
        app
    }

    #[test]
    fn test_render_not_paused() {
        let app = make_app(false);
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_render_paused() {
        let app = make_app(true);
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }
}
