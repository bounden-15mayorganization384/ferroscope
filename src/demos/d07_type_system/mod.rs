use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::{demos::Demo, theme};

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
                _ => 1,
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
            1 => generic_bounds_lines()
                .iter()
                .map(|l| Line::from(Span::styled(*l, theme::dim_style())))
                .collect(),
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
            3 => newtype_lines()
                .iter()
                .map(|(l, highlighted)| {
                    let style = if *highlighted {
                        Style::default()
                            .fg(theme::CRAB_RED)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        theme::dim_style()
                    };
                    Line::from(Span::styled(*l, style))
                })
                .collect(),
            4 => vec![
                Line::from(Span::styled("trait Iterator {", theme::dim_style())),
                Line::from(Span::styled(
                    "    type Item;",
                    Style::default()
                        .fg(theme::BORROW_YELLOW)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "    fn next(&mut self) -> Option<Self::Item>;",
                    theme::dim_style(),
                )),
                Line::from(Span::styled("}", theme::dim_style())),
                Line::from(""),
                Line::from(Span::styled("// For Vec<String>:", theme::dim_style())),
                Line::from(Span::styled(
                    "//   type Item = String",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(Span::styled("// For Vec<u32>:", theme::dim_style())),
                Line::from(Span::styled(
                    "//   type Item = u32",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
            ],
            _ => {
                let vals = [-5i32, 0, 150, 50];
                vals.iter()
                    .map(|&v| {
                        let result = pattern_match_result(v);
                        Line::from(vec![
                            Span::styled(
                                format!("  match {:4} => ", v),
                                theme::dim_style(),
                            ),
                            Span::styled(
                                result,
                                Style::default()
                                    .fg(theme::SAFE_GREEN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])
                    })
                    .collect()
            }
        };

        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default().title("Type System Demo").borders(Borders::ALL)),
            chunks[1],
        );

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
}
