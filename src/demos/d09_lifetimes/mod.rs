use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Rectangle},
        Block, Borders, Paragraph,
    },
    Frame,
};
use std::time::Duration;

const STEPS: usize = 5;

#[derive(Debug)]
pub struct LifetimesDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub show_error: bool,
    error_timer: f64,
}

impl LifetimesDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            show_error: false,
            error_timer: 0.0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        3.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.show_error = false;
        self.error_timer = 0.0;
    }
}

/// Returns a visualization of nested scope brackets at the given animation depth.
/// depth 0 = outer scope only, depth 1 = one inner scope, etc.
pub fn scope_bracket_lines(depth: usize) -> Vec<String> {
    match depth {
        0 => vec![
            "{ // outer scope".to_string(),
            "".to_string(),
            "} // outer scope ends".to_string(),
        ],
        1 => vec![
            "{ // outer scope".to_string(),
            "    let s = String::from(\"hello\");  // s lives here".to_string(),
            "".to_string(),
            "} // s dropped here".to_string(),
        ],
        2 => vec![
            "{ // outer scope".to_string(),
            "    let s = String::from(\"hello\");".to_string(),
            "    let r = &s;  // r borrows s".to_string(),
            "    println!(\"{}\", r);  // valid: s still alive".to_string(),
            "".to_string(),
            "} // r and s both drop here — no dangling ref".to_string(),
        ],
        _ => vec![
            "{ // outer scope".to_string(),
            "    let r;  // r declared (uninitialized)".to_string(),
            "    {".to_string(),
            "        let s = String::from(\"inner\");".to_string(),
            "        r = &s;  // r borrows s".to_string(),
            "    } // s dropped! r now DANGLING".to_string(),
            "    println!(\"{}\", r);  // ERROR: s does not live long enough".to_string(),
            "} // outer scope".to_string(),
        ],
    }
}

/// Returns (line, is_annotated) pairs showing lifetime annotations on a function.
pub fn lifetime_annotation_example() -> Vec<(&'static str, bool)> {
    vec![
        ("// Without lifetime annotations:", false),
        (
            "// fn longest(x: &str, y: &str) -> &str  ← compile error",
            false,
        ),
        ("", false),
        ("// With lifetime annotations:", false),
        ("fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {", true),
        ("    if x.len() > y.len() { x } else { y }", false),
        ("}", false),
        ("", false),
        ("// 'a means: the returned reference lives at least", false),
        ("// as long as the shorter of x and y.", true),
        ("// The compiler uses this to prevent dangling refs.", false),
    ]
}

/// Returns true only when the step corresponds to the dangling reference scenario.
pub fn is_dangling_scenario(step: usize) -> bool {
    step == 2
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/5: Scopes — every value has a lifetime tied to its scope",
        1 => "Step 2/5: Valid Borrow — reference lives within owner's scope",
        2 => "Step 3/5: Dangling Reference — borrow checker prevents use-after-free",
        3 => "Step 4/5: Lifetime Annotations — explicit relationship between refs",
        _ => "Step 5/5: 'static lifetime — lives for the entire program duration",
    }
}

fn step_explanation(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Every value in Rust has a lifetime — the region of code where it is valid. When a value goes out of scope, it is dropped immediately. The borrow checker tracks lifetimes at compile time.",
        1 => "A borrow (reference) is valid as long as it does not outlive the value it refers to. Here, r and s are both in scope, so the borrow is safe.",
        2 => "The borrow checker rejects this at compile time: s is dropped at the end of the inner block, but r tries to use it afterward. This is a use-after-free — impossible in safe Rust.",
        3 => "Lifetime annotations ('a) tell the compiler how the lifetimes of input and output references relate. They do not change how long values live — they just make the relationship explicit.",
        _ => "String literals like \"hello\" have the 'static lifetime — they are embedded in the binary and live for the entire program. Box::leak and lazy statics also produce 'static references.",
    }
}

