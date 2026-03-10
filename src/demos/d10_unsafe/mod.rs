use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

const STEPS: usize = 5;

#[derive(Debug)]
pub struct UnsafeDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub ptr_offset: usize,
    ptr_timer: f64,
}

impl UnsafeDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            ptr_offset: 0,
            ptr_timer: 0.0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        3.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.ptr_offset = 0;
        self.ptr_timer = 0.0;
    }
}

/// Reads 8 u32 values via raw pointer arithmetic and returns them.
/// Demonstrates that unsafe does not mean unsound — the SAFETY invariant is maintained.
pub fn raw_ptr_demo() -> Vec<u32> {
    let data: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut result = Vec::with_capacity(8);
    let ptr = data.as_ptr();
    for i in 0..8 {
        // SAFETY: `i` is in range 0..8, which is within the bounds of `data`.
        let val = unsafe { *ptr.add(i) };
        result.push(val);
    }
    result
}

/// Returns (unsafe_n / total) * 100.0, or 0.0 if total is zero.
pub fn unsafe_line_percentage(unsafe_n: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (unsafe_n as f64 / total as f64) * 100.0
}

/// The five capabilities that `unsafe` unlocks — nothing more.
pub fn list_unsafe_superpowers() -> Vec<&'static str> {
    vec![
        "Dereference raw pointers",
        "Call unsafe functions",
        "Access mutable static variables",
        "Implement unsafe traits",
        "Access fields of unions",
    ]
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/5: What unsafe unlocks — exactly 5 superpowers",
        1 => "Step 2/5: Raw pointer arithmetic — live demo via unsafe block",
        2 => "Step 3/5: Safe abstraction over unsafe internals",
        3 => "Step 4/5: unsafe in real codebases — small, isolated, annotated",
        _ => "Step 5/5: Best practices — minimize, isolate, document SAFETY",
    }
}

fn step_explanation(step: usize) -> &'static str {
    match step % STEPS {
        0 => "unsafe in Rust unlocks exactly 5 capabilities. Everything else — borrow checker, type system, lifetime rules — remains fully active inside unsafe blocks.",
        1 => "Raw pointer arithmetic is the canonical use of unsafe. We create a raw pointer to an array and walk it manually. The SAFETY comment documents why this cannot cause UB.",
        2 => "The best unsafe pattern: wrap it in a safe abstraction. Vec<T>, Rc<T>, and most of std are built this way — unsafe internals, safe public API.",
        3 => "In typical Rust projects, unsafe accounts for less than 1% of lines. Clippy and cargo-geiger can measure your unsafe surface area.",
        _ => "Rule of thumb: unsafe blocks should be small, isolated, heavily documented, and reviewed like security-sensitive code. Every unsafe block needs a SAFETY comment.",
    }
}

