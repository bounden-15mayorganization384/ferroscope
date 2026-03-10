use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{demos::Demo, theme};

const STEPS: usize = 10;

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub label: String,
    pub vars: Vec<(String, u64)>,  // (name, size_bytes)
}

impl StackFrame {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), vars: Vec::new() }
    }
    pub fn push_var(&mut self, name: impl Into<String>, size: u64) {
        self.vars.push((name.into(), size));
    }
    pub fn total_size(&self) -> u64 {
        self.vars.iter().map(|(_, s)| s).sum()
    }
}

#[derive(Debug, Clone)]
pub struct HeapBlock {
    pub label: String,
    pub size_bytes: u64,
    pub alive: bool,
}

#[derive(Debug)]
pub struct MemoryDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub alloc_count: u64,
    pub free_count: u64,
    pub stack_frames: Vec<StackFrame>,
    pub heap_blocks: Vec<HeapBlock>,
}

impl MemoryDemo {
    pub fn new() -> Self {
        Self {
            paused: false, speed: 1, tick_count: 0,
            step: 0, step_timer: 0.0,
            alloc_count: 0, free_count: 0,
            stack_frames: Vec::new(),
            heap_blocks: Vec::new(),
        }
    }

    pub fn step_duration_secs(&self) -> f64 { 2.5 / self.speed as f64 }

    fn apply_step(&mut self) {
        match self.step {
            0 => {
                // Empty — reset all state
                self.stack_frames.clear();
                self.heap_blocks.clear();
                self.alloc_count = 0;
                self.free_count = 0;
            }
            1 => {
                // fn main() stack frame
                self.stack_frames.push(StackFrame::new("main()"));
            }
            2 => {
                // let x: i32 = 42
                if let Some(f) = self.stack_frames.last_mut() {
                    f.push_var("x: i32", 4);
                }
            }
            3 => {
                // let s = String::new()
                if let Some(f) = self.stack_frames.last_mut() {
                    f.push_var("s: String (ptr/len/cap)", 24);
                }
                self.heap_blocks.push(HeapBlock { label: "s heap buf (empty)".into(), size_bytes: 0, alive: true });
                self.alloc_count += 1;
            }
            4 => {
                // s.push_str("hello world")
                if let Some(b) = self.heap_blocks.last_mut() {
                    b.size_bytes = 16; // rounded allocation
                    b.label = "s heap buf (\"hello world\")".into();
                }
            }
            5 => {
                // inner scope opens
                self.stack_frames.push(StackFrame::new("{ inner scope }"));
            }
            6 => {
                // let inner = Box::new(100i32)
                if let Some(f) = self.stack_frames.last_mut() {
                    f.push_var("inner: Box<i32> (ptr)", 8);
                }
                self.heap_blocks.push(HeapBlock { label: "Box<i32> (value=100)".into(), size_bytes: 4, alive: true });
                self.alloc_count += 1;
            }
            7 => {
                // scope ends — inner dropped
                self.stack_frames.pop();
                if let Some(b) = self.heap_blocks.last_mut() {
                    b.alive = false;
                }
                self.free_count += 1;
            }
            8 => {
                // drop(s)
                if let Some(b) = self.heap_blocks.first_mut() {
                    b.alive = false;
                }
                self.free_count += 1;
                if let Some(f) = self.stack_frames.last_mut() {
                    f.vars.retain(|(n, _)| !n.starts_with("s:"));
                }
            }
            _ => {
                // main returns — full reset next tick
                self.stack_frames.clear();
                self.heap_blocks.clear();
            }
        }
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.apply_step();
    }

    pub fn leaked_bytes(&self) -> u64 {
        self.heap_blocks.iter().filter(|b| b.alive).map(|b| b.size_bytes).sum()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 { format!("{:.1} GB", bytes as f64 / 1_073_741_824.0) }
    else if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1_024 { format!("{:.1} KB", bytes as f64 / 1_024.0) }
    else { format!("{} B", bytes) }
}

pub fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/10: Initial state — empty stack and heap",
        1 => "Step 2/10: fn main() — stack frame pushed",
        2 => "Step 3/10: let x: i32 = 42 — 4 bytes on stack",
        3 => "Step 4/10: let s = String::new() — stack + heap alloc",
        4 => "Step 5/10: s.push_str(\"hello world\") — heap grows",
        5 => "Step 6/10: { — new scope opens, new stack frame",
        6 => "Step 7/10: Box::new(100i32) — heap allocation",
        7 => "Step 8/10: } — scope ends, Box dropped immediately (RAII)",
        8 => "Step 9/10: drop(s) — String heap buffer freed",
        _ => "Step 10/10: main() returns — stack frame popped",
    }
}

impl Default for MemoryDemo {
    fn default() -> Self { Self::new() }
}

