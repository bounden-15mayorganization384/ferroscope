use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Circle, Line as CanvasLine, Rectangle},
        Block, Borders, Paragraph,
    },
    Frame,
};
use std::time::Duration;

const STEPS: usize = 5;

#[derive(Debug)]
pub struct MacrosDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub anim: f64,
    pub expand_lines: usize,
    pub token_positions: Vec<f64>,
}

impl MacrosDemo {
    pub fn new() -> Self {
        let mut d = Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            anim: 0.0,
            expand_lines: 0,
            token_positions: (0..8).map(|i| -(i as f64) * 6.0).collect(),
        };
        d.reset_step_state();
        d
    }

    fn reset_step_state(&mut self) {
        self.anim = 0.0;
        self.expand_lines = 0;
        self.token_positions = (0..8).map(|i| -(i as f64) * 6.0).collect();
    }

    pub fn step_duration_secs(&self) -> f64 {
        5.0 / self.speed as f64
    }
}

impl Default for MacrosDemo {
    fn default() -> Self {
        Self::new()
    }
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/5: macro_rules! \x2014 pattern-based code generation",
        1 => "Step 2/5: vec![] expansion \x2014 source \x2192 generated code",
        2 => "Step 3/5: Macro Hygiene \x2014 no accidental name collisions",
        3 => "Step 4/5: Proc Macro Pipeline \x2014 TokenStream \x2192 TokenStream",
        _ => "Step 5/5: macro_rules! vs C macros \x2014 AST vs text substitution",
    }
}

fn step_explanation(step: usize) -> &'static str {
    match step % STEPS {
        0 => {
            "Declarative macros (macro_rules!) match patterns in your code and expand to Rust \
              syntax at compile time. They look like function calls but are much more powerful \
              -- they can generate arbitrary code, repeat patterns, and match on syntax trees."
        }
        1 => {
            "vec![] is a standard library macro. vec![1, 2, 3] expands to \
              { let mut v = Vec::new(); v.push(1); v.push(2); v.push(3); v } \
              -- zero runtime cost versus writing it by hand."
        }
        2 => {
            "Rust macros are hygienic: identifiers created inside a macro cannot \
              accidentally collide with identifiers in the calling code. $crate:: \
              ensures paths resolve in the macro'\''s crate, not the caller'\''s."
        }
        3 => {
            "Procedural macros receive a stream of Rust tokens, transform them, \
              and emit new tokens. This enables #[derive], attribute macros, and \
              function-like proc macros. The transformation is pure Rust code at compile time."
        }
        _ => {
            "C macros do raw text substitution before parsing -- dangerous and hard to debug. \
              Rust macros operate on parsed syntax trees, so they respect scoping, types, and \
              hygiene. The compiler shows macro expansion errors with full context."
        }
    }
}

