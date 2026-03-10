use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::{demos::Demo, theme};

#[derive(Debug)]
pub struct WasmDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub animation_frame: u8,
    frame_timer: f64,
}

impl WasmDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            animation_frame: 0,
            frame_timer: 0.0,
        }
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
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Pulsing title color based on animation_frame
        let pulse = if self.animation_frame % 2 == 0 {
            theme::ASYNC_PURPLE
        } else {
            theme::HEAP_BLUE
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                "WebAssembly — Compile Rust to WASM, run everywhere at near-native speed",
                Style::default().fg(pulse).add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(pulse)),
            ),
            chunks[0],
        );

        // Split body into four panels
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(chunks[1]);

        // Panel 1: Target triples
        let triples = wasm_target_triples();
        let triple_items: Vec<ListItem> = triples
            .iter()
            .map(|(triple, desc)| {
                ListItem::new(vec![
                    Line::from(Span::styled(*triple, Style::default().fg(theme::BORROW_YELLOW).add_modifier(Modifier::BOLD))),
                    Line::from(Span::styled(format!("  {}", desc), theme::dim_style())),
                    Line::from(""),
                ])
            })
            .collect();
        frame.render_widget(
            List::new(triple_items).block(
                Block::default()
                    .title("Target Triples")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORROW_YELLOW)),
            ),
            body[0],
        );

        // Panel 2: WASM section proportions as gauges
        let sections = wasm_section_proportions();
        let section_inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                sections
                    .iter()
                    .map(|_| Constraint::Length(2))
                    .chain(std::iter::once(Constraint::Min(0)))
                    .collect::<Vec<_>>(),
            )
            .split(body[1]);

        let section_block = Block::default()
            .title("WASM Sections")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::STACK_CYAN));
        frame.render_widget(section_block, body[1]);

        // Render section gauges inside the block (with 1-cell border offset)
        let inner_area = ratatui::layout::Rect {
            x: body[1].x + 1,
            y: body[1].y + 1,
            width: body[1].width.saturating_sub(2),
            height: body[1].height.saturating_sub(2),
        };
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
        // Suppress unused variable warning for section_inner
        let _ = section_inner;

        // Panel 3: Size comparison
        let sizes = size_comparison_table();
        let size_items: Vec<ListItem> = sizes
            .iter()
            .map(|(lang, size)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:25}", lang), theme::dim_style()),
                    Span::styled(*size, Style::default().fg(theme::SAFE_GREEN).add_modifier(Modifier::BOLD)),
                ]))
            })
            .collect();
        frame.render_widget(
            List::new(size_items).block(
                Block::default()
                    .title("Binary Sizes")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::SAFE_GREEN)),
            ),
            body[2],
        );

        // Panel 4: JS type mappings
        let mappings = js_type_mappings();
        let map_items: Vec<ListItem> = mappings
            .iter()
            .map(|(rust_ty, js_ty)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:20}", rust_ty), Style::default().fg(theme::RUST_ORANGE)),
                    Span::styled("→ ", theme::dim_style()),
                    Span::styled(*js_ty, Style::default().fg(theme::HEAP_BLUE)),
                ]))
            })
            .collect();
        frame.render_widget(
            List::new(map_items).block(
                Block::default()
                    .title("JS Type Mappings")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::HEAP_BLUE)),
            ),
            body[3],
        );
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
        d.reset();
        assert_eq!(d.animation_frame, 0);
        assert_eq!(d.tick_count, 0);
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
    }
}
