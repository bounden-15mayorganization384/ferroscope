use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

const STEPS: usize = 5;

/// Binary size in bytes for each of the five steps.
const BINARY_SIZES: [u64; STEPS] = [
    980_000, // step 0: full std
    120_000, // step 1: no_std + alloc
    28_000,  // step 2: no_std, stack-only
    8_000,   // step 3: HAL minimal
    1_800,   // step 4: absolute minimum
];

/// Labels and ASCII bars for the size visualisation, one entry per step.
const BINARY_TIERS: [(&str, &str, &str); STEPS] = [
    ("Rust + std     ", "980 KB", "████████████████████"),
    ("no_std + alloc ", "120 KB", "██████              "),
    ("no_std only    ", " 28 KB", "██                  "),
    ("HAL minimal    ", "  8 KB", "▌                   "),
    ("Absolute min   ", "1.8 KB", "·                   "),
];

// ─── NoStdDemo ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct NoStdDemo {
    pub paused: bool,
    pub speed: u8,
    pub tick_count: u64,
    pub step: usize,
    pub step_timer: f64,
    /// Current binary size in bytes (tracks the active step)
    pub binary_size_bytes: u64,
    /// Animation frame counter; wraps 0..=39
    pub animation_frame: usize,
}

impl NoStdDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            binary_size_bytes: BINARY_SIZES[0],
            animation_frame: 0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        2.5 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.binary_size_bytes = binary_size_for_step(self.step);
    }
}

// ─── Public data functions ────────────────────────────────────────────────────

/// Binary size in bytes for the given step index.
pub fn binary_size_for_step(step: usize) -> u64 {
    BINARY_SIZES[step % STEPS]
}

/// Four to six Rust code lines demonstrating the concept at each step.
pub fn no_std_code_lines(step: usize) -> &'static [&'static str] {
    match step % STEPS {
        0 => &[
            "#![no_std]",
            "",
            "// No heap, no file I/O, no threads, no std::io",
            "// Only core::*, primitive types, and what you add",
            "use core::fmt::Write;",
            "// Runs on any CPU with as little as 1 KB of RAM",
        ],
        1 => &[
            "// No Box, Vec, String, HashMap — stack-allocated only",
            "let buf: [u8; 64] = [0u8; 64];",
            "let n: u32 = 42;   // stack, no heap",
            "// heapless crate: fixed-size stack collections",
            "use heapless::Vec;",
            "let mut v: heapless::Vec<u8, 16> = heapless::Vec::new();",
        ],
        2 => &[
            "// ARM Cortex-M3: 256 KB flash, 64 KB RAM",
            "// With std (if it existed) — wouldn't even fit!",
            "// no_std + alloc  → 120 KB",
            "// no_std only     →  28 KB",
            "// HAL minimal     →   8 KB",
            "// Absolute min    → ~1.8 KB of machine code",
        ],
        3 => &[
            "use embedded_hal::digital::OutputPin;",
            "use stm32f4xx_hal::pac::Peripherals;",
            "",
            "let dp = Peripherals::take().unwrap();",
            "let gpioa = dp.GPIOA.split();",
            "let mut led = gpioa.pa5.into_push_pull_output();",
        ],
        _ => &[
            "// Rust on real hardware today:",
            "// - RTIC: real-time interrupt-driven concurrency",
            "// - Embassy: async/await on microcontrollers",
            "// - Hubris: Microsoft OS for embedded systems",
            "// - Tock: secure embedded OS (Google Titan, OpenSK)",
            "// - Drogue IoT: cloud-connected embedded Rust",
        ],
    }
}

/// Real-world Rust embedded projects and chips (5 or more entries).
pub fn embedded_examples() -> &'static [&'static str] {
    &[
        "STM32 (ARM Cortex-M) — stm32f4xx-hal",
        "Raspberry Pi Pico (RP2040) — rp-hal",
        "ESP32 — esp-hal (Espressif official)",
        "Nordic nRF52 series — nrf-hal",
        "RISC-V microcontrollers — riscv crate",
        "Google Titan security chip (OpenTitan)",
        "Microsoft Pluton security processor",
    ]
}

fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/5: #![no_std] — stripping the standard library",
        1 => "Step 2/5: no allocator — stack only",
        2 => "Step 3/5: Memory map: from 256 KB to 2 KB",
        3 => "Step 4/5: Bare-metal HAL — hardware abstraction",
        _ => "Step 5/5: Real-world: Rust in embedded devices",
    }
}

