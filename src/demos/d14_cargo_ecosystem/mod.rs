use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::{demos::Demo, theme};

const STEPS: usize = 6;

// ─── CargoDemo ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CargoDemo {
    pub paused: bool,
    pub speed: u8,
    pub tick_count: u64,
    pub step: usize,
    pub step_timer: f64,
    /// Animates the highlighted line in the dep tree; wraps 0..=7
    pub dep_tree_frame: usize,
    /// Build progress 0.0..=1.0; resets to 0.0 on each new step
    pub build_progress: f64,
}

impl CargoDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            dep_tree_frame: 0,
            build_progress: 0.0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        3.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.build_progress = 0.0;
    }
}

// ─── Public data functions ────────────────────────────────────────────────────

/// Six example crates with a short purpose description.
pub fn crate_examples() -> &'static [(&'static str, &'static str)] {
    &[
        ("serde", "Serialization / deserialization framework"),
        ("tokio", "Async runtime with multi-threading"),
        ("reqwest", "Ergonomic HTTP client"),
        ("rayon", "Data parallelism via work-stealing"),
        ("clap", "Command-line argument parser"),
        ("axum", "Ergonomic async web framework"),
    ]
}

/// Five feature-flag examples with on/off state.
pub fn feature_flag_examples() -> &'static [(&'static str, bool)] {
    &[
        ("serde/derive", true),
        ("tokio/full", true),
        ("reqwest/json", true),
        ("clap/derive", false),
        ("axum/ws", false),
    ]
}

/// ASCII art dependency tree (6-8 lines).
pub fn dep_tree_lines() -> &'static [&'static str] {
    &[
        "ferroscope v0.1.0",
        "├── ratatui v0.28.0",
        "│   └── crossterm v0.27.0",
        "├── tokio v1.40.0",
        "│   ├── tokio-macros v2.4.0",
        "│   └── mio v1.0.2",
        "├── serde v1.0.213",
        "└── sysinfo v0.31.4",
    ]
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/6: cargo new — zero boilerplate",
        1 => "Step 2/6: Cargo.toml — dependencies as code",
        2 => "Step 3/6: Dependency Tree (cargo tree)",
        3 => "Step 4/6: Feature Flags",
        4 => "Step 5/6: cargo build — compilation pipeline",
        _ => "Step 6/6: The Ecosystem: crates.io",
    }
}