impl Default for UnsafeDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for UnsafeDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        self.ptr_timer += dt.as_secs_f64();

        // Animate ptr_offset 0..=7 when on the raw pointer step
        if self.step % STEPS == 1 && self.ptr_timer >= 0.3 / self.speed as f64 {
            self.ptr_offset = (self.ptr_offset + 1) % 8;
            self.ptr_timer = 0.0;
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

        match self.step % STEPS {
            0 => {
                let superpowers = list_unsafe_superpowers();
                let items: Vec<ListItem> = superpowers
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let style = if i % 2 == 0 {
                            Style::default().fg(theme::BORROW_YELLOW)
                        } else {
                            Style::default().fg(theme::STACK_CYAN)
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("  {}. ", i + 1), theme::dim_style()),
                            Span::styled(*s, style),
                        ]))
                    })
                    .collect();
                frame.render_widget(
                    List::new(items).block(
                        Block::default()
                            .title("Unsafe Superpowers")
                            .borders(Borders::ALL),
                    ),
                    chunks[1],
                );
            }
            1 => {
                let values = raw_ptr_demo();
                let mut lines: Vec<Line> = vec![
                    Line::from(Span::styled(
                        "let data: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled("let ptr = data.as_ptr();", theme::dim_style())),
                    Line::from(""),
                    Line::from(Span::styled(
                        "// SAFETY: i in 0..8, within bounds of `data`",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "unsafe { *ptr.add(i) }",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Values read:", theme::dim_style())),
                ];
                let spans: Vec<Span> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let style = if i == self.ptr_offset {
                            Style::default()
                                .fg(theme::RUST_ORANGE)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme::dim_style()
                        };
                        Span::styled(format!(" {v}"), style)
                    })
                    .collect();
                lines.push(Line::from(spans));
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("Raw Pointer Demo")
                            .borders(Borders::ALL),
                    ),
                    chunks[1],
                );
            }
            2 => {
                let lines = vec![
                    Line::from(Span::styled(
                        "// Safe public API — users never see unsafe",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "pub fn get_unchecked_safe(slice: &[u8], idx: usize) -> Option<u8> {",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "    if idx < slice.len() {",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "        // SAFETY: idx is checked above to be in-bounds",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "        Some(unsafe { *slice.as_ptr().add(idx) })",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled("    } else {", theme::dim_style())),
                    Line::from(Span::styled(
                        "        None",
                        Style::default().fg(theme::CRAB_RED),
                    )),
                    Line::from(Span::styled("    }", theme::dim_style())),
                    Line::from(Span::styled("}", theme::dim_style())),
                    Line::from(""),
                    Line::from(Span::styled(
                        "// This is exactly how Vec, String, HashMap are built.",
                        theme::dim_style(),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("Safe Abstraction Pattern")
                            .borders(Borders::ALL),
                    ),
                    chunks[1],
                );
            }
            3 => {
                let total_lines: u64 = 2400;
                let unsafe_lines: u64 = 23;
                let pct = unsafe_line_percentage(unsafe_lines, total_lines);
                let pct_ratio = (pct / 100.0).clamp(0.0, 1.0);

                let inner = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(2),
                    ])
                    .split(chunks[1]);

                frame.render_widget(
                    Paragraph::new(vec![Line::from(Span::styled(
                        format!(
                            "  Total lines: {}   Unsafe lines: {}   = {:.2}%",
                            total_lines, unsafe_lines, pct
                        ),
                        theme::dim_style(),
                    ))])
                    .block(
                        Block::default()
                            .title("Unsafe Surface Area")
                            .borders(Borders::ALL),
                    ),
                    inner[0],
                );

                frame.render_widget(
                    Gauge::default()
                        .block(Block::default().title("unsafe %").borders(Borders::ALL))
                        .gauge_style(Style::default().fg(theme::BORROW_YELLOW))
                        .ratio(pct_ratio),
                    inner[1],
                );

                frame.render_widget(
                    Paragraph::new(vec![
                        Line::from(Span::styled(
                            "  cargo-geiger: measure your unsafe footprint",
                            theme::dim_style(),
                        )),
                        Line::from(Span::styled(
                            "  clippy: warns about unnecessary unsafe",
                            theme::dim_style(),
                        )),
                    ])
                    .block(Block::default().title("Tools").borders(Borders::ALL)),
                    inner[2],
                );
            }
            _ => {
                let lines = vec![
                    Line::from(Span::styled(
                        "  Rule 1: Keep unsafe blocks as small as possible.",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "  Rule 2: Every unsafe block MUST have a // SAFETY: comment.",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "  Rule 3: Wrap unsafe in a safe public abstraction.",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "  Rule 4: Use cargo-geiger to audit unsafe in dependencies.",
                        Style::default().fg(theme::BORROW_YELLOW),
                    )),
                    Line::from(Span::styled(
                        "  Rule 5: Prefer safe alternatives when performance is not critical.",
                        Style::default().fg(theme::BORROW_YELLOW),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  unsafe does NOT disable: borrow checker, type system, lifetimes.",
                        Style::default()
                            .fg(theme::STACK_CYAN)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        "  unsafe ONLY unlocks the 5 superpowers listed in step 1.",
                        Style::default().fg(theme::STACK_CYAN),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("Best Practices")
                            .borders(Borders::ALL),
                    ),
                    chunks[1],
                );
            }
        }

        frame.render_widget(
            Paragraph::new(step_explanation(self.step))
                .block(
                    Block::default()
                        .title("Explanation")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true }),
            chunks[2],
        );
    }

    fn name(&self) -> &'static str {
        "Unsafe Rust"
    }

    fn description(&self) -> &'static str {
        "Controlled power — explicit, isolated, auditable."
    }

    fn explanation(&self) -> &'static str {
        "unsafe blocks allow raw pointer arithmetic, FFI calls, and manual memory management. \
        But unsafe does NOT disable the borrow checker or type system — \
        it only unlocks 5 specific low-level capabilities. \
        Best practice: minimize unsafe surface area, isolate it, and document every SAFETY invariant."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.ptr_offset = 0;
        self.ptr_timer = 0.0;
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

    fn quiz(&self) -> Option<(&'static str, [&'static str; 4], usize)> {
        Some((
            "What does `unsafe` in Rust actually disable?",
            [
                "All compilation checks",
                "Specific safety guarantees the compiler can't verify",
                "Memory allocation",
                "The borrow checker entirely",
            ],
            1,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_raw_ptr_demo_values() {
        let vals = raw_ptr_demo();
        assert_eq!(vals, vec![1u32, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_unsafe_line_percentage_normal() {
        let pct = unsafe_line_percentage(23, 2400);
        // 23/2400 * 100 ≈ 0.9583
        assert!((pct - 0.9583333333333334_f64).abs() < 1e-6);
    }

    #[test]
    fn test_unsafe_line_percentage_zero_total() {
        assert_eq!(unsafe_line_percentage(10, 0), 0.0);
    }

    #[test]
    fn test_unsafe_line_percentage_zero_unsafe() {
        assert_eq!(unsafe_line_percentage(0, 1000), 0.0);
    }

    #[test]
    fn test_unsafe_line_percentage_full() {
        let pct = unsafe_line_percentage(100, 100);
        assert!((pct - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_list_unsafe_superpowers_len() {
        assert_eq!(list_unsafe_superpowers().len(), 5);
    }

    #[test]
    fn test_list_unsafe_superpowers_content() {
        let sp = list_unsafe_superpowers();
        assert!(sp.contains(&"Dereference raw pointers"));
        assert!(sp.contains(&"Call unsafe functions"));
        assert!(sp.contains(&"Access mutable static variables"));
        assert!(sp.contains(&"Implement unsafe traits"));
        assert!(sp.contains(&"Access fields of unions"));
    }

    #[test]
    fn test_demo_trait_methods() {
        let mut d = UnsafeDemo::new();
        assert_eq!(d.name(), "Unsafe Rust");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
        assert!(!d.is_paused());
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
        d.set_speed(6);
        assert_eq!(d.speed(), 6);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = UnsafeDemo::new();
        d.step = 3;
        d.tick_count = 77;
        d.ptr_offset = 5;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.ptr_offset, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused() {
        let mut d = UnsafeDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_ptr_offset_advances_on_step1() {
        let mut d = UnsafeDemo::new();
        d.step = 1;
        // threshold is 0.3s at speed=1
        d.tick(Duration::from_secs_f64(0.4));
        assert_eq!(d.ptr_offset, 1);
    }

    #[test]
    fn test_ptr_offset_wraps_at_8() {
        let mut d = UnsafeDemo::new();
        d.step = 1;
        d.ptr_offset = 7;
        d.ptr_timer = 10.0;
        d.tick(Duration::from_micros(1));
        assert_eq!(d.ptr_offset, 0);
    }

    #[test]
    fn test_ptr_offset_does_not_advance_on_other_steps() {
        let mut d = UnsafeDemo::new();
        d.step = 0;
        d.tick(Duration::from_secs_f64(1.0));
        assert_eq!(d.ptr_offset, 0);
    }

    #[test]
    fn test_advance_step_wraps() {
        let mut d = UnsafeDemo::new();
        d.step = STEPS - 1;
        d.advance_step();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = UnsafeDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = UnsafeDemo::default();
        assert_eq!(d.step, 0);
        assert_eq!(d.ptr_offset, 0);
    }

    #[test]
    fn test_step_duration_secs() {
        let mut d = UnsafeDemo::new();
        d.set_speed(3);
        let dur = d.step_duration_secs();
        assert!((dur - 1.0).abs() < 1e-6);
    }
}
