use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

#[derive(Debug)]
pub struct WasmDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub animation_frame: u8,
    frame_timer: f64,
    pub step: usize,
    step_timer: f64,
}

impl WasmDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            animation_frame: 0,
            frame_timer: 0.0,
            step: 0,
            step_timer: 0.0,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        4.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % 4;
        self.step_timer = 0.0;
    }
}

/// The three primary Rust WASM compilation targets.
pub fn wasm_target_triples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("wasm32-unknown-unknown", "Browser, no std"),
        ("wasm32-wasi", "WASI runtimes (wasmtime, wasmer)"),
        ("wasm32-unknown-emscripten", "Emscripten toolchain"),
    ]
}

/// Proportions of sections in a typical WASM binary (sum to 1.0).
pub fn wasm_section_proportions() -> Vec<(&'static str, f64)> {
    vec![
        ("type", 0.08),
        ("import", 0.05),
        ("function", 0.12),
        ("table", 0.02),
        ("memory", 0.03),
        ("export", 0.07),
        ("code", 0.55),
        ("data", 0.08),
    ]
}

/// Binary size comparison between Rust WASM and other languages.
#[allow(dead_code)]
pub fn size_comparison_table() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Rust (release + strip)", "~35 KB"),
        ("C/Emscripten", "~60 KB"),
        ("Go", "~2.3 MB"),
        ("Java (GraalVM native)", "~6 MB"),
        ("Python (Pyodide)", "~25 MB"),
    ]
}

/// Rust to JavaScript type mappings via wasm-bindgen.
pub fn js_type_mappings() -> Vec<(&'static str, &'static str)> {
    vec![
        ("u32 / i32 / f64", "number"),
        ("bool", "boolean"),
        ("String / &str", "string"),
        ("JsValue", "any"),
        ("Vec<u8>", "Uint8Array"),
        ("Option<T>", "T | undefined"),
        ("Result<T,E>", "T (throws on Err)"),
    ]
}

/// Numeric binary sizes in KB for BarChart comparison.
pub fn size_comparison_kb() -> &'static [(&'static str, u64)] {
    &[
        ("Rust", 35),
        ("C/Emscr.", 60),
        ("Go", 2_300),
        ("Java", 6_000),
        ("Python", 25_000),
    ]
}

fn step_title(step: usize) -> &'static str {
    match step % 4 {
        0 => "Step 1/4: Compilation Targets — wasm32 target triples",
        1 => "Step 2/4: Binary Layout — WASM section proportions",
        2 => "Step 3/4: Binary Size — Rust vs other languages",
        _ => "Step 4/4: JS Bridge — wasm-bindgen type mappings",
    }
}