impl Default for CargoDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for CargoDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.dep_tree_frame = (self.dep_tree_frame + 1) % 8;
        self.build_progress = (self.build_progress + 0.02 * self.speed as f64).min(1.0);
        self.step_timer += dt.as_secs_f64();
        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        frame.render_widget(
            Paragraph::new(Span::styled(
                step_title(self.step),
                Style::default()
                    .fg(theme::RUST_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .title("Cargo Ecosystem")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::RUST_ORANGE)),
            ),
            chunks[0],
        );

        // Content — varies by step
        match self.step % STEPS {
            0 => {
                let lines = vec![
                    Line::from(Span::styled(
                        "$ cargo new my_project",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "    Created binary (application) `my_project`",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("my_project/", theme::dim_style())),
                    Line::from(Span::styled(
                        "├── Cargo.toml    ← dependencies, metadata, profiles",
                        Style::default().fg(theme::HEAP_BLUE),
                    )),
                    Line::from(Span::styled("└── src/", theme::dim_style())),
                    Line::from(Span::styled(
                        "    └── main.rs   ← fn main() { println!(\"Hello, world!\") }",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Zero configuration. Immediately ready to build, test, and publish.",
                        theme::dim_style(),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("cargo new")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::SAFE_GREEN)),
                    ),
                    chunks[1],
                );
            }
            1 => {
                let lines = vec![
                    Line::from(Span::styled(
                        "[package]",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        "name    = \"my_project\"",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "version = \"0.1.0\"",
                        theme::dim_style(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "[dependencies]",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(vec![
                        Span::styled("serde  = ", theme::dim_style()),
                        Span::styled(
                            "\"1.0\"",
                            Style::default().fg(theme::HEAP_BLUE),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("tokio  = { version = ", theme::dim_style()),
                        Span::styled("\"1\"", Style::default().fg(theme::HEAP_BLUE)),
                        Span::styled(", features = [", theme::dim_style()),
                        Span::styled(
                            "\"full\"",
                            Style::default().fg(theme::BORROW_YELLOW),
                        ),
                        Span::styled("] }", theme::dim_style()),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "reqwest = { version = ",
                            theme::dim_style(),
                        ),
                        Span::styled(
                            "\"0.12\"",
                            Style::default().fg(theme::HEAP_BLUE),
                        ),
                        Span::styled(", features = [", theme::dim_style()),
                        Span::styled(
                            "\"json\"",
                            Style::default().fg(theme::BORROW_YELLOW),
                        ),
                        Span::styled("] }", theme::dim_style()),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "# Cargo.lock pins every transitive dep — fully reproducible builds",
                        Style::default().fg(theme::SAFE_GREEN),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("Cargo.toml")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                    ),
                    chunks[1],
                );
            }
            2 => {
                // Dep tree with animated highlighted line
                let tree_lines = dep_tree_lines();
                let highlighted = self.dep_tree_frame % tree_lines.len();
                let items: Vec<ListItem> = tree_lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        let style = if i == highlighted {
                            Style::default()
                                .fg(theme::RUST_ORANGE)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme::dim_style()
                        };
                        ListItem::new(Line::from(Span::styled(*line, style)))
                    })
                    .collect();
                frame.render_widget(
                    List::new(items).block(
                        Block::default()
                            .title("cargo tree")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::STACK_CYAN)),
                    ),
                    chunks[1],
                );
            }
            3 => {
                // Feature flag toggles
                let flags = feature_flag_examples();
                let items: Vec<ListItem> = flags
                    .iter()
                    .map(|(name, enabled)| {
                        let (status, color) = if *enabled {
                            ("[ON] ", theme::SAFE_GREEN)
                        } else {
                            ("[OFF]", theme::CRAB_RED)
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                status,
                                Style::default()
                                    .fg(color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("  {}", name),
                                theme::label_style(),
                            ),
                        ]))
                    })
                    .collect();
                frame.render_widget(
                    List::new(items).block(
                        Block::default()
                            .title("Feature Flags")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::ASYNC_PURPLE)),
                    ),
                    chunks[1],
                );
            }
            4 => {
                // Build progress gauge
                let label = format!(
                    "cargo build --release  [{:.0}%]",
                    self.build_progress * 100.0
                );
                frame.render_widget(
                    Gauge::default()
                        .block(
                            Block::default()
                                .title("Compilation Pipeline")
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(theme::HEAP_BLUE)),
                        )
                        .gauge_style(Style::default().fg(theme::HEAP_BLUE))
                        .label(label)
                        .ratio(self.build_progress),
                    chunks[1],
                );
            }
            _ => {
                // crates.io ecosystem
                let examples = crate_examples();
                let items: Vec<ListItem> = examples
                    .iter()
                    .map(|(name, purpose)| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:12}", name),
                                Style::default()
                                    .fg(theme::HEAP_BLUE)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled("  — ", theme::dim_style()),
                            Span::styled(
                                *purpose,
                                Style::default().fg(theme::BORROW_YELLOW),
                            ),
                        ]))
                    })
                    .collect();
                frame.render_widget(
                    List::new(items).block(
                        Block::default()
                            .title("crates.io — Popular Crates")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::HEAP_BLUE)),
                    ),
                    chunks[1],
                );
            }
        }

        // Stats bar
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  crates.io: ", theme::dim_style()),
                Span::styled(
                    "150,000+ crates",
                    Style::default()
                        .fg(theme::HEAP_BLUE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("    downloads: ", theme::dim_style()),
                Span::styled(
                    "50B+",
                    Style::default()
                        .fg(theme::BORROW_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("    build: ", theme::dim_style()),
                Span::styled(
                    "reproducible & hermetic",
                    Style::default().fg(theme::SAFE_GREEN),
                ),
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
        "Cargo Ecosystem"
    }

    fn description(&self) -> &'static str {
        "Rust's build system and package manager — reproducible, fast, integrated."
    }

    fn explanation(&self) -> &'static str {
        "Cargo is Rust's built-in build system and package manager. \
        A single Cargo.toml declares all dependencies, feature flags, build profiles, and metadata. \
        Cargo.lock pins every transitive dependency to an exact version, making builds fully \
        reproducible across machines and time. crates.io hosts over 150,000 open-source crates \
        downloaded more than 50 billion times. Feature flags let you opt into optional functionality \
        without binary bloat. cargo build, cargo test, cargo doc, cargo bench, and cargo publish are \
        all first-class commands — no Makefiles, no CMake, no configuration sprawl."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.dep_tree_frame = 0;
        self.build_progress = 0.0;
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

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_name_description_explanation() {
        let d = CargoDemo::new();
        assert_eq!(d.name(), "Cargo Ecosystem");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() {
        let d = CargoDemo::new();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = CargoDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = CargoDemo::new();
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
        d.set_speed(7);
        assert_eq!(d.speed(), 7);
    }

    #[test]
    fn test_reset() {
        let mut d = CargoDemo::new();
        d.step = 3;
        d.tick_count = 50;
        d.dep_tree_frame = 5;
        d.build_progress = 0.75;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.dep_tree_frame, 0);
        assert_eq!(d.build_progress, 0.0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused_no_change() {
        let mut d = CargoDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.dep_tree_frame, 0);
        assert_eq!(d.build_progress, 0.0);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = CargoDemo::new();
        d.step_timer = d.step_duration_secs() - 0.001;
        d.tick(Duration::from_secs_f64(0.1));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_dep_tree_frame_increments() {
        let mut d = CargoDemo::new();
        d.tick(Duration::from_millis(10));
        assert_eq!(d.dep_tree_frame, 1);
    }

    #[test]
    fn test_build_progress_increments() {
        let mut d = CargoDemo::new();
        d.tick(Duration::from_millis(10));
        assert!(d.build_progress > 0.0);
    }

    #[test]
    fn test_build_progress_clamped_at_1() {
        let mut d = CargoDemo::new();
        d.build_progress = 0.99;
        // Many ticks should not push past 1.0
        for _ in 0..20 {
            d.tick(Duration::from_millis(10));
        }
        assert!(d.build_progress <= 1.0);
    }

    #[test]
    fn test_crate_examples_nonempty() {
        let examples = crate_examples();
        assert!(!examples.is_empty());
        assert_eq!(examples.len(), 6);
    }

    #[test]
    fn test_feature_flag_examples_nonempty() {
        let flags = feature_flag_examples();
        assert!(!flags.is_empty());
        assert_eq!(flags.len(), 5);
    }

    #[test]
    fn test_dep_tree_lines_nonempty() {
        let lines = dep_tree_lines();
        assert!(!lines.is_empty());
        assert!(lines.len() >= 6);
    }

    #[test]
    fn test_all_six_steps() {
        for i in 0..STEPS {
            let title = step_title(i);
            assert!(!title.is_empty(), "step {} title is empty", i);
            assert!(
                title.contains(&format!("{}/6", i + 1)),
                "step {} title missing step number",
                i
            );
        }
    }

    #[test]
    fn test_step_duration_varies_with_speed() {
        let mut d = CargoDemo::new();
        d.set_speed(3);
        let dur = d.step_duration_secs();
        // 3.0 / 3 = 1.0
        assert!((dur - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = CargoDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = CargoDemo::default();
        assert_eq!(d.step, 0);
        assert_eq!(d.build_progress, 0.0);
        assert_eq!(d.dep_tree_frame, 0);
    }
}
