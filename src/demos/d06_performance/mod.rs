use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

/// Result from a single sort benchmark run.
#[derive(Debug, Clone)]
pub struct SortResult {
    pub name: &'static str,
    pub n: usize,
    pub ns_per_op: u64,
}

impl SortResult {
    pub fn new(name: &'static str, n: usize, ns_per_op: u64) -> Self {
        Self { name, n, ns_per_op }
    }
}

/// Phase of the performance demo cycle.
#[derive(Debug, Clone, PartialEq)]
pub enum PerfPhase {
    Sort,
    Arithmetic,
    Allocation,
    LangCompare,
    Summary,
}

impl PerfPhase {
    pub fn next(&self) -> Self {
        match self {
            PerfPhase::Sort => PerfPhase::Arithmetic,
            PerfPhase::Arithmetic => PerfPhase::Allocation,
            PerfPhase::Allocation => PerfPhase::LangCompare,
            PerfPhase::LangCompare => PerfPhase::Summary,
            PerfPhase::Summary => PerfPhase::Sort,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            PerfPhase::Sort => "Sort Benchmark — sort_unstable vs sort (stable)",
            PerfPhase::Arithmetic => "Arithmetic Throughput — integer ops/sec",
            PerfPhase::Allocation => "Allocation Throughput — heap allocs/sec",
            PerfPhase::LangCompare => {
                "Language Comparison — relative sort performance (illustrative)"
            }
            PerfPhase::Summary => "Performance Summary — all benchmarks",
        }
    }
}

/// Illustrative language comparison data: (language, ns/op, color).
/// These are educational approximations, clearly labeled as such in the UI.
pub fn lang_compare_data() -> &'static [(&'static str, u64, Color)] {
    &[
        ("Rust (sort_unstable)", 5, theme::SAFE_GREEN),
        ("C++ (std::sort)", 6, theme::HEAP_BLUE),
        ("Go (sort.Slice)", 18, theme::STACK_CYAN),
        ("Java (Arrays.sort)", 22, theme::BORROW_YELLOW),
        ("Python (list.sort)", 420, theme::CRAB_RED),
    ]
}

/// The performance benchmark demo.
#[derive(Debug)]
pub struct PerformanceDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    phase_timer: f64,
    pub phase: PerfPhase,
    pub sort_results: Vec<SortResult>,
    pub arith_ops_per_sec: u64,
    pub alloc_ops_per_sec: u64,
    pub run_count: u64,
    bench_n: usize,
}

impl PerformanceDemo {
    pub fn new() -> Self {
        let mut d = Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            phase_timer: 0.0,
            phase: PerfPhase::Sort,
            sort_results: Vec::new(),
            arith_ops_per_sec: 0,
            alloc_ops_per_sec: 0,
            run_count: 0,
            bench_n: 10_000,
        };
        d.run_all_benches();
        d
    }

    pub fn phase_period_secs(&self) -> f64 {
        3.0 / self.speed as f64
    }

    fn run_all_benches(&mut self) {
        let n = self.bench_n;
        let ns_unstable = bench_std_sort_unstable(n);
        let ns_stable = bench_std_sort_stable(n);

        self.sort_results = vec![
            SortResult::new("sort_unstable", n, ns_unstable),
            SortResult::new("sort (stable)", n, ns_stable),
        ];

        self.arith_ops_per_sec = bench_arithmetic_ops_per_sec();
        self.alloc_ops_per_sec = bench_alloc_ops_per_sec();
        self.run_count += 1;

        // Cycle bench_n: 1_000 -> 10_000 -> 100_000 -> 1_000
        self.bench_n = match self.bench_n {
            1_000 => 10_000,
            10_000 => 100_000,
            _ => 1_000,
        };
    }

    pub fn best_sort_ns(&self) -> u64 {
        self.sort_results
            .iter()
            .map(|r| r.ns_per_op)
            .min()
            .unwrap_or(0)
    }
}