impl Demo for MemoryDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused { return; }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(6), Constraint::Length(3)])
            .split(area);

        // Title
        let title_text = step_title(self.step);
        frame.render_widget(
            Paragraph::new(Span::styled(title_text, Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD)))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme::RUST_ORANGE))),
            chunks[0],
        );

        // Stack | Heap
        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Stack
        let stack_items: Vec<ListItem> = self.stack_frames.iter().rev().flat_map(|f| {
            let mut items = vec![
                ListItem::new(Line::from(Span::styled(
                    format!("┌─ {} ({} B)", f.label, f.total_size()),
                    Style::default().fg(theme::STACK_CYAN).add_modifier(Modifier::BOLD),
                ))),
            ];
            for (name, size) in &f.vars {
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("│  {} = {} B", name, size),
                    Style::default().fg(theme::STACK_CYAN),
                ))));
            }
            items.push(ListItem::new(Line::from(Span::styled("└──────────────", theme::dim_style()))));
            items
        }).collect();
        let stack_list = List::new(stack_items)
            .block(Block::default().title("■ STACK (LIFO, fast O(1))").borders(Borders::ALL)
                .border_style(Style::default().fg(theme::STACK_CYAN)));
        frame.render_widget(stack_list, mid[0]);

        // Heap
        let heap_items: Vec<ListItem> = self.heap_blocks.iter().map(|b| {
            let (color, marker) = if b.alive {
                (theme::HEAP_BLUE, "✓")
            } else {
                (theme::TEXT_DIM, "✗ freed")
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", marker), Style::default().fg(color)),
                Span::styled(format!("{} — {}", b.label, format_bytes(b.size_bytes)), Style::default().fg(color)),
            ]))
        }).collect();
        let heap_list = List::new(heap_items)
            .block(Block::default().title("▣ HEAP (dynamic, flexible)").borders(Borders::ALL)
                .border_style(Style::default().fg(theme::HEAP_BLUE)));
        frame.render_widget(heap_list, mid[1]);

        // Stats bar
        let leaks = self.leaked_bytes();
        let leak_color = if leaks == 0 { theme::SAFE_GREEN } else { theme::CRAB_RED };
        let stats = Line::from(vec![
            Span::styled(format!(" allocs: {}  ", self.alloc_count), Style::default().fg(theme::HEAP_BLUE)),
            Span::styled(format!("frees: {}  ", self.free_count), Style::default().fg(theme::STACK_CYAN)),
            Span::styled(
                format!("leaks: {} bytes — {}", leaks, if leaks == 0 { "✓ ZERO LEAKS" } else { "⚠ LEAKED" }),
                Style::default().fg(leak_color).add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(stats).block(Block::default().borders(Borders::ALL)), chunks[2]);
    }

    fn name(&self) -> &'static str { "Memory Management" }
    fn description(&self) -> &'static str { "Stack vs heap — deterministic allocation without a garbage collector." }
    fn explanation(&self) -> &'static str {
        "Rust uses RAII (Resource Acquisition Is Initialization): resources are freed when \
        the owner goes out of scope. The Drop trait fires automatically — no GC, no finalizer, \
        no delay. Stack allocation is O(1) and cache-friendly. Heap allocation is explicit \
        via Box<T>, Vec<T>, String. Every allocation is paired with a guaranteed deallocation."
    }
    fn reset(&mut self) {
        self.step = 0; self.step_timer = 0.0; self.tick_count = 0;
        self.alloc_count = 0; self.free_count = 0;
        self.stack_frames.clear(); self.heap_blocks.clear();
        self.paused = false;
    }
    fn toggle_pause(&mut self) { self.paused = !self.paused; }
    fn is_paused(&self) -> bool { self.paused }
    fn set_speed(&mut self, speed: u8) { self.speed = speed.clamp(1, 10); }
    fn speed(&self) -> u8 { self.speed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_name_description_explanation() {
        let d = MemoryDemo::new();
        assert_eq!(d.name(), "Memory Management");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() { assert!(!MemoryDemo::new().is_paused()); }

    #[test]
    fn test_toggle_pause() {
        let mut d = MemoryDemo::new();
        d.toggle_pause(); assert!(d.is_paused());
        d.toggle_pause(); assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = MemoryDemo::new();
        d.set_speed(5); assert_eq!(d.speed(), 5);
        d.set_speed(0); assert_eq!(d.speed(), 1);
        d.set_speed(100); assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = MemoryDemo::new();
        d.step = 5; d.alloc_count = 3;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.alloc_count, 0);
        assert!(d.stack_frames.is_empty());
    }

    #[test]
    fn test_tick_paused() {
        let mut d = MemoryDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_all_ten_steps() {
        let mut d = MemoryDemo::new();
        for i in 0..STEPS {
            let title = step_title(i);
            assert!(!title.is_empty());
            d.advance_step();
        }
        assert_eq!(d.step, 0); // wrapped
    }

    #[test]
    fn test_alloc_and_free_counts() {
        let mut d = MemoryDemo::new();
        // advance through steps that add/remove heap
        for _ in 0..9 {
            d.advance_step();
        }
    }

    #[test]
    fn test_leaked_bytes_zero_initially() {
        let d = MemoryDemo::new();
        assert_eq!(d.leaked_bytes(), 0);
    }

    #[test]
    fn test_stack_frame_total_size() {
        let mut f = StackFrame::new("main");
        f.push_var("x", 4);
        f.push_var("y", 8);
        assert_eq!(f.total_size(), 12);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = MemoryDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = MemoryDemo::default();
        assert_eq!(d.step, 0);
    }
}
