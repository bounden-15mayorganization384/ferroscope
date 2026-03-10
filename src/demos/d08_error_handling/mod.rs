use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::{demos::Demo, theme};
use thiserror::Error;

const STEPS: usize = 6;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Permission denied")]
    PermissionDenied,
}

pub fn safe_parse_demo(s: &str) -> Option<i32> {
    s.parse::<i32>().ok()
}

pub fn simulate_error_chain(depth: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for i in 0..=depth {
        let indent = "  ".repeat(i);
        if i == depth {
            lines.push(format!("{}fn level_{}() -> Result<(), AppError> {{", indent, i));
            lines.push(format!("{}    Err(AppError::Io(\"disk full\".into()))", indent));
            lines.push(format!("{}}}", indent));
        } else {
            lines.push(format!("{}fn level_{}() -> Result<(), AppError> {{", indent, i));
            lines.push(format!(
                "{}    level_{}()?  // ? propagates error up",
                indent,
                i + 1
            ));
            lines.push(format!("{}}}", indent));
        }
    }
    lines
}

pub fn categorize_error(err: &AppError) -> &'static str {
    match err {
        AppError::Io(_) => "recoverable",
        AppError::Parse(_) => "recoverable",
        AppError::NotFound(_) => "recoverable",
        AppError::PermissionDenied => "logic",
    }
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/6: Result<T,E> — errors as values, not exceptions",
        1 => "Step 2/6: The ? operator — propagate errors up the call stack",
        2 => "Step 3/6: Option<T> combinators — safe null handling",
        3 => "Step 4/6: Custom error types with thiserror",
        4 => "Step 5/6: Exceptions vs Result — a side-by-side comparison",
        _ => "Step 6/6: Option — safe parsing, no crashes",
    }
}

#[derive(Debug)]
pub struct ErrorHandlingDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub chain_depth: usize,
    chain_timer: f64,
}

impl ErrorHandlingDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            chain_depth: 0,
            chain_timer: 0.0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        3.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.chain_depth = 0;
        self.chain_timer = 0.0;
    }
}