/// Sort n random u64s with sort_unstable. Returns nanoseconds per element.
pub fn bench_std_sort_unstable(n: usize) -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut data: Vec<u64> = (0..n).map(|_| rng.gen()).collect();
    let start = Instant::now();
    data.sort_unstable();
    let ns = start.elapsed().as_nanos() as u64;
    // Prevent optimizer from eliding the result
    let _ = data.last();
    if n == 0 {
        return 0;
    }
    (ns / n as u64).max(1)
}

/// Sort n random u64s with stable sort. Returns nanoseconds per element.
pub fn bench_std_sort_stable(n: usize) -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut data: Vec<u64> = (0..n).map(|_| rng.gen()).collect();
    let start = Instant::now();
    data.sort();
    let ns = start.elapsed().as_nanos() as u64;
    let _ = data.last();
    if n == 0 {
        return 0;
    }
    (ns / n as u64).max(1)
}

/// Benchmark integer arithmetic throughput. Returns operations per second.
pub fn bench_arithmetic_ops_per_sec() -> u64 {
    const OPS: u64 = 1_000_000;
    let start = Instant::now();
    let mut acc: u64 = 1;
    for i in 0..OPS {
        // Mix of multiply, add, xor to prevent trivial optimization
        acc = acc.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(i) ^ (i << 17);
    }
    // Prevent dead-code elimination
    let _ = acc;
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    if elapsed_ns == 0 {
        return OPS * 1_000_000_000;
    }
    OPS * 1_000_000_000 / elapsed_ns
}

/// Benchmark heap allocation throughput. Returns allocations per second.
pub fn bench_alloc_ops_per_sec() -> u64 {
    const OPS: u64 = 50_000;
    let start = Instant::now();
    for _ in 0..OPS {
        // Allocate and immediately drop a 64-byte Vec
        let v: Vec<u8> = Vec::with_capacity(64);
        // Use a volatile read to prevent the optimizer from removing the alloc
        let _ = v.capacity();
    }
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    if elapsed_ns == 0 {
        return OPS * 1_000_000_000;
    }
    OPS * 1_000_000_000 / elapsed_ns
}

