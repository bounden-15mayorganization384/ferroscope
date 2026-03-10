pub mod footer;
pub mod header;
pub mod layout;
pub mod nav;
pub mod widgets;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{app::App, demos::DemoRegistry, theme};

pub fn draw(frame: &mut Frame, app: &App, registry: &DemoRegistry) {
    let area = frame.area();
    let layout = layout::app_layout(area);

    header::render_header(frame, layout.header, app);
    nav::render_nav(frame, layout.nav, app, registry);
    registry.render_current(app.current_demo, frame, layout.content);
    footer::render_footer(frame, layout.footer, app);

    if app.show_explanation {
        render_explanation_panel(frame, layout.content, app, registry);
    }

    if app.show_help {
        render_help_overlay(frame, area);
    }
}

fn render_explanation_panel(frame: &mut Frame, area: Rect, app: &App, registry: &DemoRegistry) {
    let (_, right) = layout::right_panel(35, area);
    let explanation = registry
        .explanation(app.current_demo)
        .unwrap_or("No explanation available.");

    let para = Paragraph::new(explanation)
        .block(
            Block::default()
                .title("Explanation")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORROW_YELLOW)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(Clear, right);
    frame.render_widget(para, right);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = layout::centered_rect(60, 70, area);
    let help_text = vec![
        Line::from(Span::styled("  Keyboard Shortcuts", Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().fg(theme::BORROW_YELLOW).add_modifier(Modifier::UNDERLINED))),
        Line::from("  <- / h      Previous demo"),
        Line::from("  -> / l      Next demo"),
        Line::from("  1-9, 0      Jump to demo 1-10"),
        Line::from("  a, b, c     Jump to demo 11, 12, 13"),
        Line::from("  d, e        Jump to demo 14, 15"),
        Line::from(""),
        Line::from(Span::styled("Controls", Style::default().fg(theme::BORROW_YELLOW).add_modifier(Modifier::UNDERLINED))),
        Line::from("  Space       Pause / Resume"),
        Line::from("  R           Reset current demo"),
        Line::from("  +           Increase speed"),
        Line::from("  -           Decrease speed"),
        Line::from(""),
        Line::from(Span::styled("Display", Style::default().fg(theme::BORROW_YELLOW).add_modifier(Modifier::UNDERLINED))),
        Line::from("  E           Toggle explanation panel"),
        Line::from("  ?           Toggle this help screen"),
        Line::from("  S           Screenshot (save to file)"),
        Line::from("  Q / Esc     Quit"),
        Line::from(""),
        Line::from(Span::styled("Difficulty  [B]=Beginner  [I]=Intermediate  [A]=Advanced",
            Style::default().fg(theme::TEXT_DIM))),
    ];

    let para = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::RUST_ORANGE)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(Clear, popup);
    frame.render_widget(para, popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demos::DemoRegistry;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_draw_basic() {
        let app = App::new(15);
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_draw_with_help() {
        let mut app = App::new(15);
        app.show_help = true;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_draw_with_explanation() {
        let mut app = App::new(15);
        app.show_explanation = true;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_draw_both_overlays() {
        let mut app = App::new(15);
        app.show_help = true;
        app.show_explanation = true;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }
}
