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

const STEPS: usize = 6;

#[derive(Debug)]
pub struct TypeSystemDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub selected_item: usize,
    item_timer: f64,
}

impl TypeSystemDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            selected_item: 0,
            item_timer: 0.0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        3.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.selected_item = 0;
        self.item_timer = 0.0;
    }

    pub fn advance_item(&mut self, max_items: usize) {
        if max_items > 0 {
            self.selected_item = (self.selected_item + 1) % max_items;
        }
        self.item_timer = 0.0;
    }
}

pub fn trait_tree_lines() -> Vec<&'static str> {
    vec![
        "trait Shape { fn area(&self) -> f64; }",
        "  ├── impl Shape for Circle    { fn area → π·r² }",
        "  ├── impl Shape for Rectangle { fn area → w·h  }",
        "  └── impl Shape for Triangle  { fn area → ½·b·h }",
        "",
        "fn print_area<T: Shape>(s: &T) {  // static dispatch",
        "    println!(\"{}\", s.area());    // monomorphized",
        "}                                  // zero overhead",
    ]
}

pub fn enum_arms() -> Vec<&'static str> {
    vec![
        "enum Shape {",
        "    Circle(f64),",
        "    Rectangle(f64, f64),",
        "    Triangle(f64, f64, f64),",
        "}",
        "match shape {",
        "    Shape::Circle(r)       => π * r * r,",
        "    Shape::Rectangle(w, h) => w * h,",
        "    Shape::Triangle(a,b,c) => heron(a,b,c),",
        "}  // exhaustive — compiler catches missed arms",
    ]
}

pub fn pattern_match_result(value: i32) -> &'static str {
    match value {
        x if x < 0 => "negative",
        0 => "zero",
        x if x > 100 => "big positive",
        _ => "normal positive",
    }
}

pub fn generic_bounds_lines() -> Vec<&'static str> {
    vec![
        "fn largest<T: PartialOrd>(list: &[T]) -> &T {",
        "    // Works for i32, f64, char, String...",
        "    // Compiler generates specialized version",
        "    // for each concrete type used.",
        "    // NO virtual dispatch. NO boxing.",
        "    let mut largest = &list[0];",
        "    for item in list {",
        "        if item > largest { largest = item; }",
        "    }",
        "    largest",
        "}",
    ]
}

pub fn newtype_lines() -> Vec<(&'static str, bool)> {
    vec![
        ("struct Meters(f64);", false),
        ("struct Feet(f64);", false),
        ("", false),
        ("let m = Meters(5.0);", false),
        ("let f = Feet(16.4);", false),
        ("", false),
        ("// m + f  ← COMPILE ERROR!", true),
        ("// type system prevents unit confusion", true),
        ("", false),
        ("let m2 = Meters(3.0);", false),
        ("let total = Meters(m.0 + m2.0);  // ✓ OK", false),
    ]
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/6: Traits — static interfaces with zero-overhead dispatch",
        1 => "Step 2/6: Generics — monomorphization, no runtime cost",
        2 => "Step 3/6: Enums as Sum Types (ADTs) — exhaustive matching",
        3 => "Step 4/6: Newtype Pattern — type-level unit safety",
        4 => "Step 5/6: Associated Types — type-level computation",
        _ => "Step 6/6: Pattern Matching — rich, exhaustive, compile-verified",
    }
}

fn step_explanation(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Traits are Rust's interfaces. Generic functions using trait bounds are monomorphized at compile time — the compiler generates a specialized copy for each concrete type. No vtable, no indirection, no overhead.",
        1 => "fn largest<T: PartialOrd> works for any T that can be compared. The compiler creates separate versions for i32, f64, char etc. You write once, get many efficient specializations.",
        2 => "Enums in Rust are algebraic data types (sum types). A match expression must handle every variant — the compiler enforces exhaustiveness. No forgotten cases.",
        3 => "The newtype pattern wraps a primitive in a struct to create a distinct type. Meters(f64) and Feet(f64) are different types — adding them together is a compile error.",
        4 => "Associated types let traits define placeholder types (e.g. Iterator::Item). This gives cleaner APIs than extra generic parameters.",
        _ => "Rust's match is extremely powerful: value matching, range guards (x if x > 100), destructuring, binding, and exhaustiveness checking all combined.",
    }
}

