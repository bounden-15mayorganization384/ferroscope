use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{app::App, theme};

pub fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let speed_text = format!("Speed:{}x", app.speed);

    let pause_hint = if app.paused {
        Span::styled(" ⏸ PAUSED  ", Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("", Style::default())
    };

    let line = Line::from(vec![
        pause_hint,
        Span::styled("← → Navigate", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("Space: Pause", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("R: Reset", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled(speed_text, theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("+/-: Speed", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("E: Explain", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("?: Help", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("Q: Quit", Style::default().fg(theme::CRAB_RED)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_running() {
        let app = App::new(12);
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_footer(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_render_paused() {
        let mut app = App::new(12);
        app.paused = true;
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_footer(f, f.area(), &app)).unwrap();
    }
}
