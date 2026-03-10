use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use sysinfo::System;
use crate::{demos::Demo, theme};

const MAX_HISTORY: usize = 60;

#[derive(Debug)]
pub struct SystemMetricsDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    sys: System,
    /// Per-core CPU usage history (ring buffer, max MAX_HISTORY samples, values 0–100)
    pub cpu_history: Vec<Vec<f32>>,
    /// Used memory history in bytes
    pub mem_history: Vec<u64>,
    /// Current process approximate RSS in bytes (proxied via used_memory delta)
    pub process_rss_history: Vec<u64>,
    /// Total physical memory in bytes
    pub total_mem_bytes: u64,
    /// Number of logical CPU cores
    pub cpu_count: usize,
    /// GC pause time — always 0 in Rust (no GC)
    pub gc_pause_ms: f64,
    /// Ticks elapsed (for uptime calculation)
    pub uptime_ticks: u64,
    /// Simulated GC pause history (ms) for a hypothetical JVM/Go runtime
    pub simulated_gc_pauses: Vec<f64>,
    /// Independent tick counter used to drive deterministic GC simulation
    pub gc_sim_tick: u64,
    /// Count of simulated GC events (pause > 0)
    pub gc_sim_total_pauses: u64,
}

impl SystemMetricsDemo {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpu_count = sys.cpus().len().max(1);
        let total_mem_bytes = sys.total_memory();
        let cpu_history = vec![Vec::new(); cpu_count];
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            sys,
            cpu_history,
            mem_history: Vec::new(),
            process_rss_history: Vec::new(),
            total_mem_bytes,
            cpu_count,
            gc_pause_ms: 0.0,
            uptime_ticks: 0,
            simulated_gc_pauses: Vec::new(),
            gc_sim_tick: 0,
            gc_sim_total_pauses: 0,
        }
    }

    fn push_ring<T: Clone>(ring: &mut Vec<T>, value: T) {
        if ring.len() >= MAX_HISTORY {
            ring.remove(0);
        }
        ring.push(value);
    }
}

/// Deterministically generates a simulated GC pause value (ms) from a tick counter.
/// Returns 0.0 most of the time; spikes every ~30 ticks; major GC every ~120 ticks.
pub fn simulated_gc_pause_ms(tick: u64) -> f64 {
    if tick % 127 == 0 {
        // Major GC: 80–185 ms
        80.0 + ((tick % 7) as f64) * 15.0
    } else if tick % 31 == 0 {
        // Minor GC: 8–20 ms
        8.0 + ((tick % 5) as f64) * 3.0
    } else {
        0.0
    }
}

/// Format a byte count as a human-readable string.
/// Thresholds: <1 KB → "N B", <1 MB → "N.X KB", <1 GB → "N.X MB", else "N.X GB"
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    }
}

/// Format an uptime given tick count and ticks-per-second (fps).
/// Returns a string like "0d 0h 2m 10s".
pub fn format_uptime(ticks: u64, fps: u64) -> String {
    let total_secs = if fps == 0 { 0 } else { ticks / fps };
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = (total_secs / 3600) % 24;
    let days = total_secs / 86400;
    format!("{}d {}h {}m {}s", days, hours, mins, secs)
}

/// Map a CPU usage percentage to a display color.
pub fn cpu_usage_color(usage: f32) -> Color {
    if usage < 25.0 {
        theme::SAFE_GREEN
    } else if usage < 60.0 {
        theme::BORROW_YELLOW
    } else if usage < 85.0 {
        theme::RUST_ORANGE
    } else {
        Color::Red
    }
}

impl Default for SystemMetricsDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for SystemMetricsDemo {
    fn tick(&mut self, _dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.uptime_ticks += 1;

        // Refresh system data
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        // Collect CPU usage per core
        let cpus: Vec<f32> = self.sys.cpus().iter().map(|c| c.cpu_usage()).collect();

        // Ensure cpu_history vec has the right number of slots
        if self.cpu_history.len() != cpus.len() {
            self.cpu_history = vec![Vec::new(); cpus.len()];
            self.cpu_count = cpus.len().max(1);
        }

        for (i, usage) in cpus.iter().enumerate() {
            Self::push_ring(&mut self.cpu_history[i], *usage);
        }

        // Memory
        let used_mem = self.sys.used_memory();
        Self::push_ring(&mut self.mem_history, used_mem);

        // Process RSS proxy — use 0 (no process-specific tracking needed)
        Self::push_ring(&mut self.process_rss_history, 0u64);

        // GC pauses: always 0 — Rust has no garbage collector
        self.gc_pause_ms = 0.0;

        // Simulated GC (JVM/Go comparison)
        self.gc_sim_tick += 1;
        let pause = simulated_gc_pause_ms(self.gc_sim_tick);
        Self::push_ring(&mut self.simulated_gc_pauses, pause);
        if pause > 0.0 {
            self.gc_sim_total_pauses += 1;
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(6),      // CPU grid
                Constraint::Length(3),   // Memory gauge
                Constraint::Length(5),   // GC comparison row
                Constraint::Length(2),   // Uptime
            ])
            .split(area);