impl Default for PerformanceDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for PerformanceDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.phase_timer += dt.as_secs_f64();
        if self.phase_timer >= self.phase_period_secs() {
            self.phase_timer = 0.0;
            self.phase = self.phase.next();
            // Re-run benchmarks on each full cycle (when returning to Sort)
            if self.phase == PerfPhase::Sort {
                self.run_all_benches();
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // title
                Constraint::Min(10),   // chart area
                Constraint::Length(4), // stats bar
            ])
            .split(area);

        // Title
        frame.render_widget(
            Paragraph::new(Span::styled(
                self.phase.title(),
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

        // Chart / detail area
        match self.phase {
            PerfPhase::Sort => {
                // BarChart: sort_unstable vs sort (stable) ns/element
                let bars: Vec<Bar> = self
                    .sort_results
                    .iter()
                    .map(|r| {
                        Bar::default()
                            .value(r.ns_per_op)
                            .label(Line::from(r.name))
                            .style(Style::default().fg(theme::SAFE_GREEN))
                            .value_style(
                                Style::default()
                                    .fg(theme::RUST_ORANGE)
                                    .add_modifier(Modifier::BOLD),
                            )
                    })
                    .collect();

                let group = BarGroup::default()
                    .label(Line::from(Span::styled(
                        format!(
                            "n = {} elements",
                            self.sort_results.first().map(|r| r.n).unwrap_or(0)
                        ),
                        theme::dim_style(),
                    )))
                    .bars(&bars);

                let chart = BarChart::default()
                    .data(group)
                    .bar_width(14)
                    .bar_gap(3)
                    .max(
                        self.sort_results
                            .iter()
                            .map(|r| r.ns_per_op)
                            .max()
                            .unwrap_or(1)
                            .max(1),
                    )
                    .block(
                        Block::default()
                            .title("ns per element (lower is better)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::SAFE_GREEN)),
                    );
                frame.render_widget(chart, chunks[1]);
            }

            PerfPhase::Arithmetic => {
                let lines = vec![
                    Line::from(Span::styled(
                        "Integer Arithmetic Ops/sec:",
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Benchmark: x = x.wrapping_mul(LCG) + i ^ (i << 17)",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "Prevents constant-folding — simulates real arithmetic.",
                        theme::dim_style(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  Result: {:>14} ops/sec", fmt_ops(self.arith_ops_per_sec)),
                        Style::default()
                            .fg(theme::BORROW_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Rust: zero-overhead arithmetic, no boxing, no GC pauses.",
                        theme::dim_style(),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("Arithmetic Throughput")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::BORROW_YELLOW)),
                    ),
                    chunks[1],
                );
            }

            PerfPhase::Allocation => {
                let lines = vec![
                    Line::from(Span::styled(
                        "Heap Allocation Ops/sec:",
                        Style::default()
                            .fg(theme::HEAP_BLUE)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Benchmark: Vec::with_capacity(64) + immediate drop",
                        theme::dim_style(),
                    )),
                    Line::from(Span::styled(
                        "Measures allocator round-trip throughput (alloc + free).",
                        theme::dim_style(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!(
                            "  Result: {:>14} allocs/sec",
                            fmt_ops(self.alloc_ops_per_sec)
                        ),
                        Style::default()
                            .fg(theme::HEAP_BLUE)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Rust uses jemalloc (or system allocator): deterministic timing.",
                        theme::dim_style(),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).block(
                        Block::default()
                            .title("Allocation Throughput")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::HEAP_BLUE)),
                    ),
                    chunks[1],
                );
            }

            PerfPhase::LangCompare => {
                let lc_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(8), Constraint::Length(2)])
                    .split(chunks[1]);

                let data = lang_compare_data();
                let bars: Vec<Bar> = data
                    .iter()
                    .map(|(name, value, color)| {
                        Bar::default()
                            .value(*value)
                            .label(Line::from(*name))
                            .style(Style::default().fg(*color))
                            .value_style(
                                Style::default()
                                    .fg(theme::TEXT_PRIMARY)
                                    .add_modifier(Modifier::BOLD),
                            )
                    })
                    .collect();

                let group = BarGroup::default()
                    .label(Line::from(Span::styled(
                        "ns/op — lower is better (illustrative values)",
                        theme::dim_style(),
                    )))
                    .bars(&bars);

                let max_val = data.iter().map(|(_, v, _)| *v).max().unwrap_or(1);

                let chart = BarChart::default()
                    .data(group)
                    .bar_width(14)
                    .bar_gap(2)
                    .max(max_val)
                    .block(
                        Block::default()
                            .title(PerfPhase::LangCompare.title())
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::STACK_CYAN)),
                    );
                frame.render_widget(chart, lc_chunks[0]);

                frame.render_widget(
                    Paragraph::new(Span::styled(
                        "* Illustrative — not a formal benchmark. Relative scale only.",
                        theme::dim_style(),
                    )),
                    lc_chunks[1],
                );
            }

            PerfPhase::Summary => {
                let s0 = if !self.sort_results.is_empty() {
                    self.sort_results[0].ns_per_op.max(1) as f64
                } else {
                    1.0
                };
                let s1 = if self.sort_results.len() > 1 {
                    self.sort_results[1].ns_per_op.max(1) as f64
                } else {
                    1.0
                };
                let arith_ns = if self.arith_ops_per_sec > 0 {
                    1_000_000_000.0 / self.arith_ops_per_sec as f64
                } else {
                    1.0
                };
                let alloc_ns = if self.alloc_ops_per_sec > 0 {
                    1_000_000_000.0 / self.alloc_ops_per_sec as f64
                } else {
                    1.0
                };
                let total = (s0 + s1 + arith_ns + alloc_ns).max(1.0);
                let mut fg = crate::ui::widgets::FlameGraph::new();
                fg.color = crate::theme::SAFE_GREEN;
                fg.push_frame(
                    format!(
                        "sort_unstable  {}ns/elem",
                        self.sort_results.first().map(|r| r.ns_per_op).unwrap_or(0)
                    ),
                    s0 / total,
                );
                fg.push_frame(
                    format!(
                        "sort (stable)  {}ns/elem",
                        self.sort_results.get(1).map(|r| r.ns_per_op).unwrap_or(0)
                    ),
                    s1 / total,
                );
                fg.push_frame(
                    format!(
                        "arithmetic     {} Mops/s",
                        self.arith_ops_per_sec / 1_000_000
                    ),
                    arith_ns / total,
                );
                fg.push_frame(
                    format!("heap alloc     {} Kops/s", self.alloc_ops_per_sec / 1_000),
                    alloc_ns / total,
                );
                fg.render(frame, chunks[1]);
            }
        }

        // Stats bar
        let stats = Line::from(vec![
            Span::styled(
                format!(" run #{:3}  ", self.run_count),
                Style::default().fg(theme::ASYNC_PURPLE),
            ),
            Span::styled(format!("n = {:>7}  ", self.bench_n), theme::dim_style()),
            Span::styled(
                format!("sort_unstable: {} ns/elem", self.best_sort_ns()),
                Style::default()
                    .fg(theme::SAFE_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  arith: {} ops/sec", fmt_ops(self.arith_ops_per_sec)),
                theme::dim_style(),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(stats).block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }

    fn name(&self) -> &'static str {
        "Performance Benchmarks"
    }
    fn description(&self) -> &'static str {
        "Real-time micro-benchmarks: sort, arithmetic, and allocation throughput."
    }
    fn explanation(&self) -> &'static str {
        "Rust delivers C-like performance without a garbage collector. \
        sort_unstable is faster than stable sort because it doesn't preserve \
        the relative order of equal elements (no auxiliary memory). \
        Arithmetic throughput demonstrates the compiler's optimization quality. \
        Allocation benchmarks show the system allocator's round-trip latency. \
        All results are measured in the same process — no JIT warmup, no GC pauses."
    }
    fn reset(&mut self) {
        self.tick_count = 0;
        self.phase_timer = 0.0;
        self.phase = PerfPhase::Sort;
        self.run_count = 0;
        self.bench_n = 10_000;
        self.sort_results.clear();
        self.arith_ops_per_sec = 0;
        self.alloc_ops_per_sec = 0;
        self.paused = false;
        self.run_all_benches();
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
            "How does Rust achieve C-comparable performance?",
            [
                "JIT compilation",
                "Ahead-of-time compilation with no GC overhead",
                "Bytecode VM",
                "Hardware acceleration",
            ],
            1,
        ))
    }
}

