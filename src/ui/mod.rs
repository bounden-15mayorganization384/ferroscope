pub mod footer;
pub mod header;
pub mod layout;
pub mod nav;
pub mod quiz;
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

    // Demo content with optional transition wipe
    if app.transition_frames > 0 {
        render_transition_wipe(frame, layout.content, app, registry);
    } else {
        registry.render_current(app.current_demo, frame, layout.content);
    }

    footer::render_footer(frame, layout.footer, app, registry);

    if app.show_explanation {
        render_explanation_panel(frame, layout.content, app, registry);
    }

    if app.quiz_active {
        if let Some((question, options, correct)) = registry.quiz_current(app.current_demo) {
            quiz::render_quiz(frame, area, app, question, &options, correct);
        }
    }

    if app.show_help {
        render_help_overlay(frame, area);
    }

    if app.has_achievement_flash() {
        if let Some((name, _)) = app.achievement_flash {
            render_achievement_overlay(frame, area, name);
        }
    }

    // Particle burst — written directly to the frame buffer last so they appear on top
    render_particles(frame, app);
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
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((app.explanation_scroll, 0));

    frame.render_widget(Clear, right);
    frame.render_widget(para, right);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = layout::centered_rect(60, 75, area);
    let help_text = vec![
        Line::from(Span::styled(
            "  Keyboard Shortcuts",
            Style::default()
                .fg(theme::RUST_ORANGE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(theme::BORROW_YELLOW)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  <- / h      Previous demo"),
        Line::from("  -> / l      Next demo"),
        Line::from("  1-9, 0      Jump to demo 1-10"),
        Line::from("  a, b, c     Jump to demo 11, 12, 13"),
        Line::from("  d, f, g     Jump to demo 14, 15, 16"),
        Line::from(""),
        Line::from(Span::styled(
            "Controls",
            Style::default()
                .fg(theme::BORROW_YELLOW)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  Space       Pause / Resume"),
        Line::from("  R           Reset current demo"),
        Line::from("  V           Toggle vs-mode (Rust vs C++)"),
        Line::from("  N / P       Step forward / back (when available)"),
        Line::from("  T           Quiz mode (when available)"),
        Line::from("  +           Increase speed"),
        Line::from("  -           Decrease speed"),
        Line::from(""),
        Line::from(Span::styled(
            "Display",
            Style::default()
                .fg(theme::BORROW_YELLOW)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  E           Toggle explanation panel"),
        Line::from("  ?           Toggle this help screen"),
        Line::from("  S           Screenshot (save to file)"),
        Line::from("  Q / Esc     Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Difficulty  [B]=Beginner  [I]=Intermediate  [A]=Advanced",
            Style::default().fg(theme::TEXT_DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Tip: Try the Konami code for a surprise...",
            Style::default()
                .fg(theme::CRAB_RED)
                .add_modifier(Modifier::DIM),
        )),
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

fn render_achievement_overlay(frame: &mut Frame, area: Rect, achievement_name: &str) {
    // Small banner at the bottom-right corner
    let width = 34u16.min(area.width);
    let height = 3u16;
    let x = area.width.saturating_sub(width + 1);
    let y = area.height.saturating_sub(height + 2);
    let popup = Rect::new(x, y, width, height);

    let lines = vec![
        Line::from(Span::styled(
            " 🏆 Achievement Unlocked!",
            Style::default()
                .fg(theme::BORROW_YELLOW)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  ★  {}", achievement_name),
            Style::default()
                .fg(theme::SAFE_GREEN)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORROW_YELLOW)),
    );

    frame.render_widget(Clear, popup);
    frame.render_widget(para, popup);
}

/// Horizontal wipe transition: slides the new demo in from the left.
fn render_transition_wipe(frame: &mut Frame, area: Rect, app: &App, registry: &DemoRegistry) {
    // Render the incoming (current) demo fully
    registry.render_current(app.current_demo, frame, area);

    // Cover the not-yet-revealed right portion with a blank curtain
    if area.width == 0 || app.transition_frames == 0 {
        return;
    }
    let revealed = area
        .width
        .saturating_mul(10u16.saturating_sub(app.transition_frames as u16))
        / 10;
    let curtain_x = area.x + revealed;
    let curtain_w = area.width.saturating_sub(revealed);
    if curtain_w > 0 {
        let curtain = Rect::new(curtain_x, area.y, curtain_w, area.height);
        frame.render_widget(Clear, curtain);
        // Draw a thin glowing edge line at the reveal boundary
        if revealed > 0 {
            let edge = Rect::new(curtain_x.saturating_sub(1), area.y, 1, area.height);
            let edge_lines: Vec<Line> = (0..area.height)
                .map(|_| Line::from(Span::styled("▌", Style::default().fg(theme::RUST_ORANGE))))
                .collect();
            frame.render_widget(Paragraph::new(edge_lines), edge);
        }
    }
}

/// Render particle bursts directly into the frame buffer.
fn render_particles(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let buf = frame.buffer_mut();
    for p in &app.particles {
        let x = p.x as u16;
        let y = p.y as u16;
        if x < area.width && y < area.height {
            let cell = &mut buf[(x, y)];
            cell.set_symbol(&p.ch.to_string());
            cell.set_style(Style::default().fg(p.color));
        }
    }
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
    fn test_draw_with_explanation_scrolled() {
        let mut app = App::new(15);
        app.show_explanation = true;
        app.explanation_scroll = 3;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_draw_explanation_scroll_zero_unchanged() {
        let mut app = App::new(15);
        app.show_explanation = true;
        app.explanation_scroll = 0;
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

    #[test]
    fn test_draw_with_achievement_flash() {
        let mut app = App::new(15);
        app.achievement_flash = Some(("Explorer", 9999));
        app.tick_count = 100; // within flash window
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_draw_konami_active() {
        let mut app = App::new(15);
        app.konami_active = true;
        app.konami_countdown = 90;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_draw_all_visited() {
        let mut app = App::new(15);
        for i in 0..15 {
            app.visit(i);
        }
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &app, &registry)).unwrap();
    }
}