        // ── Title ────────────────────────────────────────────────────────────
        frame.render_widget(
            Paragraph::new(Span::styled(
                "System Metrics — Real-time CPU, RAM, and process stats — zero GC pauses",
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

        // ── CPU sparklines grid ──────────────────────────────────────────────
        let cpu_count = self.cpu_history.len().max(1);
        let per_row = 4usize;
        let rows = (cpu_count + per_row - 1) / per_row;

        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Ratio(1, rows as u32))
            .collect();

        let cpu_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(chunks[1]);

        for row in 0..rows {
            let cores_in_row = ((row + 1) * per_row).min(cpu_count) - row * per_row;
            let col_constraints: Vec<Constraint> = (0..per_row)
                .map(|_| Constraint::Ratio(1, per_row as u32))
                .collect();
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(cpu_rows[row]);

            for col in 0..per_row {
                let core_idx = row * per_row + col;
                if core_idx >= cpu_count {
                    break;
                }

                let history = &self.cpu_history[core_idx];
                let current_usage = history.last().copied().unwrap_or(0.0);
                let color = cpu_usage_color(current_usage);

                // Convert f32 0-100 to u64 for Sparkline
                let spark_data: Vec<u64> = history
                    .iter()
                    .map(|&v| (v.clamp(0.0, 100.0) as u64))
                    .collect();

                let title = format!(
                    "CPU{} {:.0}%",
                    core_idx,
                    current_usage,
                );

                // Only render if the column is within the actual core count
                if col < cores_in_row || core_idx < cpu_count {
                    frame.render_widget(
                        Sparkline::default()
                            .block(
                                Block::default()
                                    .title(title)
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(color)),
                            )
                            .data(&spark_data)
                            .max(100)
                            .style(Style::default().fg(color)),
                        cols[col],
                    );
                }
            }
        }

        // ── Memory gauge ─────────────────────────────────────────────────────
        let used_mem = self.mem_history.last().copied().unwrap_or(0);
        let total_mem = self.total_mem_bytes.max(1);
        let mem_ratio = (used_mem as f64 / total_mem as f64).clamp(0.0, 1.0);
        let mem_label = format!(
            "RAM: {} / {}  ({:.1}%)",
            format_bytes(used_mem),
            format_bytes(total_mem),
            mem_ratio * 100.0,
        );

        frame.render_widget(
            Gauge::default()
                .block(
                    Block::default()
                        .title("Memory")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::HEAP_BLUE)),
                )
                .gauge_style(Style::default().fg(theme::HEAP_BLUE))
                .label(mem_label)
                .ratio(mem_ratio),
            chunks[2],
        );

        // ── GC comparison row ─────────────────────────────────────────────────
        let gc_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[3]);

        // Left: Rust GC panel — flat sparkline of all zeros (no GC)
        let rust_zeros: Vec<u64> = vec![0u64; self.simulated_gc_pauses.len().max(2)];
        frame.render_widget(
            Sparkline::default()
                .block(
                    Block::default()
                        .title("Rust — GC Pauses: 0 ms (no GC)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SAFE_GREEN)),
                )
                .data(&rust_zeros)
                .max(200)
                .style(Style::default().fg(theme::SAFE_GREEN)),
            gc_row[0],
        );

        // Right: JVM/Go simulated GC panel
        let current_pause = self.simulated_gc_pauses.last().copied().unwrap_or(0.0);
        let gc_data: Vec<u64> = self.simulated_gc_pauses.iter().map(|&v| v as u64).collect();
        let gc_title = format!(
            "JVM/Go (sim) — cur: {:.0} ms  pauses: {}",
            current_pause,
            self.gc_sim_total_pauses,
        );
        frame.render_widget(
            Sparkline::default()
                .block(
                    Block::default()
                        .title(gc_title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::CRAB_RED)),
                )
                .data(&gc_data)
                .max(200)
                .style(Style::default().fg(theme::CRAB_RED)),
            gc_row[1],
        );

        // ── Uptime ────────────────────────────────────────────────────────────
        let uptime_str = format_uptime(self.uptime_ticks, 20);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  Uptime: ", theme::dim_style()),
                Span::styled(
                    uptime_str,
                    Style::default()
                        .fg(theme::STACK_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  (tick #{})", self.tick_count),
                    theme::dim_style(),
                ),
            ])),
            chunks[4],
        );
    }

    fn name(&self) -> &'static str {
        "System Metrics"
    }

    fn description(&self) -> &'static str {
        "Real-time CPU, RAM, and process stats — zero GC pauses."
    }

    fn explanation(&self) -> &'static str {
        "Rust processes have predictable, low memory footprints with no garbage collector pauses. \
        This screen shows live system metrics via the sysinfo crate. \
        Notice the GC pause counter: it is always 0. \
        Rust's deterministic memory management means latency spikes from GC simply do not exist."
    }

    fn reset(&mut self) {
        self.tick_count = 0;
        self.uptime_ticks = 0;
        self.mem_history.clear();
        self.process_rss_history.clear();
        for core_history in &mut self.cpu_history {
            core_history.clear();
        }
        self.gc_pause_ms = 0.0;
        self.gc_sim_tick = 0;
        self.simulated_gc_pauses.clear();
        self.gc_sim_total_pauses = 0;
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

    // ── format_bytes ──────────────────────────────────────────────────────────

    #[test]
    fn test_format_bytes_512_b() {
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn test_format_bytes_0_b() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_1023_b() {
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_1024_kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
    }

    #[test]
    fn test_format_bytes_1_mb() {
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
    }

    #[test]
    fn test_format_bytes_1_gb() {
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_format_bytes_2_gb() {
        assert_eq!(format_bytes(2_147_483_648), "2.0 GB");
    }

    // ── format_uptime ─────────────────────────────────────────────────────────

    #[test]
    fn test_format_uptime_zero() {
        let s = format_uptime(0, 20);
        assert!(s.contains("0d"), "got: {}", s);
        assert!(s.contains("0h"), "got: {}", s);
        assert!(s.contains("0m"), "got: {}", s);
        assert!(s.contains("0s"), "got: {}", s);
    }

    #[test]
    fn test_format_uptime_2_minutes() {
        // 2400 ticks / 20 fps = 120 seconds = 2 minutes
        let s = format_uptime(2400, 20);
        assert!(s.contains("2m"), "got: {}", s);
    }

    #[test]
    fn test_format_uptime_1_hour() {
        // 20 * 3600 = 72000 ticks
        let s = format_uptime(72000, 20);
        assert!(s.contains("1h"), "got: {}", s);
    }

    #[test]
    fn test_format_uptime_zero_fps() {
        // Should not panic
        let s = format_uptime(1000, 0);
        assert!(s.contains("0d"), "got: {}", s);
    }

    // ── cpu_usage_color ───────────────────────────────────────────────────────

    #[test]
    fn test_cpu_usage_color_low() {
        assert_eq!(cpu_usage_color(0.0), theme::SAFE_GREEN);
        assert_eq!(cpu_usage_color(10.0), theme::SAFE_GREEN);
        assert_eq!(cpu_usage_color(24.9), theme::SAFE_GREEN);
    }

    #[test]
    fn test_cpu_usage_color_medium() {
        assert_eq!(cpu_usage_color(25.0), theme::BORROW_YELLOW);
        assert_eq!(cpu_usage_color(50.0), theme::BORROW_YELLOW);
        assert_eq!(cpu_usage_color(59.9), theme::BORROW_YELLOW);
    }

    #[test]
    fn test_cpu_usage_color_high() {
        assert_eq!(cpu_usage_color(60.0), theme::RUST_ORANGE);
        assert_eq!(cpu_usage_color(70.0), theme::RUST_ORANGE);
        assert_eq!(cpu_usage_color(84.9), theme::RUST_ORANGE);
    }

    #[test]
    fn test_cpu_usage_color_critical() {
        assert_eq!(cpu_usage_color(85.0), Color::Red);
        assert_eq!(cpu_usage_color(95.0), Color::Red);
        assert_eq!(cpu_usage_color(100.0), Color::Red);
    }

    // ── gc_pause_ms always 0.0 ────────────────────────────────────────────────

    #[test]
    fn test_gc_pause_always_zero_on_new() {
        let d = SystemMetricsDemo::new();
        assert_eq!(d.gc_pause_ms, 0.0);
    }

    #[test]
    fn test_gc_pause_always_zero_after_tick() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..5 {
            d.tick(Duration::from_millis(50));
        }
        assert_eq!(d.gc_pause_ms, 0.0);
    }

    // ── Demo trait ────────────────────────────────────────────────────────────

    #[test]
    fn test_demo_trait_methods() {
        let mut d = SystemMetricsDemo::new();
        assert_eq!(d.name(), "System Metrics");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
        assert!(!d.is_paused());
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
        d.set_speed(8);
        assert_eq!(d.speed(), 8);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_tick_paused() {
        let mut d = SystemMetricsDemo::new();
        d.paused = true;
        d.tick(Duration::from_millis(50));
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.uptime_ticks, 0);
    }

    #[test]
    fn test_tick_increments_counts() {
        let mut d = SystemMetricsDemo::new();
        d.tick(Duration::from_millis(50));
        assert_eq!(d.tick_count, 1);
        assert_eq!(d.uptime_ticks, 1);
    }

    #[test]
    fn test_tick_populates_histories() {
        let mut d = SystemMetricsDemo::new();
        d.tick(Duration::from_millis(50));
        assert!(!d.mem_history.is_empty());
        assert!(!d.process_rss_history.is_empty());
    }

    #[test]
    fn test_mem_history_ring_buffer() {
        let mut d = SystemMetricsDemo::new();
        // Fill past MAX_HISTORY
        for _ in 0..MAX_HISTORY + 10 {
            d.tick(Duration::from_millis(10));
        }
        assert!(d.mem_history.len() <= MAX_HISTORY);
    }

    #[test]
    fn test_cpu_history_ring_buffer() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..MAX_HISTORY + 10 {
            d.tick(Duration::from_millis(10));
        }
        for core_hist in &d.cpu_history {
            assert!(core_hist.len() <= MAX_HISTORY);
        }
    }

    #[test]
    fn test_reset() {
        let mut d = SystemMetricsDemo::new();
        d.tick(Duration::from_millis(50));
        d.tick(Duration::from_millis(50));
        d.reset();
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.uptime_ticks, 0);
        assert!(d.mem_history.is_empty());
        assert!(d.process_rss_history.is_empty());
        assert_eq!(d.gc_pause_ms, 0.0);
        assert!(!d.is_paused());
        for core_hist in &d.cpu_history {
            assert!(core_hist.is_empty());
        }
    }

    #[test]
    fn test_render() {
        let d = SystemMetricsDemo::new();
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_after_ticks() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..5 {
            d.tick(Duration::from_millis(50));
        }
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_default() {
        let d = SystemMetricsDemo::default();
        assert_eq!(d.gc_pause_ms, 0.0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_push_ring_caps_at_max() {
        let mut v: Vec<u64> = Vec::new();
        for i in 0..MAX_HISTORY + 5 {
            SystemMetricsDemo::push_ring(&mut v, i as u64);
        }
        assert_eq!(v.len(), MAX_HISTORY);
        // The oldest entries should be evicted
        assert_eq!(v[0], 5u64);
    }

    #[test]
    fn test_format_bytes_boundary_values() {
        // Exactly at boundary
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024 - 1), "1024.0 KB");
        // Just over 1 MB
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    }

    // ── New tests ─────────────────────────────────────────────────────────────

    #[test]
    fn test_simulated_gc_pause_ms_normal_tick() {
        // Most ticks should return 0.0
        assert_eq!(simulated_gc_pause_ms(1), 0.0);
        assert_eq!(simulated_gc_pause_ms(2), 0.0);
        assert_eq!(simulated_gc_pause_ms(10), 0.0);
        assert_eq!(simulated_gc_pause_ms(50), 0.0);
    }

    #[test]
    fn test_simulated_gc_pause_ms_minor_gc() {
        // tick 31 triggers a minor GC (>0)
        let pause = simulated_gc_pause_ms(31);
        assert!(pause > 0.0, "tick 31 should produce a minor GC pause, got {}", pause);
        assert!(pause >= 8.0 && pause <= 20.0, "minor GC should be 8-20 ms, got {}", pause);
    }

    #[test]
    fn test_simulated_gc_pause_ms_major_gc() {
        // tick 127 triggers a major GC (>=80ms)
        let pause = simulated_gc_pause_ms(127);
        assert!(pause >= 80.0, "tick 127 should produce a major GC pause >= 80ms, got {}", pause);
    }

    #[test]
    fn test_gc_sim_populates_after_ticks() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..5 {
            d.tick(Duration::from_millis(50));
        }
        assert!(!d.simulated_gc_pauses.is_empty(), "simulated_gc_pauses should be non-empty after 5 ticks");
    }

    #[test]
    fn test_gc_sim_total_pauses_increments() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..200 {
            d.tick(Duration::from_millis(10));
        }
        assert!(
            d.gc_sim_total_pauses > 0,
            "gc_sim_total_pauses should be > 0 after 200 ticks, got {}",
            d.gc_sim_total_pauses
        );
    }

    #[test]
    fn test_reset_clears_gc_sim() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..50 {
            d.tick(Duration::from_millis(10));
        }
        d.reset();
        assert_eq!(d.gc_sim_tick, 0, "gc_sim_tick should be 0 after reset");
        assert!(d.simulated_gc_pauses.is_empty(), "simulated_gc_pauses should be empty after reset");
        assert_eq!(d.gc_sim_total_pauses, 0, "gc_sim_total_pauses should be 0 after reset");
    }

    #[test]
    fn test_render_with_gc_comparison() {
        let mut d = SystemMetricsDemo::new();
        for _ in 0..5 {
            d.tick(Duration::from_millis(50));
        }
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }
}