impl Demo for MacrosDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        self.anim = (self.step_timer / self.step_duration_secs()).min(1.0);

        if self.step % STEPS == 1 {
            let target = (self.anim * 8.0) as usize;
            if target > self.expand_lines {
                self.expand_lines = target.min(8);
            }
        }

        if self.step % STEPS == 3 {
            for pos in &mut self.token_positions {
                *pos += dt.as_secs_f64() * self.speed as f64 * 15.0;
                if *pos > 105.0 {
                    *pos -= 130.0;
                }
            }
        }

        if self.step_timer >= self.step_duration_secs() {
            self.step = (self.step + 1) % STEPS;
            self.step_timer = 0.0;
            self.reset_step_state();
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

        let center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(chunks[1]);

        let code_lines = self.build_code_lines();
        frame.render_widget(
            Paragraph::new(code_lines)
                .block(Block::default().title("Macro Code").borders(Borders::ALL)),
            center[0],
        );

        let step = self.step % STEPS;
        let anim = self.anim;
        let expand_lines = self.expand_lines;
        let token_positions = self.token_positions.clone();
        let tick = self.tick_count;

        let diagram = Canvas::default()
            .block(
                Block::default()
                    .title("Expansion Diagram")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::ASYNC_PURPLE)),
            )
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .marker(ratatui::symbols::Marker::Braille)
            .paint(move |ctx| match step {
                0 => {
                    ctx.draw(&Rectangle {
                        x: 2.0,
                        y: 70.0,
                        width: 40.0,
                        height: 22.0,
                        color: theme::BORROW_YELLOW,
                    });
                    ctx.print(
                        5.0,
                        88.0,
                        Span::styled("Pattern:", Style::default().fg(theme::BORROW_YELLOW)),
                    );
                    ctx.print(
                        5.0,
                        82.0,
                        Span::styled(
                            "($x:expr, $y:expr)",
                            Style::default().fg(theme::TEXT_PRIMARY),
                        ),
                    );
                    ctx.print(
                        5.0,
                        76.0,
                        Span::styled("=> { $x + $y }", Style::default().fg(theme::TEXT_DIM)),
                    );
                    let arrow_x = 42.0 + anim * 12.0;
                    ctx.draw(&CanvasLine {
                        x1: 42.0,
                        y1: 81.0,
                        x2: arrow_x,
                        y2: 81.0,
                        color: theme::RUST_ORANGE,
                    });
                    if anim > 0.5 {
                        let alpha = ((anim - 0.5) * 2.0).min(1.0);
                        let ex = 58.0 + (1.0 - alpha) * 10.0;
                        ctx.draw(&Rectangle {
                            x: ex,
                            y: 70.0,
                            width: 36.0,
                            height: 22.0,
                            color: theme::SAFE_GREEN,
                        });
                        ctx.print(
                            ex + 3.0,
                            88.0,
                            Span::styled("Expanded:", Style::default().fg(theme::SAFE_GREEN)),
                        );
                        ctx.print(
                            ex + 3.0,
                            82.0,
                            Span::styled("add!(2, 3)", Style::default().fg(theme::TEXT_PRIMARY)),
                        );
                        ctx.print(
                            ex + 3.0,
                            76.0,
                            Span::styled("-> 2 + 3", Style::default().fg(theme::SAFE_GREEN)),
                        );
                    }
                    ctx.print(
                        5.0,
                        8.0,
                        Span::styled(
                            "expanded at compile time",
                            Style::default().fg(theme::TEXT_DIM),
                        ),
                    );
                }
                1 => {
                    ctx.draw(&Rectangle {
                        x: 2.0,
                        y: 80.0,
                        width: 38.0,
                        height: 14.0,
                        color: theme::BORROW_YELLOW,
                    });
                    ctx.print(
                        5.0,
                        89.0,
                        Span::styled("source:", Style::default().fg(theme::BORROW_YELLOW)),
                    );
                    ctx.print(
                        5.0,
                        83.0,
                        Span::styled("vec![1, 2, 3]", Style::default().fg(theme::TEXT_PRIMARY)),
                    );
                    let lines = [
                        ("let mut v = Vec::new();", theme::SAFE_GREEN),
                        ("v.push(1);", theme::STACK_CYAN),
                        ("v.push(2);", theme::STACK_CYAN),
                        ("v.push(3);", theme::STACK_CYAN),
                        ("v", theme::SAFE_GREEN),
                    ];
                    for (i, &(line, color)) in lines.iter().enumerate() {
                        if i < expand_lines {
                            let y = 68.0 - i as f64 * 11.0;
                            ctx.draw(&Rectangle {
                                x: 45.0,
                                y,
                                width: 50.0,
                                height: 9.0,
                                color,
                            });
                            ctx.print(
                                47.0,
                                y + 3.0,
                                Span::styled(line, Style::default().fg(color)),
                            );
                        }
                    }
                    ctx.print(
                        5.0,
                        5.0,
                        Span::styled(
                            "identical to hand-written",
                            Style::default().fg(theme::TEXT_DIM),
                        ),
                    );
                }
                2 => {
                    ctx.draw(&Rectangle {
                        x: 2.0,
                        y: 55.0,
                        width: 44.0,
                        height: 38.0,
                        color: theme::BORROW_YELLOW,
                    });
                    ctx.print(
                        5.0,
                        90.0,
                        Span::styled("Macro scope:", Style::default().fg(theme::BORROW_YELLOW)),
                    );
                    ctx.print(
                        5.0,
                        82.0,
                        Span::styled(
                            "let x = 42; // macro's",
                            Style::default().fg(theme::TEXT_DIM),
                        ),
                    );
                    ctx.print(
                        5.0,
                        66.0,
                        Span::styled("$crate::helper()", Style::default().fg(theme::STACK_CYAN)),
                    );
                    ctx.draw(&Rectangle {
                        x: 54.0,
                        y: 55.0,
                        width: 44.0,
                        height: 38.0,
                        color: theme::SAFE_GREEN,
                    });
                    ctx.print(
                        57.0,
                        90.0,
                        Span::styled("Call site:", Style::default().fg(theme::SAFE_GREEN)),
                    );
                    ctx.print(
                        57.0,
                        82.0,
                        Span::styled(
                            "let x = 7;  // yours",
                            Style::default().fg(theme::TEXT_PRIMARY),
                        ),
                    );
                    ctx.print(
                        57.0,
                        74.0,
                        Span::styled("my_macro!(...);", Style::default().fg(theme::TEXT_PRIMARY)),
                    );
                    ctx.print(
                        57.0,
                        66.0,
                        Span::styled("x still == 7 OK", Style::default().fg(theme::SAFE_GREEN)),
                    );
                    let shield_x = 49.0 + (tick % 40) as f64 * 0.05;
                    ctx.draw(&CanvasLine {
                        x1: shield_x,
                        y1: 55.0,
                        x2: shield_x,
                        y2: 93.0,
                        color: theme::RUST_ORANGE,
                    });
                    ctx.print(
                        15.0,
                        10.0,
                        Span::styled("hygiene barrier", Style::default().fg(theme::RUST_ORANGE)),
                    );
                }
                3 => {
                    let stages = [
                        (2.0_f64, "Source"),
                        (26.0, "Parse"),
                        (52.0, "Transform"),
                        (76.0, "Emit"),
                    ];
                    for &(x, label) in &stages {
                        ctx.draw(&Rectangle {
                            x,
                            y: 60.0,
                            width: 20.0,
                            height: 22.0,
                            color: theme::HEAP_BLUE,
                        });
                        ctx.print(
                            x + 2.0,
                            71.0,
                            Span::styled(label, Style::default().fg(theme::TEXT_PRIMARY)),
                        );
                    }
                    for &(x, _) in &stages[..3] {
                        ctx.draw(&CanvasLine {
                            x1: x + 20.0,
                            y1: 71.0,
                            x2: x + 26.0,
                            y2: 71.0,
                            color: theme::RUST_ORANGE,
                        });
                    }
                    for &pos in &token_positions {
                        if (0.0..=100.0).contains(&pos) {
                            ctx.draw(&Circle {
                                x: pos,
                                y: 71.0,
                                radius: 1.5,
                                color: theme::BORROW_YELLOW,
                            });
                        }
                    }
                    ctx.print(
                        2.0,
                        88.0,
                        Span::styled("TokenStream in", Style::default().fg(theme::TEXT_DIM)),
                    );
                    ctx.print(
                        60.0,
                        88.0,
                        Span::styled("TokenStream out", Style::default().fg(theme::SAFE_GREEN)),
                    );
                    ctx.print(
                        5.0,
                        8.0,
                        Span::styled(
                            "#[derive] uses proc macros",
                            Style::default().fg(theme::TEXT_DIM),
                        ),
                    );
                }
                _ => {
                    ctx.draw(&Rectangle {
                        x: 2.0,
                        y: 58.0,
                        width: 44.0,
                        height: 34.0,
                        color: theme::CRAB_RED,
                    });
                    ctx.print(
                        5.0,
                        89.0,
                        Span::styled("C Macro:", Style::default().fg(theme::CRAB_RED)),
                    );
                    ctx.print(
                        5.0,
                        81.0,
                        Span::styled("#define SQ(x) x*x", Style::default().fg(theme::TEXT_DIM)),
                    );
                    ctx.print(
                        5.0,
                        73.0,
                        Span::styled("SQ(1+2) -> 5  WRONG", Style::default().fg(theme::CRAB_RED)),
                    );
                    ctx.print(
                        5.0,
                        65.0,
                        Span::styled("text substitution", Style::default().fg(theme::CRAB_RED)),
                    );
                    ctx.draw(&Rectangle {
                        x: 54.0,
                        y: 58.0,
                        width: 44.0,
                        height: 34.0,
                        color: theme::SAFE_GREEN,
                    });
                    ctx.print(
                        57.0,
                        89.0,
                        Span::styled("Rust Macro:", Style::default().fg(theme::SAFE_GREEN)),
                    );
                    ctx.print(
                        57.0,
                        81.0,
                        Span::styled("macro_rules! sq", Style::default().fg(theme::TEXT_DIM)),
                    );
                    ctx.print(
                        57.0,
                        73.0,
                        Span::styled("sq!(1+2) -> 9  OK", Style::default().fg(theme::SAFE_GREEN)),
                    );
                    ctx.print(
                        57.0,
                        65.0,
                        Span::styled("AST-aware", Style::default().fg(theme::SAFE_GREEN)),
                    );
                }
            });

        frame.render_widget(diagram, center[1]);

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
        "Macros"
    }

    fn description(&self) -> &'static str {
        "Compile-time code generation -- declarative, procedural, hygienic."
    }

    fn explanation(&self) -> &'static str {
        "Rust macros are compile-time code generators that operate on syntax trees, \
        not raw text. macro_rules! matches patterns in your code and expands them to \
        valid Rust syntax. Procedural macros (#[derive], attribute macros) receive a \
        TokenStream and emit new code. All macros are hygienic -- no accidental name \
        collisions between macro-generated identifiers and caller code."
    }

    fn reset(&mut self) {
        self.tick_count = 0;
        self.step = 0;
        self.step_timer = 0.0;
        self.paused = false;
        self.reset_step_state();
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

    fn supports_step_control(&self) -> bool {
        true
    }

    fn step_forward(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.reset_step_state();
    }

    fn step_back(&mut self) {
        self.step = if self.step == 0 {
            STEPS - 1
        } else {
            self.step - 1
        };
        self.step_timer = 0.0;
        self.reset_step_state();
    }

    fn quiz(&self) -> Option<(&'static str, [&'static str; 4], usize)> {
        Some((
            "What does macro_rules! operate on?",
            [
                "Raw text before parsing",
                "Parsed syntax tree (AST)",
                "Binary machine code",
                "LLVM IR",
            ],
            1,
        ))
    }
}

