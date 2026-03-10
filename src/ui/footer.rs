use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{app::App, demos::DemoRegistry, theme};

/// 26 Rust facts that cycle in the footer ticker.
pub fn rust_facts() -> &'static [&'static str] {
    &[
        "Rust has been voted Stack Overflow's #1 Most Loved Language 8 years in a row.",
        "The Linux kernel accepted Rust as its second official language in 2022.",
        "Rust's borrow checker has never allowed a data race in safe code.",
        "Mozilla created Rust to build safer browser components in Firefox.",
        "Rust programs can run without a garbage collector or runtime.",
        "The Rust compiler catches use-after-free, double-free, and null pointer bugs at compile time.",
        "Rust's cargo build tool downloads and compiles dependencies automatically.",
        "WebAssembly support makes Rust a first-class language for the web.",
        "Microsoft has adopted Rust for Windows components to reduce memory safety bugs.",
        "The Android Open Source Project is rewriting core components in Rust.",
        "Rust's zero-cost abstractions mean high-level code compiles to optimal assembly.",
        "Rust closures, iterators, and trait objects compile to the same code as hand-written loops.",
        "The `unsafe` keyword lets you opt out of safety checks when needed — but it stays local.",
        "Rust's trait system is more powerful than Java interfaces, with no runtime overhead.",
        "Amazon, Google, Meta, and Apple all use Rust in production systems.",
        "Rust prevents iterator invalidation by tracking borrows through the type system.",
        "A Rust `enum` can hold data, making it a full algebraic data type.",
        "Rust's `Result<T, E>` type eliminates null pointer exceptions at the type level.",
        "The Ferris crab (🦀) is Rust's unofficial mascot, created by artist Karen Rustad Tölva.",
        "Rust's lifetime annotations let you write provably correct code without a GC pause.",
        "Rust binaries can be as small as 1.8 KB when targeting embedded no_std environments.",
        "Cargo has over 150,000 crates on crates.io and counting.",
        "Rust supports async/await for high-performance concurrent I/O without thread overhead.",
        "The `clippy` linter catches hundreds of common mistakes before they reach production.",
        "Rust's pattern matching is exhaustive — the compiler ensures every case is handled.",
        "rayon makes data-parallel code trivially correct: just replace `.iter()` with `.par_iter()`.",
    ]
}

pub fn render_footer(frame: &mut Frame, area: Rect, app: &App, registry: &DemoRegistry) {
    if area.height == 0 {
        return;
    }

    let facts = rust_facts();

    // Advance every 90 ticks (~3 s at 30fps); scroll-in from right over 40 ticks
    let fact_period: u64 = 90;
    let fact_idx = (app.fact_tick / fact_period) as usize % facts.len();
    let scroll_tick = app.fact_tick % fact_period;

    let fact = facts[fact_idx];

    // Creep the fact string into view from the right over the first 30 ticks.
    // Use char count (not byte length) so multi-byte characters don't cause
    // a byte-boundary panic when slicing.
    let char_count = fact.chars().count();
    let visible_char_count = if scroll_tick < 30 {
        (char_count * scroll_tick as usize / 30).min(char_count)
    } else {
        char_count
    };
    let fact_display = if visible_char_count == 0 {
        ""
    } else {
        // Find the byte offset of the visible_char_count-th character boundary.
        fact.char_indices()
            .nth(visible_char_count)
            .map(|(byte_i, _)| &fact[..byte_i])
            .unwrap_or(fact) // nth returns None when visible_char_count >= char_count
    };

    let speed_text = format!("Speed:{}x", app.speed);
    let pause_hint = if app.paused {
        Span::styled(
            " ⏸ PAUSED  ",
            Style::default()
                .fg(theme::RUST_ORANGE)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let step_hint = if registry.supports_step_control(app.current_demo) {
        vec![
            Span::styled(" │ ", theme::dim_style()),
            Span::styled(
                "N/P: Step",
                Style::default()
                    .fg(theme::SAFE_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    let quiz_hint = if registry.quiz_current(app.current_demo).is_some() {
        vec![
            Span::styled(" │ ", theme::dim_style()),
            Span::styled(
                "T: Quiz",
                Style::default()
                    .fg(theme::BORROW_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    // Line 0: Rust Facts ticker
    let fact_line = Line::from(vec![
        Span::styled("💡 ", Style::default().fg(theme::BORROW_YELLOW)),
        Span::styled(fact_display, theme::dim_style()),
    ]);

    // Line 1: key controls
    let mut controls_spans = vec![
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
        Span::styled("V: vs-mode", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("E: Explain", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("?: Help", theme::dim_style()),
        Span::styled(" │ ", theme::dim_style()),
        Span::styled("Q: Quit", Style::default().fg(theme::CRAB_RED)),
    ];
    controls_spans.extend(step_hint);
    controls_spans.extend(quiz_hint);
    let controls_line = Line::from(controls_spans);

    frame.render_widget(Paragraph::new(vec![fact_line, controls_line]), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demos::DemoRegistry;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_rust_facts_count() {
        let facts = rust_facts();
        assert!(
            facts.len() >= 24,
            "expected >= 24 facts, got {}",
            facts.len()
        );
    }

    #[test]
    fn test_rust_facts_nonempty() {
        for fact in rust_facts() {
            assert!(!fact.is_empty(), "rust_facts must not have empty entries");
        }
    }

    #[test]
    fn test_render_running() {
        let app = App::new(16);
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_footer(f, f.area(), &app, &registry))
            .unwrap();
    }

    #[test]
    fn test_render_paused() {
        let mut app = App::new(16);
        app.paused = true;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_footer(f, f.area(), &app, &registry))
            .unwrap();
    }

    #[test]
    fn test_render_fact_cycles() {
        let mut app = App::new(16);
        // Advance far enough to cycle facts
        app.fact_tick = 90 * 100;
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_footer(f, f.area(), &app, &registry))
            .unwrap();
    }

    #[test]
    fn test_render_scrolling_in() {
        let mut app = App::new(16);
        app.fact_tick = 10; // mid-scroll
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(120, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_footer(f, f.area(), &app, &registry))
            .unwrap();
    }

    #[test]
    fn test_render_multibyte_fact_mid_scroll() {
        // Fact index 12 contains an em-dash (—, 3 bytes).
        // scroll_tick values 1-29 used to panic on byte-boundary slicing.
        let facts = rust_facts();
        let emdash_fact_idx = facts
            .iter()
            .position(|f| f.contains('—'))
            .expect("expected at least one fact with an em-dash");
        let mut app = App::new(16);
        let registry = DemoRegistry::new();
        for tick_offset in [5u64, 10, 15, 20, 25] {
            app.fact_tick = emdash_fact_idx as u64 * 90 + tick_offset;
            let backend = TestBackend::new(120, 2);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| render_footer(f, f.area(), &app, &registry))
                .unwrap();
        }
    }

    #[test]
    fn test_render_zero_area() {
        let app = App::new(16);
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        // render into a zero-height rect — should not panic
        let zero_rect = Rect::new(0, 0, 80, 0);
        terminal
            .draw(|f| render_footer(f, zero_rect, &app, &registry))
            .unwrap();
    }

    #[test]
    fn test_render_step_control_demo() {
        let mut app = App::new(16);
        app.current_demo = 15; // d16 Macros supports step control
        let registry = DemoRegistry::new();
        let backend = TestBackend::new(160, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_footer(f, f.area(), &app, &registry))
            .unwrap();
    }
}
