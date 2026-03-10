use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

const STEPS: usize = 6;

// ─── StepInfo ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct StepInfo {
    pub title: &'static str,
    pub code: &'static [&'static str],
    pub explanation: &'static str,
}

// ─── CompileTimeDemo ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CompileTimeDemo {
    pub paused: bool,
    pub speed: u8,
    pub tick_count: u64,
    pub step: usize,
    pub step_timer: f64,
    pub bugs_caught_compile_time: u64,
    pub bugs_caught_runtime: u64,
    pub animation_tick: u64,
    /// Target that bugs_caught_compile_time counts up toward (animated).
    pub counter_target: u64,
}

impl CompileTimeDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            bugs_caught_compile_time: 0,
            bugs_caught_runtime: 0,
            animation_tick: 0,
            counter_target: 0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        2.5 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        // Bump target by 1, 2, or 3 deterministically based on step index
        let increment = ((self.step % 3) as u64) + 1;
        self.counter_target += increment;
        // bugs_caught_runtime is never incremented — zero runtime surprises
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
    }

    pub fn get_step_info(step: usize) -> StepInfo {
        match step % STEPS {
            0 => StepInfo {
                title: "const Evaluation",
                code: &[
                    "const MAX_SIZE: usize = 1024 * 1024;",
                    "const BUFFER: [u8; MAX_SIZE] = [0u8; MAX_SIZE];",
                    "",
                    "// Fully evaluated at compile time — zero runtime cost",
                    "const HEX_A: u8 = 0x41; // 65",
                    "const IS_POWER: bool = (MAX_SIZE & (MAX_SIZE - 1)) == 0;",
                ],
                explanation: "Rust evaluates const expressions entirely at compile time. \
                    Arrays sized by const, mathematical expressions, and struct literals using \
                    const values are all resolved before any binary is produced. The result is \
                    embedded directly in the .rodata section — no runtime computation at all.",
            },
            1 => StepInfo {
                title: "const fn",
                code: &[
                    "const fn factorial(n: u64) -> u64 {",
                    "    if n == 0 { 1 } else { n * factorial(n - 1) }",
                    "}",
                    "",
                    "const FACT_10: u64 = factorial(10); // 3628800 at compile time",
                    "const FACT_20: u64 = factorial(20); // 2432902008176640000",
                ],
                explanation: "const fn functions can be called at compile time when all arguments \
                    are constants. The compiler evaluates the entire call graph at build time and \
                    embeds the result in the binary. No function call overhead at runtime.",
            },
            2 => StepInfo {
                title: "static_assertions",
                code: &[
                    "use static_assertions::assert_eq_size;",
                    "",
                    "assert_eq_size!(u32, [u8; 4]);        // passes",
                    "assert_eq_size!(Option<&str>, usize);  // passes",
                    "// assert_eq_size!(u32, u64);          // COMPILE ERROR",
                    "// assert_eq_size!(bool, u16);         // COMPILE ERROR",
                ],
                explanation: "The static_assertions crate provides macros that emit compile errors \
                    when size, alignment, or trait constraints are violated. These checks run at \
                    zero runtime cost — a wrong size assumption is a build failure, not a test failure.",
            },
            3 => StepInfo {
                title: "Typestate Pattern",
                code: &[
                    "struct Connection<State>(PhantomData<State>);",
                    "struct Disconnected;",
                    "struct Connected;",
                    "impl Connection<Disconnected> {",
                    "    fn connect(self) -> Connection<Connected> { unimplemented!() }",
                    "}",
                ],
                explanation: "The typestate pattern encodes program state into the type system. \
                    Calling send() on a disconnected socket is a compile error, not a runtime panic. \
                    PhantomData<State> carries the state marker at zero memory cost.",
            },
            4 => StepInfo {
                title: "Type-Level Invariants",
                code: &[
                    "struct NonEmpty<T>(Vec<T>);",
                    "impl<T> NonEmpty<T> {",
                    "    fn new(first: T) -> Self { NonEmpty(vec![first]) }",
                    "    // first() is always safe — vec can never be empty",
                    "    fn first(&self) -> &T { &self.0[0] }",
                    "}",
                ],
                explanation: "Newtype wrappers enforce invariants at the type level. NonEmpty<T> is \
                    guaranteed non-empty by construction — the only way to create one is with at least \
                    one element. The compiler enforces this, so first() never panics.",
            },
            _ => StepInfo {
                title: "Summary: Compiler as Safeguard",
                code: &[
                    "// Entire bug categories caught at compile time:",
                    "// 1. Use-after-free      -> borrow checker error",
                    "// 2. Data race           -> Send/Sync trait check",
                    "// 3. Null dereference    -> Option<T> forces handling",
                    "// 4. Wrong state         -> typestate pattern",
                    "// 5. Size mismatch       -> static_assertions",
                ],
                explanation: "Rust's compiler is the most powerful safeguard in your toolbox. \
                    It catches entire categories of bugs before code ships — with zero runtime overhead. \
                    The type system, borrow checker, const evaluator, and trait solver work together \
                    to make many classes of bugs literally impossible to express.",
            },
        }
    }
}