impl Default for WasmDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for WasmDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.frame_timer += dt.as_secs_f64();

        // Advance animation_frame at roughly 2 fps scaled by speed
        if self.frame_timer >= 0.5 / self.speed as f64 {
            self.animation_frame = (self.animation_frame + 1) % 60;
            self.frame_timer = 0.0;
        }

        self.step_timer += dt.as_secs_f64();
        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Pulsing title color based on animation_frame
        let pulse = if self.animation_frame.is_multiple_of(2) {
            theme::ASYNC_PURPLE
        } else {
            theme::HEAP_BLUE
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                step_title(self.step),
                Style::default().fg(pulse).add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(pulse)),
            ),
            chunks[0],
        );

        match self.step % 4 {
            0 => {
                // Step 1: Target triples
                let triples = wasm_target_triples();
                let triple_items: Vec<ListItem> = triples
                    .iter()
                    .map(|(triple, desc)| {
                        ListItem::new(vec![
                            Line::from(Span::styled(
                                *triple,
                                Style::default()
                                    .fg(theme::BORROW_YELLOW)
                                    .add_modifier(Modifier::BOLD),
                            )),
                            Line::from(Span::styled(format!("  {}", desc), theme::dim_style())),
                            Line::from(""),
                        ])
                    })
                    .collect();
                let cmd_line = Line::from(vec![
                    Span::styled("$ cargo build --target ", theme::dim_style()),
                    Span::styled(
                        "wasm32-unknown-unknown",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" --release", theme::dim_style()),
                ]);
                let inner = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(8), Constraint::Length(3)])
                    .split(chunks[1]);
                frame.render_widget(
                    List::new(triple_items).block(
                        Block::default()
                            .title("Target Triples")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                    ),
                    inner[0],
                );
                frame.render_widget(
                    Paragraph::new(cmd_line).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::STACK_CYAN)),
                    ),
                    inner[1],
                );
            }
            1 => {
                // Step 2: WASM section proportions as gauges
                let sections = wasm_section_proportions();
                let section_block = Block::default()
                    .title("WASM Binary Sections")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::STACK_CYAN));
                let inner_area = ratatui::layout::Rect {
                    x: chunks[1].x + 1,
                    y: chunks[1].y + 1,
                    width: chunks[1].width.saturating_sub(2),
                    height: chunks[1].height.saturating_sub(2),
                };
                frame.render_widget(section_block, chunks[1]);
                let gauge_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        sections
                            .iter()
                            .map(|_| Constraint::Length(1))
                            .chain(std::iter::once(Constraint::Min(0)))
                            .collect::<Vec<_>>(),
                    )
                    .split(inner_area);
                for (i, (name, ratio)) in sections.iter().enumerate() {
                    if i < gauge_layout.len().saturating_sub(1) {
                        let color = if *name == "code" {
                            theme::SAFE_GREEN
                        } else {
                            theme::STACK_CYAN
                        };
                        let label = format!("{:8} {:4.0}%", name, ratio * 100.0);
                        frame.render_widget(
                            Gauge::default()
                                .gauge_style(Style::default().fg(color))
                                .label(label)
                                .ratio(ratio.clamp(0.0, 1.0)),
                            gauge_layout[i],
                        );
                    }
                }
            }
            2 => {
                // Step 3: BarChart binary size comparison
                let sizes = size_comparison_kb();
                let max_kb = sizes.iter().map(|(_, kb)| *kb).max().unwrap_or(1);
                let bars: Vec<Bar> = sizes
                    .iter()
                    .map(|(lang, kb)| {
                        let color = if *lang == "Rust" {
                            theme::SAFE_GREEN
                        } else {
                            theme::TEXT_DIM
                        };
                        Bar::default()
                            .value(*kb)
                            .label(Line::from(*lang))
                            .style(Style::default().fg(color))
                    })
                    .collect();
                let group = BarGroup::default().bars(&bars);
                let arch = std::env::consts::ARCH;
                let os = std::env::consts::OS;
                let title = format!("Binary Size (KB) — platform: {arch}/{os}");
                frame.render_widget(
                    BarChart::default()
                        .data(group)
                        .bar_width(10)
                        .bar_gap(2)
                        .max(max_kb)
                        .block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(theme::SAFE_GREEN)),
                        ),
                    chunks[1],
                );
            }
            _ => {
                // Step 4: JS type mappings + wasm-bindgen overview
                let mappings = js_type_mappings();
                let map_items: Vec<ListItem> = mappings
                    .iter()
                    .map(|(rust_ty, js_ty)| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:20}", rust_ty),
                                Style::default().fg(theme::RUST_ORANGE),
                            ),
                            Span::styled("→ ", theme::dim_style()),
                            Span::styled(*js_ty, Style::default().fg(theme::HEAP_BLUE)),
                        ]))
                    })
                    .collect();
                let arch = std::env::consts::ARCH;
                let os = std::env::consts::OS;
                let footer_text = format!(
                    "  wasm-bindgen marshals Rust types to/from JS — platform: {arch}/{os}"
                );
                let inner = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(8), Constraint::Length(3)])
                    .split(chunks[1]);
                frame.render_widget(
                    List::new(map_items).block(
                        Block::default()
                            .title("JS Type Mappings (wasm-bindgen)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::HEAP_BLUE)),
                    ),
                    inner[0],
                );
                frame.render_widget(
                    Paragraph::new(Span::styled(footer_text, theme::dim_style())).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::STACK_CYAN)),
                    ),
                    inner[1],
                );
            }
        }
    }

    fn name(&self) -> &'static str {
        "WebAssembly"
    }

    fn description(&self) -> &'static str {
        "Compile Rust to WASM — run in any browser at near-native speed."
    }

    fn explanation(&self) -> &'static str {
        "Rust has first-class WebAssembly support via wasm-bindgen and wasm-pack. \
        The same Rust code that runs natively can run in Chrome, Firefox, or Node.js at near-native speed, \
        with a tiny binary size (~35 KB for a release build). \
        WASI extends this to server-side WASM runtimes like wasmtime and wasmer."
    }

    fn reset(&mut self) {
        self.animation_frame = 0;
        self.frame_timer = 0.0;
        self.tick_count = 0;
        self.step = 0;
        self.step_timer = 0.0;
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
    fn test_wasm_target_triples_len() {
        assert_eq!(wasm_target_triples().len(), 3);
    }

    #[test]
    fn test_wasm_target_triples_content() {
        let triples = wasm_target_triples();
        assert_eq!(triples[0].0, "wasm32-unknown-unknown");
        assert_eq!(triples[1].0, "wasm32-wasi");
        assert_eq!(triples[2].0, "wasm32-unknown-emscripten");
    }

    #[test]
    fn test_wasm_section_proportions_sum() {
        let sections = wasm_section_proportions();
        let sum: f64 = sections.iter().map(|(_, v)| v).sum();
        assert!((sum - 1.0).abs() < 1e-9, "sum = {}", sum);
    }

    #[test]
    fn test_wasm_section_proportions_nonempty() {
        assert!(!wasm_section_proportions().is_empty());
    }

    #[test]
    fn test_size_comparison_table_nonempty() {
        let table = size_comparison_table();
        assert!(!table.is_empty());
        assert!(table.len() >= 3);
    }

    #[test]
    fn test_size_comparison_has_rust() {
        let table = size_comparison_table();
        assert!(table.iter().any(|(lang, _)| lang.contains("Rust")));
    }

    #[test]
    fn test_js_type_mappings_nonempty() {
        let mappings = js_type_mappings();
        assert!(!mappings.is_empty());
        assert!(mappings.len() >= 4);
    }

    #[test]
    fn test_js_type_mappings_has_string() {
        let mappings = js_type_mappings();
        assert!(mappings.iter().any(|(_, js)| *js == "string"));
    }

    #[test]
    fn test_animation_frame_wraps_at_60() {
        let mut d = WasmDemo::new();
        d.animation_frame = 59;
        d.frame_timer = 10.0;
        d.tick(Duration::from_micros(1));
        assert_eq!(d.animation_frame, 0);
    }

    #[test]
    fn test_animation_frame_advances() {
        let mut d = WasmDemo::new();
        // 0.6s > 0.5s threshold at speed=1
        d.tick(Duration::from_secs_f64(0.6));
        assert_eq!(d.animation_frame, 1);
    }

    #[test]
    fn test_tick_paused() {
        let mut d = WasmDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.animation_frame, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_demo_trait_methods() {
        let mut d = WasmDemo::new();
        assert_eq!(d.name(), "WebAssembly");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
        assert!(!d.is_paused());
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
        d.set_speed(4);
        assert_eq!(d.speed(), 4);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = WasmDemo::new();
        d.animation_frame = 42;
        d.tick_count = 999;
        d.step = 3;
        d.reset();
        assert_eq!(d.animation_frame, 0);
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.step, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_render() {
        let d = WasmDemo::new();
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_different_frames() {
        let mut d = WasmDemo::new();
        for frame in [0u8, 1, 30, 59] {
            d.animation_frame = frame;
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
        }
    }

    #[test]
    fn test_default() {
        let d = WasmDemo::default();
        assert_eq!(d.animation_frame, 0);
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = WasmDemo::new();
        for step in 0..4 {
            d.step = step;
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
        }
    }

    #[test]
    fn test_advance_step_wraps() {
        let mut d = WasmDemo::new();
        d.step = 3;
        d.advance_step();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_step_duration_varies_with_speed() {
        let mut d = WasmDemo::new();
        d.set_speed(2);
        assert!((d.step_duration_secs() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = WasmDemo::new();
        // step_duration at speed=1 is 4.0s
        d.tick(Duration::from_secs_f64(4.1));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_size_comparison_kb_has_rust() {
        let sizes = size_comparison_kb();
        assert!(!sizes.is_empty());
        assert!(sizes.iter().any(|(lang, _)| *lang == "Rust"));
    }

    #[test]
    fn test_size_comparison_kb_rust_smallest() {
        let sizes = size_comparison_kb();
        let rust_kb = sizes.iter().find(|(l, _)| *l == "Rust").unwrap().1;
        for (lang, kb) in sizes {
            if *lang != "Rust" {
                assert!(
                    rust_kb < *kb,
                    "Rust ({rust_kb} KB) should be smaller than {lang} ({kb} KB)"
                );
            }
        }
    }

    #[test]
    fn test_step_titles_all_steps() {
        for i in 0..4 {
            assert!(!step_title(i).is_empty());
            assert!(step_title(i).contains(&format!("{}/4", i + 1)));
        }
    }
}
