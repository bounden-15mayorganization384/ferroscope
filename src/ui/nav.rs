use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{app::App, demos::DemoRegistry, theme};

/// Keyboard shortcut characters for each demo slot (up to 16 demos).
const KEYS: &[&str] = &[
    "1","2","3","4","5","6","7","8","9","0","a","b","c","d","f",
];

/// Difficulty level of each demo by index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub fn badge(&self) -> &'static str {
        match self {
            Difficulty::Beginner     => "B",
            Difficulty::Intermediate => "I",
            Difficulty::Advanced     => "A",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Difficulty::Beginner     => theme::SAFE_GREEN,
            Difficulty::Intermediate => theme::BORROW_YELLOW,
            Difficulty::Advanced     => theme::CRAB_RED,
        }
    }
}

/// Returns the difficulty level for a given demo index (0-based).
pub fn demo_difficulty(idx: usize) -> Difficulty {
    match idx {
        0  => Difficulty::Beginner,      // Ownership & Borrowing
        1  => Difficulty::Beginner,      // Memory Management
        2  => Difficulty::Intermediate,  // Zero-Cost Abstractions
        3  => Difficulty::Intermediate,  // Fearless Concurrency
        4  => Difficulty::Intermediate,  // Async / Await
        5  => Difficulty::Beginner,      // Performance Benchmarks
        6  => Difficulty::Intermediate,  // Type System
        7  => Difficulty::Beginner,      // Error Handling
        8  => Difficulty::Intermediate,  // Lifetimes
        9  => Difficulty::Advanced,      // Unsafe Rust
        10 => Difficulty::Advanced,      // WebAssembly
        11 => Difficulty::Beginner,      // System Metrics
        12 => Difficulty::Intermediate,  // Compile-Time Guarantees
        13 => Difficulty::Beginner,      // Cargo Ecosystem
        14 => Difficulty::Advanced,      // Embedded / no_std
        _  => Difficulty::Intermediate,
    }
}

pub fn render_nav(frame: &mut Frame, area: Rect, app: &App, registry: &DemoRegistry) {
    let titles: Vec<Line> = (0..registry.len())
        .map(|i| {
            let key = KEYS.get(i).copied().unwrap_or("?");
            let name = registry.name(i).unwrap_or("???");
            let diff = demo_difficulty(i);
            let visited = i < 32 && (app.visited_demos & (1u32 << i)) != 0;
            let (visit_sym, visit_style) = if visited {
                ("✓", Style::default().fg(theme::SAFE_GREEN))
            } else {
                ("·", theme::dim_style())
            };
            Line::from(vec![
                Span::styled(visit_sym, visit_style),
                Span::styled(
                    format!("[{}] ", key),
                    theme::dim_style(),
                ),
                Span::styled(
                    name,
                    Style::default().fg(theme::TEXT_PRIMARY),
                ),
                Span::styled(
                    format!(" [{}]", diff.badge()),
                    Style::default().fg(diff.color()).add_modifier(Modifier::DIM),
                ),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.current_demo)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme::TEXT_DIM)))
        .style(theme::dim_style())
        .highlight_style(
            Style::default()
                .fg(theme::RUST_ORANGE)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demos::DemoRegistry;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_demo_0_selected() {
        let app = App::new(15);
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(200, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_nav(f, f.area(), &app, &registry)).unwrap();
    }

    #[test]
    fn test_render_last_demo_selected() {
        let mut app = App::new(15);
        app.current_demo = 14;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(200, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_nav(f, f.area(), &app, &registry)).unwrap();
    }

    #[test]
    fn test_difficulty_all_indices() {
        for i in 0..15 {
            let d = demo_difficulty(i);
            assert!(!d.badge().is_empty());
            let _ = d.color();
        }
        // Out of range falls back to Intermediate
        assert_eq!(demo_difficulty(99), Difficulty::Intermediate);
    }

    #[test]
    fn test_difficulty_beginner_demos() {
        assert_eq!(demo_difficulty(0), Difficulty::Beginner);
        assert_eq!(demo_difficulty(1), Difficulty::Beginner);
        assert_eq!(demo_difficulty(5), Difficulty::Beginner);
        assert_eq!(demo_difficulty(7), Difficulty::Beginner);
        assert_eq!(demo_difficulty(11), Difficulty::Beginner);
        assert_eq!(demo_difficulty(13), Difficulty::Beginner);
    }

    #[test]
    fn test_difficulty_advanced_demos() {
        assert_eq!(demo_difficulty(9), Difficulty::Advanced);
        assert_eq!(demo_difficulty(10), Difficulty::Advanced);
        assert_eq!(demo_difficulty(14), Difficulty::Advanced);
    }

    #[test]
    fn test_difficulty_intermediate_demos() {
        assert_eq!(demo_difficulty(2), Difficulty::Intermediate);
        assert_eq!(demo_difficulty(3), Difficulty::Intermediate);
        assert_eq!(demo_difficulty(4), Difficulty::Intermediate);
        assert_eq!(demo_difficulty(6), Difficulty::Intermediate);
        assert_eq!(demo_difficulty(8), Difficulty::Intermediate);
        assert_eq!(demo_difficulty(12), Difficulty::Intermediate);
    }

    #[test]
    fn test_difficulty_badge_values() {
        assert_eq!(Difficulty::Beginner.badge(), "B");
        assert_eq!(Difficulty::Intermediate.badge(), "I");
        assert_eq!(Difficulty::Advanced.badge(), "A");
    }

    #[test]
    fn test_difficulty_colors_distinct() {
        let b = Difficulty::Beginner.color();
        let i = Difficulty::Intermediate.color();
        let a = Difficulty::Advanced.color();
        assert_ne!(b, i);
        assert_ne!(i, a);
        assert_ne!(b, a);
    }

    #[test]
    fn test_keys_cover_15_demos() {
        for i in 0..15 {
            let k = KEYS.get(i).copied().unwrap_or("?");
            assert_ne!(k, "?", "No key defined for demo index {}", i);
        }
    }

    #[test]
    fn test_render_with_some_visited() {
        let mut app = App::new(15);
        // Visit a few demos
        for i in [0, 3, 7, 9, 14] { app.visit(i); }
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(200, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_nav(f, f.area(), &app, &registry)).unwrap();
    }

    #[test]
    fn test_render_all_visited() {
        let mut app = App::new(15);
        for i in 0..15 { app.visit(i); }
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(200, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_nav(f, f.area(), &app, &registry)).unwrap();
    }
}