fn code_line_style(l: &str) -> Style {
    if l.is_empty() {
        Style::default()
    } else if l.starts_with("//") {
        theme::dim_style()
    } else if l.starts_with("const ") || l.starts_with("const fn") {
        Style::default().fg(theme::ASYNC_PURPLE)
    } else if l.contains("COMPILE ERROR") || l.contains("error") {
        Style::default()
            .fg(theme::CRAB_RED)
            .add_modifier(Modifier::BOLD)
    } else if l.starts_with("struct ")
        || l.starts_with("impl ")
        || l.starts_with("use ")
        || l.starts_with("assert_eq_size!")
    {
        Style::default().fg(theme::BORROW_YELLOW)
    } else {
        Style::default().fg(theme::SAFE_GREEN)
    }
}

impl Default for CompileTimeDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for CompileTimeDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.animation_tick = self.animation_tick.wrapping_add(1);
        // Animate counter catching up to target
        if self.bugs_caught_compile_time < self.counter_target {
            self.bugs_caught_compile_time += 1;
        }
        self.step_timer += dt.as_secs_f64();
        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let pulse_char = match self.animation_tick % 4 {
            0 | 2 => "◉",
            _ => "●",
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(area);

        let info = Self::get_step_info(self.step);
        let title_text = format!("{} {}", pulse_char, info.title);

        frame.render_widget(
            Paragraph::new(Span::styled(
                title_text,
                Style::default()
                    .fg(theme::RUST_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .title("Compile-Time Guarantees")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::RUST_ORANGE)),
            ),
            chunks[0],
        );

        // Split main area left / right 50/50
        if self.step % STEPS == 3 {
            // Step 3 (Typestate): side-by-side invalid vs valid state panels
            let typestate_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let invalid_lines = vec![
                Line::from(Span::styled(
                    "// ✗ COMPILE ERROR — invalid usage:",
                    theme::dim_style(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "let c: Connection<Disconnected>;",
                    Style::default().fg(theme::CRAB_RED),
                )),
                Line::from(Span::styled(
                    "c.send();  // not defined on Disconnected",
                    Style::default()
                        .fg(theme::CRAB_RED)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// error[E0599]: no method `send`",
                    theme::dim_style(),
                )),
                Line::from(Span::styled(
                    "// must call .connect() first",
                    theme::dim_style(),
                )),
            ];
            frame.render_widget(
                Paragraph::new(invalid_lines).block(
                    Block::default()
                        .title("✗ Invalid State")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::CRAB_RED)),
                ),
                typestate_cols[0],
            );

            let valid_lines = vec![
                Line::from(Span::styled(
                    "// ✓ Correct — typestate enforces sequence:",
                    theme::dim_style(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "let c = Connection::<Disconnected>;",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "let c = c.connect();  // → Connected",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "c.send();  // ✓ proven valid at compile time",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// Zero overhead — PhantomData<State>",
                    theme::dim_style(),
                )),
            ];
            frame.render_widget(
                Paragraph::new(valid_lines).block(
                    Block::default()
                        .title("✓ Valid State")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SAFE_GREEN)),
                ),
                typestate_cols[1],
            );
        } else {
            let main_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // Left: code panel with syntax highlighting
            let code_lines: Vec<Line> = info
                .code
                .iter()
                .map(|l| Line::from(Span::styled(*l, code_line_style(l))))
                .collect();
            frame.render_widget(
                Paragraph::new(code_lines).block(
                    Block::default()
                        .title("Code")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SAFE_GREEN)),
                ),
                main_cols[0],
            );

            // Right: explanation text
            frame.render_widget(
                Paragraph::new(info.explanation)
                    .block(
                        Block::default()
                            .title("Explanation")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                    )
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                main_cols[1],
            );
        }

        // Stats bar
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  Bugs caught at compile time: ", theme::dim_style()),
                Span::styled(
                    format!("{}", self.bugs_caught_compile_time),
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("   Bugs caught at runtime: ", theme::dim_style()),
                Span::styled("0", Style::default().fg(theme::CRAB_RED)),
                Span::styled("   (Zero runtime surprises)", theme::dim_style()),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::STACK_CYAN)),
            ),
            chunks[2],
        );
    }

    fn name(&self) -> &'static str {
        "Compile-Time Guarantees"
    }

    fn description(&self) -> &'static str {
        "The compiler catches bugs before they ship — const eval, static asserts, typestate."
    }

    fn explanation(&self) -> &'static str {
        "Rust's compiler is a theorem prover, not just a syntax checker. \
        It verifies memory safety, thread safety, and user-defined invariants at compile time. \
        const expressions are evaluated before the binary exists. static_assertions catch size and \
        alignment mismatches at build time. The typestate pattern encodes program state into types, \
        so invalid state transitions are compile errors — not runtime panics. Newtype wrappers enforce \
        invariants by construction. The result: entire categories of bugs that plague C, C++, Java, \
        and Python codebases simply cannot exist in safe Rust — not because of a runtime check, but \
        because the compiler mathematically proves them impossible."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.animation_tick = 0;
        self.bugs_caught_compile_time = 0;
        self.bugs_caught_runtime = 0;
        self.counter_target = 0;
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
            "What is `const fn` in Rust used for?",
            [
                "Runtime optimization",
                "Functions evaluated at compile time",
                "Inline assembly",
                "Async functions",
            ],
            1,
        ))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_name_description_explanation() {
        let d = CompileTimeDemo::new();
        assert_eq!(d.name(), "Compile-Time Guarantees");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() {
        let d = CompileTimeDemo::new();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = CompileTimeDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = CompileTimeDemo::new();
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
        d.set_speed(5);
        assert_eq!(d.speed(), 5);
    }

    #[test]
    fn test_reset() {
        let mut d = CompileTimeDemo::new();
        d.step = 4;
        d.tick_count = 100;
        d.animation_tick = 42;
        d.bugs_caught_compile_time = 10;
        d.bugs_caught_runtime = 5;
        d.counter_target = 15;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.animation_tick, 0);
        assert_eq!(d.bugs_caught_compile_time, 0);
        assert_eq!(d.bugs_caught_runtime, 0);
        assert_eq!(d.counter_target, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused_no_change() {
        let mut d = CompileTimeDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.animation_tick, 0);
        assert_eq!(d.step, 0);
        assert_eq!(d.step_timer, 0.0);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = CompileTimeDemo::new();
        d.step_timer = d.step_duration_secs() - 0.001;
        d.tick(Duration::from_secs_f64(0.1));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_all_six_steps_have_info() {
        for i in 0..STEPS {
            let info = CompileTimeDemo::get_step_info(i);
            assert!(!info.title.is_empty(), "step {} title is empty", i);
            assert!(!info.code.is_empty(), "step {} code is empty", i);
            assert!(
                info.code.len() >= 3,
                "step {} code should have at least 3 lines",
                i
            );
            assert!(
                !info.explanation.is_empty(),
                "step {} explanation is empty",
                i
            );
        }
    }

    #[test]
    fn test_bugs_caught_increments_on_advance() {
        let mut d = CompileTimeDemo::new();
        let before = d.counter_target;
        d.advance_step();
        assert!(
            d.counter_target > before,
            "counter_target should increase on advance_step"
        );
    }

    #[test]
    fn test_runtime_bugs_always_zero() {
        let mut d = CompileTimeDemo::new();
        for _ in 0..12 {
            d.advance_step();
        }
        assert_eq!(d.bugs_caught_runtime, 0);
    }

    #[test]
    fn test_step_duration_varies_with_speed() {
        let mut d = CompileTimeDemo::new();
        d.set_speed(5);
        let dur = d.step_duration_secs();
        // 2.5 / 5 = 0.5
        assert!((dur - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = CompileTimeDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = CompileTimeDemo::default();
        assert_eq!(d.step, 0);
        assert_eq!(d.animation_tick, 0);
        assert_eq!(d.bugs_caught_compile_time, 0);
        assert_eq!(d.bugs_caught_runtime, 0);
        assert_eq!(d.counter_target, 0);
        assert!(!d.paused);
    }

    #[test]
    fn test_animation_tick_increments() {
        let mut d = CompileTimeDemo::new();
        d.tick(Duration::from_millis(10));
        assert_eq!(d.animation_tick, 1);
        d.tick(Duration::from_millis(10));
        assert_eq!(d.animation_tick, 2);
    }

    #[test]
    fn test_bugs_caught_animates_toward_target() {
        let mut d = CompileTimeDemo::new();
        d.advance_step(); // bumps counter_target by 1
        assert_eq!(d.bugs_caught_compile_time, 0);
        assert!(d.counter_target > 0);
        d.tick(Duration::from_millis(1));
        assert_eq!(d.bugs_caught_compile_time, 1); // one tick → one increment
        assert_eq!(d.bugs_caught_compile_time, d.counter_target.min(1));
    }

    #[test]
    fn test_syntax_highlight_comment() {
        let style = code_line_style("// This is a comment");
        assert_eq!(style, theme::dim_style());
    }

    #[test]
    fn test_syntax_highlight_const_keyword() {
        let style = code_line_style("const MAX: usize = 42;");
        assert_eq!(style, Style::default().fg(theme::ASYNC_PURPLE));
    }

    #[test]
    fn test_syntax_highlight_struct() {
        let style = code_line_style("struct Foo;");
        assert_eq!(style, Style::default().fg(theme::BORROW_YELLOW));
    }

    #[test]
    fn test_syntax_highlight_default() {
        let style = code_line_style("    some_value: 42,");
        assert_eq!(style, Style::default().fg(theme::SAFE_GREEN));
    }

    #[test]
    fn test_render_step3_split() {
        let mut d = CompileTimeDemo::new();
        d.step = 3; // typestate step → triggers the split layout
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }
}