impl MacrosDemo {
    fn build_code_lines(&self) -> Vec<Line<'static>> {
        match self.step % STEPS {
            0 => vec![
                Line::from(Span::styled(
                    "macro_rules! add {",
                    Style::default()
                        .fg(theme::RUST_ORANGE)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "    ($x:expr, $y:expr) => {",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "        $x + $y",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    };",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled("}", Style::default().fg(theme::RUST_ORANGE))),
                Line::from(""),
                Line::from(Span::styled("// Invocation:", theme::dim_style())),
                Line::from(Span::styled(
                    "let result = add!(2, 3);",
                    Style::default().fg(theme::TEXT_PRIMARY),
                )),
                Line::from(""),
                Line::from(Span::styled("// Expands to:", theme::dim_style())),
                Line::from(Span::styled(
                    "let result = 2 + 3;",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
            ],
            1 => vec![
                Line::from(Span::styled("// You write:", theme::dim_style())),
                Line::from(Span::styled(
                    "let v = vec![1, 2, 3];",
                    Style::default()
                        .fg(theme::BORROW_YELLOW)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled("// Compiler generates:", theme::dim_style())),
                Line::from(Span::styled(
                    "let v = {",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    let mut v = Vec::new();",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    v.push(1);",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    v.push(2);",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    v.push(3);",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    v",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled("};", Style::default().fg(theme::SAFE_GREEN))),
            ],
            2 => vec![
                Line::from(Span::styled(
                    "macro_rules! my_macro {",
                    Style::default().fg(theme::RUST_ORANGE),
                )),
                Line::from(Span::styled(
                    "    ($val:expr) => {{",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "        let x = $val * 2;",
                    Style::default().fg(theme::TEXT_DIM),
                )),
                Line::from(Span::styled(
                    "        $crate::print_val(x);",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    "    }}",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled("}", Style::default().fg(theme::RUST_ORANGE))),
                Line::from(""),
                Line::from(Span::styled(
                    "let x = 7; // caller's x",
                    Style::default().fg(theme::TEXT_PRIMARY),
                )),
                Line::from(Span::styled(
                    "my_macro!(3);",
                    Style::default().fg(theme::TEXT_PRIMARY),
                )),
                Line::from(Span::styled(
                    "assert_eq!(x, 7); // OK",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
            ],
            3 => vec![
                Line::from(Span::styled("// proc macro crate", theme::dim_style())),
                Line::from(Span::styled(
                    "#[proc_macro_derive(Hello)]",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "pub fn hello_derive(",
                    Style::default().fg(theme::RUST_ORANGE),
                )),
                Line::from(Span::styled(
                    "    input: TokenStream",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled(
                    ") -> TokenStream {",
                    Style::default().fg(theme::RUST_ORANGE),
                )),
                Line::from(Span::styled(
                    "    let ast = parse(input);",
                    Style::default().fg(theme::TEXT_DIM),
                )),
                Line::from(Span::styled(
                    "    generate_impl(&ast)",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled("}", Style::default().fg(theme::RUST_ORANGE))),
            ],
            _ => vec![
                Line::from(Span::styled(
                    "// C (UNSAFE)",
                    Style::default().fg(theme::CRAB_RED),
                )),
                Line::from(Span::styled(
                    "#define SQUARE(x) x*x",
                    Style::default().fg(theme::CRAB_RED),
                )),
                Line::from(Span::styled(
                    "SQUARE(1+2) // -> 5 WRONG",
                    Style::default().fg(theme::CRAB_RED),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "// Rust",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "macro_rules! square {",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled(
                    "    ($x:expr) => { ($x)*($x) };",
                    Style::default().fg(theme::STACK_CYAN),
                )),
                Line::from(Span::styled("}", Style::default().fg(theme::SAFE_GREEN))),
                Line::from(Span::styled(
                    "square!(1+2) // -> 9 OK",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_name_description_explanation() {
        let d = MacrosDemo::new();
        assert_eq!(d.name(), "Macros");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_initial_state() {
        let d = MacrosDemo::new();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = MacrosDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_clamped() {
        let mut d = MacrosDemo::new();
        d.set_speed(5);
        assert_eq!(d.speed(), 5);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_step_forward_wraps() {
        let mut d = MacrosDemo::new();
        d.step = STEPS - 1;
        d.step_forward();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_step_back_wraps() {
        let mut d = MacrosDemo::new();
        d.step = 0;
        d.step_back();
        assert_eq!(d.step, STEPS - 1);
    }

    #[test]
    fn test_supports_step_control() {
        assert!(MacrosDemo::new().supports_step_control());
    }

    #[test]
    fn test_quiz_present() {
        let d = MacrosDemo::new();
        let q = d.quiz();
        assert!(q.is_some());
        let (question, options, correct) = q.unwrap();
        assert!(!question.is_empty());
        assert_eq!(options.len(), 4);
        assert!(correct < 4);
    }

    #[test]
    fn test_reset() {
        let mut d = MacrosDemo::new();
        d.step = 3;
        d.tick_count = 100;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = MacrosDemo::new();
        d.set_speed(10);
        d.tick(Duration::from_secs_f64(0.6));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_tick_paused() {
        let mut d = MacrosDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = MacrosDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(160, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.step_forward();
        }
    }

    #[test]
    fn test_default() {
        let d = MacrosDemo::default();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_step3_token_positions_advance() {
        let mut d = MacrosDemo::new();
        d.step = 3;
        let initial: Vec<f64> = d.token_positions.clone();
        d.tick(Duration::from_secs_f64(0.05));
        assert_ne!(d.token_positions, initial);
    }
}