impl Default for ErrorHandlingDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for ErrorHandlingDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        self.chain_timer += dt.as_secs_f64();

        if self.chain_timer >= 0.5 / self.speed as f64 {
            self.chain_depth = (self.chain_depth + 1).min(4);
            self.chain_timer = 0.0;
        }

        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                step_title(self.step),
                Style::default()
                    .fg(theme::RUST_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::RUST_ORANGE)),
            ),
            chunks[0],
        );

        let content_lines: Vec<Line> = match self.step % STEPS {
            0 => vec![
                Line::from(Span::styled(
                    "fn read_file(path: &str) -> Result<String, AppError> {",
                    theme::dim_style(),
                )),
                Line::from(Span::styled("    if !exists(path) {", theme::dim_style())),
                Line::from(Span::styled(
                    "        return Err(AppError::NotFound(path.into()));  // ← error",
                    Style::default().fg(theme::CRAB_RED),
                )),
                Line::from(Span::styled("    }", theme::dim_style())),
                Line::from(Span::styled(
                    "    Ok(std::fs::read_to_string(path)?)",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled("}", theme::dim_style())),
                Line::from(""),
                Line::from(Span::styled(
                    "// Caller MUST handle both Ok and Err",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "match read_file(\"data.txt\") {",
                    theme::dim_style(),
                )),
                Line::from(Span::styled(
                    "    Ok(content) => println!(\"{}\", content),",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    Err(e)      => eprintln!(\"Error: {}\", e),",
                    Style::default().fg(theme::CRAB_RED),
                )),
                Line::from(Span::styled("}", theme::dim_style())),
            ],
            1 => {
                let chain = simulate_error_chain(self.chain_depth);
                chain
                    .iter()
                    .map(|l| Line::from(Span::styled(l.clone(), theme::dim_style())))
                    .collect()
            }
            2 => vec![
                Line::from(Span::styled(
                    "let s: Option<&str> = Some(\"42\");",
                    theme::dim_style(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// Chaining on Some:",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "s.map(|v| v.trim())",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    " .and_then(|v| v.parse::<i32>().ok())",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    " .unwrap_or(0)  // → 42",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// On None:",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "None::<&str>.map(...).and_then(...)",
                    theme::dim_style(),
                )),
                Line::from(Span::styled(
                    "  .unwrap_or(0)  // → 0 (no crash!)",
                    Style::default()
                        .fg(theme::STACK_CYAN)
                        .add_modifier(Modifier::BOLD),
                )),
            ],
            3 => vec![
                Line::from(Span::styled(
                    "#[derive(Error, Debug)]",
                    theme::dim_style(),
                )),
                Line::from(Span::styled("enum AppError {", theme::dim_style())),
                Line::from(Span::styled(
                    "    #[error(\"IO error: {0}\")]",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "    Io(String),",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    #[error(\"Parse error: {0}\")]",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "    Parse(String),",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    #[error(\"Not found: {0}\")]",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "    NotFound(String),",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    #[error(\"Permission denied\")]",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "    PermissionDenied,",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled("}", theme::dim_style())),
            ],
            4 => {
                let mid = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(chunks[1]);
                let exceptions = vec![
                    Line::from(Span::styled(
                        "Java/Python:",
                        Style::default()
                            .fg(theme::CRAB_RED)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled("try {", theme::dim_style())),
                    Line::from(Span::styled("  doSomething();", theme::dim_style())),
                    Line::from(Span::styled(
                        "} catch (Exception e) {",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "  // Easy to forget this!",
                        Style::default().fg(theme::CRAB_RED),
                    )),
                    Line::from(Span::styled("}", theme::dim_style())),
                    Line::from(""),
                    Line::from(Span::styled(
                        "x Exception: unchecked",
                        Style::default().fg(theme::CRAB_RED),
                    )),
                    Line::from(Span::styled(
                        "x Can silently ignore errors",
                        Style::default().fg(theme::CRAB_RED),
                    )),
                    Line::from(Span::styled(
                        "x Stack unwind overhead",
                        Style::default().fg(theme::CRAB_RED),
                    )),
                ];
                let rust_err = vec![
                    Line::from(Span::styled(
                        "Rust:",
                        Style::default()
                            .fg(theme::SAFE_GREEN)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        "let result = do_something()?;",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "// ? forces you to handle the error",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(Span::styled(
                        "✓ Errors in type signature",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "✓ Cannot silently ignore",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "✓ Zero-cost when no error",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(exceptions).block(
                        Block::default()
                            .title("Other Languages")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::CRAB_RED)),
                    ),
                    mid[0],
                );
                frame.render_widget(
                    Paragraph::new(rust_err).block(
                        Block::default()
                            .title("Rust")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::SAFE_GREEN)),
                    ),
                    mid[1],
                );
                vec![]
            }
            _ => {
                let tests = [
                    ("\"42\"", safe_parse_demo("42")),
                    ("\"abc\"", safe_parse_demo("abc")),
                    ("\"\"", safe_parse_demo("")),
                ];
                tests
                    .iter()
                    .map(|(input, result)| {
                        let (color, val_str) = match result {
                            Some(v) => (theme::SAFE_GREEN, format!("Some({v})")),
                            None => (theme::CRAB_RED, "None  (no crash!)".to_string()),
                        };
                        Line::from(vec![
                            Span::styled(
                                format!("  safe_parse({input}) → "),
                                theme::dim_style(),
                            ),
                            Span::styled(
                                val_str,
                                Style::default()
                                    .fg(color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])
                    })
                    .collect()
            }
        };

        if self.step % STEPS != 4 {
            frame.render_widget(
                Paragraph::new(content_lines)
                    .block(Block::default().title("Error Handling").borders(Borders::ALL)),
                chunks[1],
            );
        }

        let expl_text = match self.step % STEPS {
            0 => "Result<T,E> forces the caller to acknowledge both success (Ok) and failure (Err). It's impossible to ignore an error without explicitly writing code to do so.",
            1 => "The ? operator unwraps Ok or returns Err immediately. It propagates errors up the call stack automatically, like exceptions — but the propagation is visible in the type signature.",
            2 => "Option<T> replaces null. Combinators (.map, .and_then, .unwrap_or) allow safe transformation without explicit null checks. No NullPointerException is possible.",
            3 => "thiserror generates Display implementations for custom error enums. Each variant can carry data and has a human-readable message.",
            4 => "Unlike exception-based error handling, Rust's Result has zero runtime overhead when no error occurs. Errors are just values — no stack unwinding, no hidden control flow.",
            _ => "safe_parse() returns Option<i32>. Invalid input returns None, not a panic or exception. The caller decides how to handle the missing case.",
        };
        frame.render_widget(
            Paragraph::new(expl_text)
                .block(
                    Block::default()
                        .title("Key Insight")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true }),
            chunks[2],
        );
    }

    fn name(&self) -> &'static str {
        "Error Handling"
    }

    fn description(&self) -> &'static str {
        "Errors as values — no exceptions, no null, no surprises."
    }

    fn explanation(&self) -> &'static str {
        "Result<T,E> and Option<T> make errors explicit in the type system. \
        The ? operator propagates errors up the call stack with zero overhead. \
        You cannot ignore a Result without the compiler warning. \
        No null pointer exceptions, no unhandled exceptions, no hidden failure modes."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.chain_depth = 0;
        self.chain_timer = 0.0;
        self.paused = false;
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn set_speed(&mut self, speed: u8) {
        self.speed = speed.clamp(1, 10);
    }

    fn speed(&self) -> u8 {
        self.speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_safe_parse_valid() {
        assert_eq!(safe_parse_demo("42"), Some(42));
    }

    #[test]
    fn test_safe_parse_invalid() {
        assert_eq!(safe_parse_demo("abc"), None);
    }

    #[test]
    fn test_safe_parse_empty() {
        assert_eq!(safe_parse_demo(""), None);
    }

    #[test]
    fn test_safe_parse_negative() {
        assert_eq!(safe_parse_demo("-7"), Some(-7));
    }

    #[test]
    fn test_simulate_error_chain_lengths() {
        for depth in 0..=4 {
            let lines = simulate_error_chain(depth);
            assert!(!lines.is_empty(), "depth {} should have lines", depth);
            assert_eq!(lines.len(), (depth + 1) * 3);
        }
    }

    #[test]
    fn test_categorize_error_all_variants() {
        assert_eq!(categorize_error(&AppError::Io("x".into())), "recoverable");
        assert_eq!(
            categorize_error(&AppError::Parse("x".into())),
            "recoverable"
        );
        assert_eq!(
            categorize_error(&AppError::NotFound("x".into())),
            "recoverable"
        );
        assert_eq!(categorize_error(&AppError::PermissionDenied), "logic");
    }

    #[test]
    fn test_app_error_display() {
        assert!(AppError::Io("disk full".into())
            .to_string()
            .contains("IO error"));
        assert!(AppError::Parse("bad".into())
            .to_string()
            .contains("Parse error"));
        assert!(AppError::NotFound("file".into())
            .to_string()
            .contains("Not found"));
        assert_eq!(AppError::PermissionDenied.to_string(), "Permission denied");
    }

    #[test]
    fn test_demo_trait_methods() {
        let mut d = ErrorHandlingDemo::new();
        assert_eq!(d.name(), "Error Handling");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
        assert!(!d.is_paused());
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
        d.set_speed(7);
        assert_eq!(d.speed(), 7);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = ErrorHandlingDemo::new();
        d.step = 4;
        d.tick_count = 100;
        d.chain_depth = 3;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.chain_depth, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused() {
        let mut d = ErrorHandlingDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_chain_depth_capped_at_4() {
        let mut d = ErrorHandlingDemo::new();
        d.chain_depth = 4;
        d.chain_timer = 10.0;
        d.tick(Duration::from_micros(1));
        assert_eq!(d.chain_depth, 4);
    }

    #[test]
    fn test_advance_step_wraps() {
        let mut d = ErrorHandlingDemo::new();
        d.step = STEPS - 1;
        d.advance_step();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = ErrorHandlingDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = ErrorHandlingDemo::default();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_step_duration_secs() {
        let mut d = ErrorHandlingDemo::new();
        d.set_speed(3);
        let dur = d.step_duration_secs();
        assert!((dur - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chain_depth_animates() {
        let mut d = ErrorHandlingDemo::new();
        // At speed=1, chain animates every 0.5s
        d.tick(Duration::from_secs_f64(0.6));
        assert_eq!(d.chain_depth, 1);
    }

    #[test]
    fn test_simulate_error_chain_content() {
        let lines = simulate_error_chain(2);
        // Should mention ? propagation
        let joined = lines.join("\n");
        assert!(joined.contains("propagates error up"));
    }
}