impl Default for TypeSystemDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for TypeSystemDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        self.item_timer += dt.as_secs_f64();

        if self.item_timer >= 0.6 / self.speed as f64 {
            let max = match self.step % STEPS {
                0 => trait_tree_lines().len(),
                1 => generic_bounds_lines().len(),
                2 => enum_arms().len(),
                3 => newtype_lines().len(),
                4 => 9, // 9 lines in associated types block
                _ => 4, // 4 scan values for pattern matching
            };
            self.advance_item(max);
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

        let lines: Vec<Line> = match self.step % STEPS {
            0 => {
                let tl = trait_tree_lines();
                let len = tl.len();
                tl.iter()
                    .enumerate()
                    .map(|(i, l)| {
                        let style = if i == self.selected_item % len {
                            Style::default()
                                .fg(theme::SAFE_GREEN)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme::dim_style()
                        };
                        Line::from(Span::styled(*l, style))
                    })
                    .collect()
            }
            1 => {
                let gl = generic_bounds_lines();
                let len = gl.len();
                gl.iter()
                    .enumerate()
                    .map(|(i, l)| {
                        let style = if i == self.selected_item % len {
                            Style::default()
                                .fg(theme::HEAP_BLUE)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme::dim_style()
                        };
                        Line::from(Span::styled(*l, style))
                    })
                    .collect()
            }
            2 => {
                let ea = enum_arms();
                let len = ea.len();
                ea.iter()
                    .enumerate()
                    .map(|(i, l)| {
                        let style = if i == self.selected_item % len {
                            Style::default()
                                .fg(theme::BORROW_YELLOW)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme::dim_style()
                        };
                        Line::from(Span::styled(*l, style))
                    })
                    .collect()
            }
            3 => {
                let nl = newtype_lines();
                let len = nl.len();
                nl.iter()
                    .enumerate()
                    .map(|(i, (l, highlighted))| {
                        let style = if i == self.selected_item % len {
                            Style::default()
                                .fg(theme::BORROW_YELLOW)
                                .add_modifier(Modifier::BOLD)
                        } else if *highlighted {
                            Style::default()
                                .fg(theme::CRAB_RED)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme::dim_style()
                        };
                        Line::from(Span::styled(*l, style))
                    })
                    .collect()
            }
            4 => {
                let assoc: &[(&str, bool)] = &[
                    ("trait Iterator {", false),
                    ("    type Item;", true),
                    ("    fn next(&mut self) -> Option<Self::Item>;", false),
                    ("}", false),
                    ("", false),
                    ("// For Vec<String>:", false),
                    ("//   type Item = String", true),
                    ("// For Vec<u32>:", false),
                    ("//   type Item = u32", true),
                ];
                let len = assoc.len();
                assoc
                    .iter()
                    .enumerate()
                    .map(|(i, (l, is_key))| {
                        let style = if i == self.selected_item % len {
                            Style::default()
                                .fg(theme::BORROW_YELLOW)
                                .add_modifier(Modifier::BOLD)
                        } else if *is_key {
                            Style::default().fg(theme::SAFE_GREEN)
                        } else {
                            theme::dim_style()
                        };
                        Line::from(Span::styled(*l, style))
                    })
                    .collect()
            }
            _ => {
                let scan_vals: [i32; 4] = [-100, 0, 25, 150];
                let active = self.selected_item % 4;
                scan_vals
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        let result = pattern_match_result(v);
                        let is_active = i == active;
                        let (val_style, res_style) = if is_active {
                            (
                                Style::default()
                                    .fg(theme::RUST_ORANGE)
                                    .add_modifier(Modifier::BOLD),
                                Style::default()
                                    .fg(theme::SAFE_GREEN)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            (theme::dim_style(), theme::dim_style())
                        };
                        Line::from(vec![
                            Span::styled(format!("  match {:5} => ", v), val_style),
                            Span::styled(result, res_style),
                        ])
                    })
                    .collect()
            }
        };

        // Split center: code left, dispatch diagram canvas right
        let center_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(chunks[1]);

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title("Type System Demo")
                    .borders(Borders::ALL),
            ),
            center_split[0],
        );

        // ── Type dispatch diagram Canvas ──────────────────────────────────────
        let active_idx = self.selected_item;
        let current_step = self.step % STEPS;

        let diagram = Canvas::default()
            .block(
                Block::default()
                    .title("Dispatch Diagram")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::HEAP_BLUE)),
            )
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .marker(ratatui::symbols::Marker::Braille)
            .paint(move |ctx| {
                match current_step {
                    0 => {
                        // Static dispatch fan: T:Shape → Circle, Rectangle, Triangle
                        let targets = [
                            (80.0_f64, 80.0_f64, "Circle", theme::SAFE_GREEN),
                            (80.0_f64, 50.0_f64, "Rectangle", theme::SAFE_GREEN),
                            (80.0_f64, 20.0_f64, "Triangle", theme::SAFE_GREEN),
                        ];
                        // Source box
                        ctx.draw(&Rectangle {
                            x: 5.0,
                            y: 42.0,
                            width: 24.0,
                            height: 16.0,
                            color: theme::BORROW_YELLOW,
                        });
                        ctx.print(
                            7.0,
                            50.0,
                            Span::styled("T: Shape", Style::default().fg(theme::BORROW_YELLOW)),
                        );

                        for (idx, &(tx, ty, label, _)) in targets.iter().enumerate() {
                            let line_color = if idx == active_idx % 3 {
                                theme::SAFE_GREEN
                            } else {
                                theme::TEXT_DIM
                            };
                            ctx.draw(&CanvasLine {
                                x1: 29.0,
                                y1: 50.0,
                                x2: tx,
                                y2: ty + 7.0,
                                color: line_color,
                            });
                            ctx.draw(&Rectangle {
                                x: tx - 1.0,
                                y: ty,
                                width: 20.0,
                                height: 10.0,
                                color: line_color,
                            });
                            ctx.print(
                                tx + 1.0,
                                ty + 4.0,
                                Span::styled(label, Style::default().fg(line_color)),
                            );
                        }
                        ctx.print(
                            25.0,
                            12.0,
                            Span::styled("zero overhead", Style::default().fg(theme::TEXT_DIM)),
                        );
                        ctx.print(
                            22.0,
                            6.0,
                            Span::styled("monomorphized", Style::default().fg(theme::TEXT_DIM)),
                        );
                    }
                    1 => {
                        // Monomorphization: 1 generic → 3 concrete copies
                        ctx.draw(&Rectangle {
                            x: 30.0,
                            y: 80.0,
                            width: 40.0,
                            height: 12.0,
                            color: theme::HEAP_BLUE,
                        });
                        ctx.print(
                            35.0,
                            86.0,
                            Span::styled("fn largest<T>", Style::default().fg(theme::HEAP_BLUE)),
                        );

                        let copies = [
                            (5.0_f64, 50.0_f64, "largest<i32>"),
                            (33.0_f64, 50.0_f64, "largest<f64>"),
                            (60.0_f64, 50.0_f64, "largest<char>"),
                        ];
                        for (cx, cy, label) in copies {
                            let inline_color = if cx == 5.0 && active_idx % 3 == 0
                                || cx == 33.0 && active_idx % 3 == 1
                                || cx == 60.0 && active_idx % 3 == 2
                            {
                                theme::SAFE_GREEN
                            } else {
                                theme::TEXT_DIM
                            };
                            ctx.draw(&CanvasLine {
                                x1: 50.0,
                                y1: 80.0,
                                x2: cx + 15.0,
                                y2: 65.0,
                                color: inline_color,
                            });
                            ctx.draw(&Rectangle {
                                x: cx,
                                y: cy,
                                width: 28.0,
                                height: 12.0,
                                color: inline_color,
                            });
                            ctx.print(
                                cx + 2.0,
                                cy + 5.0,
                                Span::styled(label, Style::default().fg(inline_color)),
                            );
                        }
                        ctx.print(
                            20.0,
                            8.0,
                            Span::styled(
                                "compile-time specialization",
                                Style::default().fg(theme::TEXT_DIM),
                            ),
                        );
                    }
                    2 => {
                        // Enum sum type: variants as stacked boxes
                        let variants = [
                            (20.0_f64, 75.0_f64, "Circle(f64)", theme::STACK_CYAN),
                            (20.0_f64, 55.0_f64, "Rectangle(f64,f64)", theme::STACK_CYAN),
                            (
                                20.0_f64,
                                35.0_f64,
                                "Triangle(f64,f64,f64)",
                                theme::STACK_CYAN,
                            ),
                        ];
                        ctx.draw(&Rectangle {
                            x: 5.0,
                            y: 30.0,
                            width: 90.0,
                            height: 62.0,
                            color: theme::TEXT_DIM,
                        });
                        ctx.print(
                            35.0,
                            94.0,
                            Span::styled("enum Shape", Style::default().fg(theme::BORROW_YELLOW)),
                        );
                        for (i, &(vx, vy, label, color)) in variants.iter().enumerate() {
                            let c = if i == active_idx % 3 {
                                color
                            } else {
                                theme::TEXT_DIM
                            };
                            ctx.draw(&Rectangle {
                                x: vx,
                                y: vy,
                                width: 60.0,
                                height: 12.0,
                                color: c,
                            });
                            ctx.print(
                                vx + 3.0,
                                vy + 5.0,
                                Span::styled(label, Style::default().fg(c)),
                            );
                        }
                        ctx.print(
                            10.0,
                            10.0,
                            Span::styled(
                                "exhaustive match required",
                                Style::default().fg(theme::BORROW_YELLOW),
                            ),
                        );
                    }
                    3 => {
                        // Newtype: Meters vs Feet — distinct boxes with X
                        ctx.draw(&Rectangle {
                            x: 5.0,
                            y: 55.0,
                            width: 35.0,
                            height: 20.0,
                            color: theme::SAFE_GREEN,
                        });
                        ctx.print(
                            10.0,
                            65.0,
                            Span::styled("Meters(f64)", Style::default().fg(theme::SAFE_GREEN)),
                        );

                        ctx.draw(&Rectangle {
                            x: 60.0,
                            y: 55.0,
                            width: 35.0,
                            height: 20.0,
                            color: theme::CRAB_RED,
                        });
                        ctx.print(
                            65.0,
                            65.0,
                            Span::styled("Feet(f64)", Style::default().fg(theme::CRAB_RED)),
                        );

                        // X mark between them
                        ctx.draw(&CanvasLine {
                            x1: 40.0,
                            y1: 57.0,
                            x2: 60.0,
                            y2: 73.0,
                            color: theme::CRAB_RED,
                        });
                        ctx.draw(&CanvasLine {
                            x1: 40.0,
                            y1: 73.0,
                            x2: 60.0,
                            y2: 57.0,
                            color: theme::CRAB_RED,
                        });
                        ctx.print(
                            20.0,
                            25.0,
                            Span::styled(
                                "type system prevents mixing",
                                Style::default().fg(theme::CRAB_RED),
                            ),
                        );
                        ctx.print(
                            15.0,
                            15.0,
                            Span::styled(
                                "compile error: Meters + Feet",
                                Style::default().fg(theme::TEXT_DIM),
                            ),
                        );
                    }
                    4 => {
                        // Associated types: Iterator → Item
                        ctx.draw(&Rectangle {
                            x: 10.0,
                            y: 75.0,
                            width: 35.0,
                            height: 16.0,
                            color: theme::BORROW_YELLOW,
                        });
                        ctx.print(
                            14.0,
                            83.0,
                            Span::styled("Iterator", Style::default().fg(theme::BORROW_YELLOW)),
                        );
                        ctx.print(
                            14.0,
                            78.0,
                            Span::styled("type Item", Style::default().fg(theme::SAFE_GREEN)),
                        );

                        // Concrete impls
                        let impls = [
                            (
                                55.0_f64,
                                80.0_f64,
                                "Vec<String>",
                                "Item=String",
                                theme::STACK_CYAN,
                            ),
                            (
                                55.0_f64,
                                55.0_f64,
                                "Vec<u32>",
                                "Item=u32",
                                theme::SAFE_GREEN,
                            ),
                            (
                                55.0_f64,
                                30.0_f64,
                                "Range<i32>",
                                "Item=i32",
                                theme::HEAP_BLUE,
                            ),
                        ];
                        for (i, &(ix, iy, iname, item, color)) in impls.iter().enumerate() {
                            let c = if i == active_idx % 3 {
                                color
                            } else {
                                theme::TEXT_DIM
                            };
                            ctx.draw(&CanvasLine {
                                x1: 45.0,
                                y1: 83.0,
                                x2: ix,
                                y2: iy + 7.0,
                                color: c,
                            });
                            ctx.draw(&Rectangle {
                                x: ix,
                                y: iy,
                                width: 38.0,
                                height: 14.0,
                                color: c,
                            });
                            ctx.print(
                                ix + 2.0,
                                iy + 9.0,
                                Span::styled(iname, Style::default().fg(c)),
                            );
                            ctx.print(
                                ix + 2.0,
                                iy + 3.0,
                                Span::styled(item, Style::default().fg(c)),
                            );
                        }
                    }
                    _ => {
                        // Pattern matching: decision tree
                        ctx.draw(&Rectangle {
                            x: 30.0,
                            y: 82.0,
                            width: 40.0,
                            height: 12.0,
                            color: theme::RUST_ORANGE,
                        });
                        ctx.print(
                            34.0,
                            88.0,
                            Span::styled("match value", Style::default().fg(theme::RUST_ORANGE)),
                        );

                        let arms = [
                            (5.0_f64, 62.0_f64, "negative", theme::CRAB_RED),
                            (35.0_f64, 62.0_f64, "zero", theme::BORROW_YELLOW),
                            (65.0_f64, 62.0_f64, "positive", theme::SAFE_GREEN),
                        ];
                        let active = active_idx % 3;
                        for (i, &(ax, ay, label, color)) in arms.iter().enumerate() {
                            let c = if i == active { color } else { theme::TEXT_DIM };
                            ctx.draw(&CanvasLine {
                                x1: 50.0,
                                y1: 82.0,
                                x2: ax + 15.0,
                                y2: 76.0,
                                color: c,
                            });
                            ctx.draw(&Rectangle {
                                x: ax,
                                y: ay,
                                width: 28.0,
                                height: 12.0,
                                color: c,
                            });
                            ctx.print(
                                ax + 3.0,
                                ay + 5.0,
                                Span::styled(label, Style::default().fg(c)),
                            );
                        }
                        ctx.print(
                            8.0,
                            15.0,
                            Span::styled(
                                "exhaustive — no missing arms",
                                Style::default().fg(theme::TEXT_DIM),
                            ),
                        );
                    }
                }
            });

        frame.render_widget(diagram, center_split[1]);

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
        "Type System"
    }

    fn description(&self) -> &'static str {
        "Generics, traits, enums as sum types — resolved at compile time."
    }

    fn explanation(&self) -> &'static str {
        "Rust's type system combines generics (monomorphized at compile time), traits (static or dynamic dispatch), \
        algebraic data types (enums as sum types), and exhaustive pattern matching. \
        There is no null, no untagged union, no missing case — the compiler proves your program handles all possibilities."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.selected_item = 0;
        self.item_timer = 0.0;
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
            "What is the key difference between static and dynamic dispatch in Rust?",
            [
                "Dynamic is faster",
                "Static monomorphizes at compile time",
                "Static requires vtables",
                "Dynamic prevents abstraction",
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
    fn test_trait_tree_lines_nonempty() {
        assert!(!trait_tree_lines().is_empty());
    }

    #[test]
    fn test_enum_arms_nonempty() {
        assert!(!enum_arms().is_empty());
    }

    #[test]
    fn test_pattern_match_negative() {
        assert_eq!(pattern_match_result(-5), "negative");
    }

    #[test]
    fn test_pattern_match_zero() {
        assert_eq!(pattern_match_result(0), "zero");
    }

    #[test]
    fn test_pattern_match_big() {
        assert_eq!(pattern_match_result(150), "big positive");
    }

    #[test]
    fn test_pattern_match_normal() {
        assert_eq!(pattern_match_result(50), "normal positive");
    }

    #[test]
    fn test_generic_bounds_nonempty() {
        assert!(!generic_bounds_lines().is_empty());
    }

    #[test]
    fn test_newtype_lines_nonempty() {
        assert!(!newtype_lines().is_empty());
    }

    #[test]
    fn test_step_titles_all_steps() {
        for i in 0..STEPS {
            assert!(!step_title(i).is_empty());
        }
    }

    #[test]
    fn test_step_explanations_all_steps() {
        for i in 0..STEPS {
            assert!(!step_explanation(i).is_empty());
        }
    }

    #[test]
    fn test_demo_trait_methods() {
        let mut d = TypeSystemDemo::new();
        assert_eq!(d.name(), "Type System");
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
        let mut d = TypeSystemDemo::new();
        d.step = 4;
        d.tick_count = 100;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused() {
        let mut d = TypeSystemDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_advance_step_wraps() {
        let mut d = TypeSystemDemo::new();
        d.step = STEPS - 1;
        d.advance_step();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_advance_item_max_zero() {
        let mut d = TypeSystemDemo::new();
        d.advance_item(0); // should not panic, selected stays 0
        assert_eq!(d.selected_item, 0);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = TypeSystemDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = TypeSystemDemo::default();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_step_duration_secs() {
        let mut d = TypeSystemDemo::new();
        d.set_speed(3);
        let dur = d.step_duration_secs();
        assert!((dur - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = TypeSystemDemo::new();
        // step_duration at speed=1 is 3.0s; tick with 4s should advance
        d.tick(Duration::from_secs_f64(4.0));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_tick_advances_item() {
        let mut d = TypeSystemDemo::new();
        // item_timer threshold at speed=1 is 0.6s
        d.tick(Duration::from_secs_f64(0.7));
        assert_eq!(d.selected_item, 1);
    }

    #[test]
    fn test_newtype_lines_has_error_entries() {
        let lines = newtype_lines();
        let highlighted: Vec<_> = lines.iter().filter(|(_, h)| *h).collect();
        assert!(!highlighted.is_empty());
    }

    #[test]
    fn test_selected_item_cycles_step1() {
        let mut d = TypeSystemDemo::new();
        d.step = 1;
        let max = generic_bounds_lines().len();
        d.selected_item = max - 1;
        d.advance_item(max);
        assert_eq!(d.selected_item, 0);
    }

    #[test]
    fn test_selected_item_cycles_step4() {
        let mut d = TypeSystemDemo::new();
        d.step = 4;
        d.selected_item = 8;
        d.advance_item(9);
        assert_eq!(d.selected_item, 0);
    }

    #[test]
    fn test_selected_item_cycles_step5() {
        let mut d = TypeSystemDemo::new();
        d.step = 5;
        d.selected_item = 3;
        d.advance_item(4);
        assert_eq!(d.selected_item, 0);
    }

    #[test]
    fn test_pattern_match_scan_vals() {
        assert_eq!(pattern_match_result(-100), "negative");
        assert_eq!(pattern_match_result(25), "normal positive");
        assert_eq!(pattern_match_result(0), "zero");
        assert_eq!(pattern_match_result(150), "big positive");
    }

    #[test]
    fn test_render_all_steps_with_cycling() {
        let mut d = TypeSystemDemo::new();
        for step in 0..STEPS {
            d.step = step;
            for sel in [0usize, 1, 3] {
                d.selected_item = sel;
                let backend = TestBackend::new(120, 30);
                let mut terminal = Terminal::new(backend).unwrap();
                terminal.draw(|f| d.render(f, f.area())).unwrap();
            }
        }
    }
}
