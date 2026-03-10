use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app::App, theme};

const DEMO_COUNT: usize = 15;

/// Build a text-based progress bar with `width` characters.
fn progress_bar(visited: usize, total: usize, width: usize) -> String {
    if total == 0 || width == 0 {
        return "░".repeat(width);
    }
    let filled = (visited.min(total) * width / total).min(width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

pub fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if app.konami_active {
            Style::default().fg(theme::CRAB_RED)
        } else {
            Style::default().fg(theme::RUST_ORANGE)
        });

    // Render the block first, then work inside the inner area
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Row 0: crab + title + version/speed/tick
            Constraint::Length(1), // Row 1: progress bar
            Constraint::Min(0),    // Row 2: konami / extra
        ])
        .split(inner);

    // ── Row 0: Title ──────────────────────────────────────────────────────────
    let crab = App::crab_frame_str(app.crab_frame);

    let paused_span = if app.paused {
        Span::styled("  ⏸ PAUSED", Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    let title_line = Line::from(vec![
        Span::styled(crab, Style::default().fg(theme::CRAB_RED)),
        Span::styled(" 🦀 ", Style::default()),
        Span::styled("FERROSCOPE", Style::default()
            .fg(theme::RUST_ORANGE)
            .add_modifier(Modifier::BOLD)),
        Span::styled("  │  Rust Capabilities Explorer", theme::dim_style()),
        Span::styled(
            format!("  │  v0.1.0  │  Speed:{}x  │  tick:{}", app.speed, app.tick_count),
            theme::dim_style(),
        ),
        paused_span,
    ]);
    frame.render_widget(Paragraph::new(title_line), inner_chunks[0]);

    // ── Row 1: Explored progress bar ─────────────────────────────────────────
    if inner_chunks[1].height > 0 {
        let visited = app.visited_count().min(DEMO_COUNT);
        let bar_width = (inner.width.saturating_sub(28)) as usize;
        let bar = progress_bar(visited, DEMO_COUNT, bar_width);

        let bar_color = if visited == DEMO_COUNT {
            theme::SAFE_GREEN
        } else if visited >= 10 {
            theme::BORROW_YELLOW
        } else {
            Color::DarkGray
        };

        let progress_line = Line::from(vec![
            Span::styled("Explored: ", theme::dim_style()),
            Span::styled(bar, Style::default().fg(bar_color)),
            Span::styled(
                format!("  {}/{} demos", visited, DEMO_COUNT),
                Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(progress_line), inner_chunks[1]);
    }

    // ── Row 2: Konami CRAB MODE ───────────────────────────────────────────────
    if inner_chunks[2].height > 0 && app.konami_active {
        let pulse = if (app.tick_count / 4) % 2 == 0 {
            "🦀🦀🦀  ★ CRAB MODE ACTIVATED ★  🦀🦀🦀"
        } else {
            "   ★ CRAB MODE ACTIVATED ★   "
        };
        let konami_line = Line::from(Span::styled(
            pulse,
            Style::default().fg(theme::CRAB_RED).add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(Paragraph::new(konami_line), inner_chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn make_app(paused: bool) -> App {
        let mut app = App::new(15);
        app.paused = paused;
        app
    }

    #[test]
    fn test_render_not_paused() {
        let app = make_app(false);
        let backend = TestBackend::new(120, 7);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_render_paused() {
        let app = make_app(true);
        let backend = TestBackend::new(120, 7);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_render_konami_active() {
        let mut app = make_app(false);
        app.konami_active = true;
        app.konami_countdown = 100;
        let backend = TestBackend::new(120, 7);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_render_all_demos_visited() {
        let mut app = make_app(false);
        for i in 0..15 { app.visit(i); }
        let backend = TestBackend::new(120, 7);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_render_minimal_area() {
        let app = make_app(false);
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_header(f, f.area(), &app)).unwrap();
    }

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0, 15, 20);
        assert_eq!(bar.chars().count(), 20);
        assert!(bar.chars().all(|c| c == '░'));
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(15, 15, 20);
        assert!(bar.chars().all(|c| c == '█'));
    }

    #[test]
    fn test_progress_bar_half() {
        let bar = progress_bar(7, 15, 20);
        let filled = bar.chars().filter(|&c| c == '█').count();
        assert!(filled > 0 && filled < 20);
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let bar = progress_bar(5, 0, 10);
        assert_eq!(bar.chars().count(), 10);
    }

    #[test]
    fn test_crab_frames_all_cycle() {
        for i in 0..8u8 {
            let s = App::crab_frame_str(i);
            assert!(!s.is_empty());
        }
    }
}