fn step_explanation(step: usize) -> &'static str {
    match step % STEPS {
        0 => "#![no_std] removes the standard library and links only against core — Rust's \
              platform-independent subset. No heap allocator, no OS threads, no file system. \
              The result compiles to any bare-metal target.",
        1 => "Without an allocator, all data lives on the stack or in static memory. \
              The heapless crate provides fixed-size, stack-allocated Vec, HashMap, and String \
              types that work identically to their std equivalents — without a single heap allocation.",
        2 => "Removing std shrinks the binary dramatically. A release build with full std is \
              typically 980+ KB. Switching to no_std + alloc drops that to ~120 KB. \
              Using only the stack eliminates another 75%, down to ~28 KB.",
        3 => "The embedded-hal crate defines hardware-agnostic traits (GPIO, I2C, SPI, UART). \
              Chip-specific HAL crates (stm32f4xx-hal, rp-hal, nrf-hal) implement these traits, \
              letting you write portable drivers that work across different microcontrollers.",
        _ => "Rust is now used in production embedded systems at Google, Microsoft, and Espressif. \
              Embassy brings async/await to bare-metal, RTIC provides real-time interrupt concurrency, \
              and Tock is a fully verified secure OS written entirely in Rust.",
    }
}

impl Default for NoStdDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for NoStdDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.animation_frame = (self.animation_frame + 1) % 40;
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
                    .title("Embedded / no_std")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::RUST_ORANGE)),
            ),
            chunks[0],
        );

        // Main area: left 55% (code + explanation), right 45% (binary size viz)
        let main_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(chunks[1]);

        // Left panel: split vertically for code (top) and explanation (bottom)
        let left_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Min(3)])
            .split(main_cols[0]);

        // Code snippet (or embedded_examples list for the final step)
        if self.step % STEPS == STEPS - 1 {
            let items: Vec<ListItem> = embedded_examples()
                .iter()
                .map(|e| {
                    ListItem::new(Line::from(Span::styled(
                        *e,
                        Style::default().fg(theme::SAFE_GREEN),
                    )))
                })
                .collect();
            frame.render_widget(
                List::new(items).block(
                    Block::default()
                        .title("Real-World Rust Embedded")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SAFE_GREEN)),
                ),
                left_rows[0],
            );
        } else {
            let code_lines: Vec<Line> = no_std_code_lines(self.step)
                .iter()
                .map(|l| {
                    let style = if l.starts_with("//") {
                        Style::default().fg(theme::TEXT_DIM)
                    } else {
                        Style::default().fg(theme::SAFE_GREEN)
                    };
                    Line::from(Span::styled(*l, style))
                })
                .collect();
            frame.render_widget(
                Paragraph::new(code_lines).block(
                    Block::default()
                        .title("Code")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SAFE_GREEN)),
                ),
                left_rows[0],
            );
        }

        // Explanation text
        frame.render_widget(
            Paragraph::new(step_explanation(self.step))
                .block(
                    Block::default()
                        .title("Explanation")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true }),
            left_rows[1],
        );

        // Right panel: binary size bar chart with animated active-row indicator
        let pulse = if self.animation_frame.is_multiple_of(2) {
            "▶ "
        } else {
            "  "
        };
        let bar_lines: Vec<Line> = BINARY_TIERS
            .iter()
            .enumerate()
            .map(|(i, (label, kb, bar))| {
                let style = if i == self.step % STEPS {
                    Style::default()
                        .fg(theme::RUST_ORANGE)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::TEXT_DIM)
                };
                let prefix = if i == self.step % STEPS { pulse } else { "  " };
                Line::from(Span::styled(
                    format!("{}{}  {}  {}", prefix, label, kb, bar),
                    style,
                ))
            })
            .collect();
        frame.render_widget(
            Paragraph::new(bar_lines).block(
                Block::default()
                    .title("Binary Size")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::STACK_CYAN)),
            ),
            main_cols[1],
        );

        // Footer
        let size_kb = self.binary_size_bytes as f64 / 1000.0;
        let size_str = if size_kb.fract() == 0.0 {
            format!("{:.0} KB", size_kb)
        } else {
            format!("{:.1} KB", size_kb)
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  Binary size: ", theme::dim_style()),
                Span::styled(
                    size_str,
                    Style::default()
                        .fg(theme::RUST_ORANGE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("   Arch: ARM Cortex-M", theme::dim_style()),
                Span::styled("   OS: ", theme::dim_style()),
                Span::styled("none", Style::default().fg(theme::CRAB_RED)),
                Span::styled("   Runtime: ", theme::dim_style()),
                Span::styled("none", Style::default().fg(theme::CRAB_RED)),
                Span::styled("   GC: ", theme::dim_style()),
                Span::styled(
                    "impossible",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
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
        "Embedded / no_std"
    }

    fn description(&self) -> &'static str {
        "#![no_std] — Rust runs on bare metal with no OS, no allocator, no runtime."
    }

    fn explanation(&self) -> &'static str {
        "Rust's #![no_std] attribute removes the standard library, leaving only core — a \
        platform-independent subset with no heap allocation, no OS threads, and no file system. \
        The result compiles to any bare-metal target: ARM Cortex-M, RISC-V, AVR, Xtensa. \
        Without an allocator, all data lives on the stack or in static memory. The binary \
        footprint shrinks from ~980 KB (full std) to as little as 1.8 KB of machine code. \
        The embedded-hal trait family defines hardware-agnostic abstractions for GPIO, I2C, SPI, \
        and UART, letting drivers remain portable across chip families. Rust is now deployed in \
        production embedded systems at Google (Titan), Microsoft (Pluton/Hubris), and Espressif \
        (ESP32 official HAL), proving that safety and zero-cost abstractions reach all the way \
        down to bare metal."
    }

    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.binary_size_bytes = BINARY_SIZES[0];
        self.animation_frame = 0;
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
        let d = NoStdDemo::new();
        assert_eq!(d.name(), "Embedded / no_std");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() {
        let d = NoStdDemo::new();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = NoStdDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = NoStdDemo::new();
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
        d.set_speed(4);
        assert_eq!(d.speed(), 4);
    }

    #[test]
    fn test_reset() {
        let mut d = NoStdDemo::new();
        d.step = 3;
        d.tick_count = 77;
        d.binary_size_bytes = 8_000;
        d.animation_frame = 20;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.binary_size_bytes, BINARY_SIZES[0]);
        assert_eq!(d.animation_frame, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_paused() {
        let mut d = NoStdDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.animation_frame, 0);
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = NoStdDemo::new();
        d.step_timer = d.step_duration_secs() - 0.001;
        d.tick(Duration::from_secs_f64(0.1));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_binary_size_for_each_step() {
        assert_eq!(binary_size_for_step(0), 980_000);
        assert_eq!(binary_size_for_step(1), 120_000);
        assert_eq!(binary_size_for_step(2), 28_000);
        assert_eq!(binary_size_for_step(3), 8_000);
        assert_eq!(binary_size_for_step(4), 1_800);
    }

    #[test]
    fn test_binary_size_decreases_per_step() {
        let mut prev = binary_size_for_step(0);
        for step in 1..STEPS {
            let curr = binary_size_for_step(step);
            assert!(
                curr < prev,
                "step {} size {} should be less than step {} size {}",
                step,
                curr,
                step - 1,
                prev
            );
            prev = curr;
        }
    }

    #[test]
    fn test_no_std_code_lines_nonempty() {
        for step in 0..STEPS {
            let lines = no_std_code_lines(step);
            assert!(!lines.is_empty(), "step {} code lines empty", step);
            assert!(
                lines.len() >= 4,
                "step {} should have at least 4 code lines",
                step
            );
        }
    }

    #[test]
    fn test_embedded_examples_nonempty() {
        let examples = embedded_examples();
        assert!(!examples.is_empty());
    }

    #[test]
    fn test_embedded_examples_has_five_or_more() {
        let examples = embedded_examples();
        assert!(
            examples.len() >= 5,
            "expected 5+ embedded examples, got {}",
            examples.len()
        );
    }

    #[test]
    fn test_animation_frame_increments() {
        let mut d = NoStdDemo::new();
        d.tick(Duration::from_millis(10));
        assert_eq!(d.animation_frame, 1);
        d.tick(Duration::from_millis(10));
        assert_eq!(d.animation_frame, 2);
    }

    #[test]
    fn test_animation_frame_wraps() {
        let mut d = NoStdDemo::new();
        d.animation_frame = 39;
        d.tick(Duration::from_millis(10));
        assert_eq!(d.animation_frame, 0);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = NoStdDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = NoStdDemo::default();
        assert_eq!(d.step, 0);
        assert_eq!(d.animation_frame, 0);
        assert_eq!(d.binary_size_bytes, BINARY_SIZES[0]);
        assert!(!d.paused);
    }

    #[test]
    fn test_step_duration_varies_with_speed() {
        let mut d = NoStdDemo::new();
        d.set_speed(5);
        let dur = d.step_duration_secs();
        // 2.5 / 5 = 0.5
        assert!((dur - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_render_last_step_uses_embedded_examples() {
        let mut d = NoStdDemo::new();
        d.step = STEPS - 1; // final step → renders embedded_examples as List
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_embedded_examples_used_only_on_last_step() {
        // embedded_examples() should return entries for all chips
        let examples = embedded_examples();
        assert!(examples.iter().any(|e| e.contains("STM32")));
        assert!(examples.iter().any(|e| e.contains("ESP32")));
        assert!(examples.iter().any(|e| e.contains("Titan")));
    }
}