impl Default for LifetimesDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for LifetimesDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        self.error_timer += dt.as_secs_f64();

        // For dangling reference step, flash show_error every 0.5s
        if is_dangling_scenario(self.step % STEPS) {
            if self.error_timer >= 0.5 / self.speed as f64 {
                self.show_error = !self.show_error;
                self.error_timer = 0.0;
            }
        } else {
            self.show_error = false;
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
            0 => {
                // Animate scope brackets growing
                let depth = ((self.step_timer * self.speed as f64 * 1.5) as usize).min(3);
                scope_bracket_lines(depth)
                    .iter()
                    .map(|l| Line::from(Span::styled(l.clone(), theme::dim_style())))
                    .collect()
            }
            1 => vec![
                Line::from(Span::styled(
                    "{ // outer scope",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    let s = String::from(\"hello\");",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    let r = &s;  // r borrows s — valid",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "    println!(\"{}\", r);  // OK: s still alive",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "} // both r and s drop here",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(""),
                Line::from(Span::styled("  r's lifetime: ──────┐", theme::dim_style())),
                Line::from(Span::styled(
                    "  s's lifetime: ─────────┐",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "  r does NOT outlive s — safe!",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
            ],
            2 => {
                let error_style = if self.show_error {
                    Style::default()
                        .fg(theme::CRAB_RED)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::CRAB_RED)
                };
                vec![
                    Line::from(Span::styled(
                        "{ // outer scope",
                        Style::default().fg(theme::STACK_CYAN),
                    )),
                    Line::from(Span::styled(
                        "    let r;  // declared, not yet initialized",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "    {",
                        Style::default().fg(theme::BORROW_YELLOW),
                    )),
                    Line::from(Span::styled(
                        "        let s = String::from(\"inner\");",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(Span::styled(
                        "        r = &s;  // r borrows s",
                        Style::default().fg(theme::BORROW_YELLOW),
                    )),
                    Line::from(Span::styled(
                        "    } // s DROPPED HERE — r now dangling!",
                        error_style,
                    )),
                    Line::from(Span::styled(
                        "    println!(\"{}\", r);  // ERROR: use after free",
                        error_style,
                    )),
                    Line::from(Span::styled(
                        "} // ↑ borrow checker rejects this at compile time",
                        Style::default()
                            .fg(theme::CRAB_RED)
                            .add_modifier(Modifier::BOLD),
                    )),
                ]
            }
            3 => lifetime_annotation_example()
                .iter()
                .map(|(l, annotated)| {
                    let style = if *annotated {
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        theme::dim_style()
                    };
                    Line::from(Span::styled(*l, style))
                })
                .collect(),
            _ => vec![
                Line::from(Span::styled(
                    "// String literals have 'static lifetime",
                    theme::dim_style(),
                )),
                Line::from(Span::styled(
                    "let s: &'static str = \"hello, world\";",
                    Style::default()
                        .fg(theme::ASYNC_PURPLE)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// They are embedded in the binary — live forever",
                    theme::dim_style(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// Box::leak also produces 'static:",
                    theme::dim_style(),
                )),
                Line::from(Span::styled(
                    "let leaked: &'static str = Box::leak(",
                    Style::default().fg(theme::ASYNC_PURPLE),
                )),
                Line::from(Span::styled(
                    "    String::from(\"dynamic\").into_boxed_str()",
                    Style::default().fg(theme::ASYNC_PURPLE),
                )),
                Line::from(Span::styled(");", Style::default().fg(theme::ASYNC_PURPLE))),
                Line::from(""),
                Line::from(Span::styled(
                    "// Thread::spawn requires 'static bounds on closures",
                    theme::dim_style(),
                )),
                Line::from(Span::styled(
                    "// — ensures no dangling refs cross thread boundaries",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
            ],
        };

        // Split center panel: code left, Gantt chart right
        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(chunks[1]);

        frame.render_widget(
            Paragraph::new(content_lines).block(
                Block::default()
                    .title("Lifetimes Demo")
                    .borders(Borders::ALL),
            ),
            mid[0],
        );

        // ── Canvas Gantt chart ────────────────────────────────────────────────
        let step = self.step % STEPS;
        let cursor_x = (self.step_timer / self.step_duration_secs() * 100.0).min(99.5);

        let gantt = Canvas::default()
            .block(
                Block::default()
                    .title("Lifetime Timeline")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORROW_YELLOW)),
            )
            .x_bounds([-2.0, 102.0])
            .y_bounds([0.0, 12.0])
            .marker(ratatui::symbols::Marker::Braille)
            .paint(move |ctx| {
                // Timeline labels
                ctx.print(
                    -1.5,
                    9.5,
                    Span::styled("'r", Style::default().fg(theme::ASYNC_PURPLE)),
                );
                ctx.print(
                    -1.5,
                    6.5,
                    Span::styled("'s", Style::default().fg(theme::STACK_CYAN)),
                );

                match step {
                    0 => {
                        // Scopes: just outer scope bar
                        ctx.draw(&Rectangle {
                            x: 0.0,
                            y: 5.0,
                            width: 100.0,
                            height: 2.0,
                            color: theme::STACK_CYAN,
                        });
                        ctx.print(
                            15.0,
                            5.5,
                            Span::styled("outer scope", Style::default().fg(theme::TEXT_PRIMARY)),
                        );
                    }
                    1 => {
                        // Valid borrow: s and r both live same scope
                        ctx.draw(&Rectangle {
                            x: 0.0,
                            y: 4.5,
                            width: 100.0,
                            height: 1.5,
                            color: theme::SAFE_GREEN,
                        });
                        ctx.draw(&Rectangle {
                            x: 15.0,
                            y: 7.5,
                            width: 70.0,
                            height: 1.5,
                            color: theme::SAFE_GREEN,
                        });
                        ctx.print(
                            30.0,
                            5.0,
                            Span::styled("'s lives here", Style::default().fg(theme::TEXT_PRIMARY)),
                        );
                        ctx.print(
                            28.0,
                            8.0,
                            Span::styled(
                                "r borrows s — safe",
                                Style::default().fg(theme::TEXT_PRIMARY),
                            ),
                        );
                    }
                    2 => {
                        // Dangling: s ends at 55, r continues (CRAB_RED after 55)
                        ctx.draw(&Rectangle {
                            x: 0.0,
                            y: 4.5,
                            width: 55.0,
                            height: 1.5,
                            color: theme::SAFE_GREEN,
                        });
                        // r: valid portion
                        ctx.draw(&Rectangle {
                            x: 5.0,
                            y: 7.5,
                            width: 50.0,
                            height: 1.5,
                            color: theme::BORROW_YELLOW,
                        });
                        // r: dangling portion (past where s dropped)
                        ctx.draw(&Rectangle {
                            x: 55.0,
                            y: 7.5,
                            width: 40.0,
                            height: 1.5,
                            color: theme::CRAB_RED,
                        });
                        // drop marker
                        ctx.draw(&CanvasLine {
                            x1: 55.0,
                            y1: 0.0,
                            x2: 55.0,
                            y2: 12.0,
                            color: theme::CRAB_RED,
                        });
                        ctx.print(
                            56.0,
                            2.5,
                            Span::styled("s dropped!", Style::default().fg(theme::CRAB_RED)),
                        );
                        ctx.print(
                            57.0,
                            8.0,
                            Span::styled("dangling!", Style::default().fg(theme::CRAB_RED)),
                        );
                    }
                    3 => {
                        // Lifetime annotations: 'a spans both r and s
                        ctx.draw(&Rectangle {
                            x: 10.0,
                            y: 4.5,
                            width: 80.0,
                            height: 1.5,
                            color: theme::BORROW_YELLOW,
                        });
                        ctx.draw(&Rectangle {
                            x: 10.0,
                            y: 7.5,
                            width: 80.0,
                            height: 1.5,
                            color: theme::BORROW_YELLOW,
                        });
                        ctx.print(
                            20.0,
                            5.0,
                            Span::styled(
                                "'a (s: &'a str)",
                                Style::default().fg(theme::BORROW_YELLOW),
                            ),
                        );
                        ctx.print(
                            20.0,
                            8.0,
                            Span::styled(
                                "'a (r: &'a str)",
                                Style::default().fg(theme::BORROW_YELLOW),
                            ),
                        );
                        // Bracket showing 'a span
                        ctx.draw(&CanvasLine {
                            x1: 10.0,
                            y1: 1.5,
                            x2: 90.0,
                            y2: 1.5,
                            color: theme::RUST_ORANGE,
                        });
                        ctx.print(
                            40.0,
                            2.0,
                            Span::styled(
                                "'a outlives both",
                                Style::default().fg(theme::RUST_ORANGE),
                            ),
                        );
                    }
                    _ => {
                        // 'static: full program lifetime
                        ctx.draw(&Rectangle {
                            x: 0.0,
                            y: 5.0,
                            width: 100.0,
                            height: 2.0,
                            color: theme::ASYNC_PURPLE,
                        });
                        ctx.print(
                            20.0,
                            5.5,
                            Span::styled(
                                "'static — entire program",
                                Style::default().fg(theme::TEXT_PRIMARY),
                            ),
                        );
                        // Arrow showing extension beyond program end
                        ctx.draw(&CanvasLine {
                            x1: 98.0,
                            y1: 6.0,
                            x2: 101.0,
                            y2: 6.0,
                            color: theme::ASYNC_PURPLE,
                        });
                    }
                }

                // Now cursor (sweeps left to right with step timer)
                ctx.draw(&CanvasLine {
                    x1: cursor_x,
                    y1: 0.0,
                    x2: cursor_x,
                    y2: 12.0,
                    color: theme::RUST_ORANGE,
                });
                // x-axis ticks
                for tick in [0.0_f64, 25.0, 50.0, 75.0, 100.0] {
                    ctx.draw(&CanvasLine {
                        x1: tick,
                        y1: 0.0,
                        x2: tick,
                        y2: 0.5,
                        color: theme::TEXT_DIM,
                    });
                }
                ctx.draw(&CanvasLine {
                    x1: 0.0,
                    y1: 0.3,
                    x2: 100.0,
                    y2: 0.3,
                    color: theme::TEXT_DIM,
                });
            });

        frame.render_widget(gantt, mid[1]);

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
        "Lifetimes"
    }

    fn description(&self) -> &'static str {
        "Compile-time proof that references never outlive their data."
    }

    fn explanation(&self) -> &'static str {
        "Lifetimes are Rust's system for tracking how long references are valid. \
        The borrow checker uses lifetime annotations to prove that no reference outlives the data it points to. \
        This prevents the entire class of use-after-free bugs that plague C/C++ codebases — \
        at compile time, with zero runtime overhead."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.show_error = false;
        self.error_timer = 0.0;
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
            "What does `'a: 'b` mean in a lifetime bound?",
            [
                "'a equals 'b",
                "'a outlives 'b",
                "'b outlives 'a",
                "Both lifetimes are static",
            ],
            1,
        ))
    }

    fn supports_step_control(&self) -> bool {
        true
    }

    fn step_forward(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
    }

    fn step_back(&mut self) {
        self.step = (self.step + STEPS - 1) % STEPS;
        self.step_timer = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_scope_bracket_lines_depth_0() {
        let lines = scope_bracket_lines(0);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("outer scope"));
    }

    #[test]
    fn test_scope_bracket_lines_depth_1() {
        let lines = scope_bracket_lines(1);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("let s")));
    }

    #[test]
    fn test_scope_bracket_lines_depth_2() {
        let lines = scope_bracket_lines(2);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("let r")));
    }

    #[test]
    fn test_scope_bracket_lines_depth_3() {
        let lines = scope_bracket_lines(3);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("DANGLING")));
    }

    #[test]
    fn test_lifetime_annotation_example_nonempty() {
        let lines = lifetime_annotation_example();
        assert!(!lines.is_empty());
        // At least one annotated line
        assert!(lines.iter().any(|(_, annotated)| *annotated));
    }

    #[test]
    fn test_lifetime_annotation_has_lifetime_syntax() {
        let lines = lifetime_annotation_example();
        let combined: String = lines.iter().map(|(l, _)| *l).collect::<Vec<_>>().join("\n");
        assert!(combined.contains("'a"));
    }

    #[test]
    fn test_is_dangling_scenario_step_0() {
        assert!(!is_dangling_scenario(0));
    }

    #[test]
    fn test_is_dangling_scenario_step_1() {
        assert!(!is_dangling_scenario(1));
    }

    #[test]
    fn test_is_dangling_scenario_step_2() {
        assert!(is_dangling_scenario(2));
    }

    #[test]
    fn test_is_dangling_scenario_step_3() {
        assert!(!is_dangling_scenario(3));
    }

    #[test]
    fn test_is_dangling_scenario_step_4() {
        assert!(!is_dangling_scenario(4));
    }

    #[test]
    fn test_show_error_animation_on_step_2() {
        let mut d = LifetimesDemo::new();
        d.step = 2;
        // Initially show_error is false
        assert!(!d.show_error);
        // After one threshold tick, it should flip
        d.tick(Duration::from_secs_f64(0.6)); // > 0.5s at speed=1
        assert!(d.show_error);
        // Tick again — should flip back
        d.tick(Duration::from_secs_f64(0.6));
        assert!(!d.show_error);
    }

    #[test]
    fn test_show_error_not_set_on_other_steps() {
        let mut d = LifetimesDemo::new();
        d.step = 1;
        d.tick(Duration::from_secs_f64(1.0));
        assert!(!d.show_error);
    }

    #[test]
    fn test_demo_trait_methods() {
        let mut d = LifetimesDemo::new();
        assert_eq!(d.name(), "Lifetimes");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
        assert!(!d.is_paused());
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
        d.set_speed(5);
        assert_eq!(d.speed(), 5);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = LifetimesDemo::new();
        d.step = 3;
        d.tick_count = 99;
        d.show_error = true;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert!(!d.show_error);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused() {
        let mut d = LifetimesDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_advance_step_wraps() {
        let mut d = LifetimesDemo::new();
        d.step = STEPS - 1;
        d.advance_step();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = LifetimesDemo::new();
        d.tick(Duration::from_secs_f64(4.0));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = LifetimesDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_render_step2_with_error_flash() {
        let mut d = LifetimesDemo::new();
        d.step = 2;
        d.show_error = true;
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_default() {
        let d = LifetimesDemo::default();
        assert_eq!(d.step, 0);
        assert!(!d.show_error);
    }

    #[test]
    fn test_step_duration_secs() {
        let mut d = LifetimesDemo::new();
        d.set_speed(3);
        let dur = d.step_duration_secs();
        assert!((dur - 1.0).abs() < 1e-6);
    }
}