/// Format large op counts as e.g. "1.23 B", "456 M", "789 K".
pub fn fmt_ops(ops: u64) -> String {
    if ops >= 1_000_000_000 {
        format!("{:.2} B/s", ops as f64 / 1_000_000_000.0)
    } else if ops >= 1_000_000 {
        format!("{:.2} M/s", ops as f64 / 1_000_000.0)
    } else if ops >= 1_000 {
        format!("{:.2} K/s", ops as f64 / 1_000.0)
    } else {
        format!("{} /s", ops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // --- Public bench function tests ---

    #[test]
    fn test_bench_sort_unstable_nonzero() {
        let ns = bench_std_sort_unstable(1_000);
        assert!(
            ns > 0,
            "bench_std_sort_unstable should return > 0, got {}",
            ns
        );
    }

    #[test]
    fn test_bench_sort_stable_nonzero() {
        let ns = bench_std_sort_stable(1_000);
        assert!(
            ns > 0,
            "bench_std_sort_stable should return > 0, got {}",
            ns
        );
    }

    #[test]
    fn test_bench_sort_unstable_zero_n() {
        // n=0 should return 0 without panicking
        let ns = bench_std_sort_unstable(0);
        assert_eq!(ns, 0);
    }

    #[test]
    fn test_bench_sort_stable_zero_n() {
        let ns = bench_std_sort_stable(0);
        assert_eq!(ns, 0);
    }

    #[test]
    fn test_bench_sort_unstable_large_n() {
        let ns = bench_std_sort_unstable(50_000);
        assert!(ns > 0);
    }

    #[test]
    fn test_bench_sort_stable_large_n() {
        let ns = bench_std_sort_stable(50_000);
        assert!(ns > 0);
    }

    #[test]
    fn test_bench_arithmetic_ops_per_sec_nonzero() {
        let ops = bench_arithmetic_ops_per_sec();
        assert!(
            ops > 0,
            "bench_arithmetic_ops_per_sec should return > 0, got {}",
            ops
        );
        // Sanity: modern CPUs do > 10M integer ops/sec
        assert!(ops > 1_000_000, "expected > 1M ops/sec, got {}", ops);
    }

    #[test]
    fn test_bench_alloc_ops_per_sec_nonzero() {
        let ops = bench_alloc_ops_per_sec();
        assert!(
            ops > 0,
            "bench_alloc_ops_per_sec should return > 0, got {}",
            ops
        );
        // Sanity: should do at least 1K allocs/sec
        assert!(ops > 1_000, "expected > 1K allocs/sec, got {}", ops);
    }

    // --- SortResult tests ---

    #[test]
    fn test_sort_result_new() {
        let r = SortResult::new("test", 1000, 42);
        assert_eq!(r.name, "test");
        assert_eq!(r.n, 1000);
        assert_eq!(r.ns_per_op, 42);
    }

    // --- fmt_ops tests ---

    #[test]
    fn test_fmt_ops_small() {
        let s = fmt_ops(500);
        assert!(s.contains("/s"));
    }

    #[test]
    fn test_fmt_ops_kilo() {
        let s = fmt_ops(1_500);
        assert!(s.contains("K/s"));
    }

    #[test]
    fn test_fmt_ops_mega() {
        let s = fmt_ops(2_500_000);
        assert!(s.contains("M/s"));
    }

    #[test]
    fn test_fmt_ops_giga() {
        let s = fmt_ops(3_000_000_000);
        assert!(s.contains("B/s"));
    }

    // --- Demo trait tests ---

    #[test]
    fn test_name_description_explanation() {
        let d = PerformanceDemo::new();
        assert_eq!(d.name(), "Performance Benchmarks");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() {
        assert!(!PerformanceDemo::new().is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = PerformanceDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = PerformanceDemo::new();
        d.set_speed(7);
        assert_eq!(d.speed(), 7);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(100);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_new_runs_benches() {
        let d = PerformanceDemo::new();
        assert_eq!(d.run_count, 1);
        assert!(!d.sort_results.is_empty());
        assert!(d.arith_ops_per_sec > 0);
        assert!(d.alloc_ops_per_sec > 0);
    }

    #[test]
    fn test_sort_results_have_two_entries() {
        let d = PerformanceDemo::new();
        assert_eq!(d.sort_results.len(), 2);
    }

    #[test]
    fn test_sort_results_ns_per_op_nonzero() {
        let d = PerformanceDemo::new();
        for r in &d.sort_results {
            assert!(r.ns_per_op > 0, "ns_per_op for {} should be > 0", r.name);
        }
    }

    #[test]
    fn test_best_sort_ns() {
        let d = PerformanceDemo::new();
        let best = d.best_sort_ns();
        assert!(best > 0);
        assert!(
            best <= d
                .sort_results
                .iter()
                .map(|r| r.ns_per_op)
                .max()
                .unwrap_or(0)
        );
    }

    #[test]
    fn test_reset() {
        let mut d = PerformanceDemo::new();
        d.tick_count = 999;
        d.phase = PerfPhase::Summary;
        d.reset();
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.phase, PerfPhase::Sort);
        assert_eq!(d.run_count, 1); // reset runs benches once
        assert!(!d.sort_results.is_empty());
    }

    #[test]
    fn test_tick_paused_no_change() {
        let mut d = PerformanceDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.phase, PerfPhase::Sort);
    }

    #[test]
    fn test_tick_advances_phase() {
        let mut d = PerformanceDemo::new();
        d.set_speed(10);
        // period = 3.0 / 10 = 0.3s
        d.tick(Duration::from_secs_f64(0.4));
        // Should have advanced from Sort to Arithmetic
        assert_eq!(d.phase, PerfPhase::Arithmetic);
    }

    #[test]
    fn test_phase_cycle() {
        let p = PerfPhase::Sort;
        assert_eq!(p.next(), PerfPhase::Arithmetic);
        assert_eq!(PerfPhase::Arithmetic.next(), PerfPhase::Allocation);
        assert_eq!(PerfPhase::Allocation.next(), PerfPhase::LangCompare);
        assert_eq!(PerfPhase::LangCompare.next(), PerfPhase::Summary);
        assert_eq!(PerfPhase::Summary.next(), PerfPhase::Sort);
    }

    #[test]
    fn test_phase_titles_nonempty() {
        assert!(!PerfPhase::Sort.title().is_empty());
        assert!(!PerfPhase::Arithmetic.title().is_empty());
        assert!(!PerfPhase::Allocation.title().is_empty());
        assert!(!PerfPhase::LangCompare.title().is_empty());
        assert!(!PerfPhase::Summary.title().is_empty());
    }

    #[test]
    fn test_phase_period_varies_with_speed() {
        let mut d = PerformanceDemo::new();
        d.set_speed(1);
        let slow = d.phase_period_secs();
        d.set_speed(10);
        let fast = d.phase_period_secs();
        assert!(fast < slow);
    }

    #[test]
    fn test_render_sort_phase() {
        let d = PerformanceDemo::new();
        assert_eq!(d.phase, PerfPhase::Sort);
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_all_phases() {
        let mut d = PerformanceDemo::new();
        let phases = [
            PerfPhase::Sort,
            PerfPhase::Arithmetic,
            PerfPhase::Allocation,
            PerfPhase::Summary,
        ];
        for phase in &phases {
            d.phase = phase.clone();
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
        }
    }

    #[test]
    fn test_default() {
        let d = PerformanceDemo::default();
        assert_eq!(d.run_count, 1);
        assert_eq!(d.phase, PerfPhase::Sort);
    }

    #[test]
    fn test_tick_count_increments() {
        let mut d = PerformanceDemo::new();
        d.tick(Duration::from_millis(10));
        assert_eq!(d.tick_count, 1);
    }

    #[test]
    fn test_bench_n_cycles_after_full_phase_cycle() {
        let mut d = PerformanceDemo::new();
        let initial_n = d.bench_n;
        d.set_speed(10);
        // Tick through 4 phases
        for _ in 0..4 {
            d.tick(Duration::from_secs_f64(0.4));
        }
        // bench_n should be one of the valid cycling values
        assert!(
            d.bench_n == 1_000 || d.bench_n == 10_000 || d.bench_n == 100_000,
            "bench_n should be one of 1000/10000/100000, got {}",
            d.bench_n
        );
        let _ = initial_n;
    }

    // ── New tests ─────────────────────────────────────────────────────────────

    #[test]
    fn test_lang_compare_data_len() {
        assert_eq!(lang_compare_data().len(), 5);
    }

    #[test]
    fn test_lang_compare_data_rust_fastest() {
        let data = lang_compare_data();
        let rust_ns = data
            .iter()
            .find(|(name, _, _)| name.contains("Rust"))
            .map(|(_, ns, _)| *ns)
            .expect("Rust entry not found");
        let min_ns = data.iter().map(|(_, ns, _)| *ns).min().unwrap();
        assert_eq!(rust_ns, min_ns, "Rust should have the smallest ns/op value");
    }

    #[test]
    fn test_phase_cycle_includes_lang_compare() {
        assert_eq!(PerfPhase::Allocation.next(), PerfPhase::LangCompare);
        assert_eq!(PerfPhase::LangCompare.next(), PerfPhase::Summary);
    }

    #[test]
    fn test_lang_compare_title_nonempty() {
        assert!(!PerfPhase::LangCompare.title().is_empty());
        assert!(PerfPhase::LangCompare.title().contains("Language"));
    }

    #[test]
    fn test_render_lang_compare_phase() {
        let mut d = PerformanceDemo::new();
        d.phase = PerfPhase::LangCompare;
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }
}
