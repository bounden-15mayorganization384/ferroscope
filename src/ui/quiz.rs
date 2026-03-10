use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{app::App, theme, ui::layout::centered_rect};

/// Render the quiz overlay: a centered popup with the question, 4 options,
/// and a flash result if an answer was just given.
pub fn render_quiz(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    question: &str,
    options: &[&str; 4],
    correct: usize,
) {
    let popup = centered_rect(60, 50, area);
    frame.render_widget(Clear, popup);

    let border_color = match app.quiz_last_result {
        Some(true) => theme::SAFE_GREEN,
        Some(false) => theme::CRAB_RED,
        None => theme::BORROW_YELLOW,
    };

    let mut lines: Vec<Line> = Vec::new();

    // Score line
    let score_text = format!("Score: {}/{}", app.quiz_correct, app.quiz_total);
    lines.push(Line::from(Span::styled(
        score_text,
        Style::default()
            .fg(theme::BORROW_YELLOW)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Question
    lines.push(Line::from(Span::styled(
        question,
        Style::default()
            .fg(theme::TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Answer options
    let option_keys = ["1", "2", "3", "4"];
    for (i, opt) in options.iter().enumerate() {
        let (key_color, opt_color) = match app.quiz_last_result {
            Some(true) if i == correct => (theme::SAFE_GREEN, theme::SAFE_GREEN),
            Some(false) if i == correct => (theme::SAFE_GREEN, theme::SAFE_GREEN),
            _ => (theme::RUST_ORANGE, theme::TEXT_PRIMARY),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  [{}] ", option_keys[i]),
                Style::default().fg(key_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(*opt, Style::default().fg(opt_color)),
        ]));
    }

    lines.push(Line::from(""));

    // Result flash
    match app.quiz_last_result {
        Some(true) => {
            lines.push(Line::from(Span::styled(
                "  ✓  Correct!",
                Style::default()
                    .fg(theme::SAFE_GREEN)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        Some(false) => {
            lines.push(Line::from(Span::styled(
                "  ✗  Wrong — try again",
                Style::default()
                    .fg(theme::CRAB_RED)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        None => {
            lines.push(Line::from(Span::styled(
                "  Press 1-4 to answer  │  T to close",
                Style::default().fg(theme::TEXT_DIM),
            )));
        }
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Quiz ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(para, popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn make_opts() -> [&'static str; 4] {
        ["Option A", "Option B", "Option C", "Option D"]
    }

    #[test]
    fn test_render_quiz_no_answer() {
        let app = App::new(16);
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_quiz(f, f.area(), &app, "What is ownership?", &make_opts(), 0))
            .unwrap();
    }

    #[test]
    fn test_render_quiz_correct_answer() {
        let mut app = App::new(16);
        app.quiz_last_result = Some(true);
        app.quiz_correct = 1;
        app.quiz_total = 1;
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_quiz(f, f.area(), &app, "What is ownership?", &make_opts(), 0))
            .unwrap();
    }

    #[test]
    fn test_render_quiz_wrong_answer() {
        let mut app = App::new(16);
        app.quiz_last_result = Some(false);
        app.quiz_total = 1;
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_quiz(f, f.area(), &app, "What is ownership?", &make_opts(), 0))
            .unwrap();
    }

    #[test]
    fn test_render_quiz_small_area() {
        let app = App::new(16);
        let backend = TestBackend::new(40, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_quiz(f, f.area(), &app, "Short question?", &make_opts(), 2))
            .unwrap();
    }
}
