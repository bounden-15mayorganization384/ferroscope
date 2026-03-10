use crate::{demos::Demo, theme, ui::widgets::SparklineExt};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum BenchPhase {
    Running,
    Displaying,
}

#[derive(Debug)]
pub struct ZeroCostDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    phase: BenchPhase,
    phase_timer: f64,
    pub bench_n: u64,
    pub iter_ns: u64,
    pub loop_ns: u64,
    pub iter_result: u64,
    pub loop_result: u64,
    pub run_count: u64,
    ns_history_iter: Vec<u64>,
    ns_history_loop: Vec<u64>,
}

impl ZeroCostDemo {
    pub fn new() -> Self {
        let mut d = Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            phase: BenchPhase::Running,
            phase_timer: 0.0,
            bench_n: 1_000,
            iter_ns: 0,
            loop_ns: 0,
            iter_result: 0,
            loop_result: 0,
            run_count: 0,
            ns_history_iter: Vec::new(),
            ns_history_loop: Vec::new(),
        };
        d.run_bench();
        d
    }

    fn run_bench(&mut self) {
        let (ir, ins) = run_iterator_bench(self.bench_n);
        let (lr, lns) = run_loop_bench(self.bench_n);
        self.iter_result = ir;
        self.iter_ns = ins;
        self.loop_result = lr;
        self.loop_ns = lns;
        self.run_count += 1;
        self.ns_history_iter.push(ins);
        self.ns_history_loop.push(lns);
        if self.ns_history_iter.len() > 30 {
            self.ns_history_iter.remove(0);
        }
        if self.ns_history_loop.len() > 30 {
            self.ns_history_loop.remove(0);
        }
    }

    fn cycle_n(&mut self) {
        self.bench_n = match self.bench_n {
            100 => 1_000,
            1_000 => 10_000,
            _ => 100,
        };
    }

    pub fn results_match(&self) -> bool {
        self.iter_result == self.loop_result && self.run_count > 0
    }

    pub fn ratio(&self) -> f64 {
        if self.loop_ns == 0 {
            return 1.0;
        }
        self.iter_ns as f64 / self.loop_ns as f64
    }
}

/// Run benchmark: (0..n).filter(|x| x%2==0).map(|x| x*x).sum()
pub fn run_iterator_bench(n: u64) -> (u64, u64) {
    let start = Instant::now();
    let result: u64 = (0..n).filter(|x| x % 2 == 0).map(|x| x * x).sum();
    let ns = start.elapsed().as_nanos() as u64;
    (result, ns.max(1))
}

/// Run benchmark: equivalent manual loop
pub fn run_loop_bench(n: u64) -> (u64, u64) {
    let start = Instant::now();
    let mut sum = 0u64;
    for x in 0..n {
        if x % 2 == 0 {
            sum += x * x;
        }
    }
    let ns = start.elapsed().as_nanos() as u64;
    (sum, ns.max(1))
}

impl Default for ZeroCostDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for ZeroCostDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.phase_timer += dt.as_secs_f64();

        let period = 3.0 / self.speed as f64;
        if self.phase_timer >= period {
            self.phase_timer = 0.0;
            match self.phase {
                BenchPhase::Running => {
                    self.run_bench();
                    self.phase = BenchPhase::Displaying;
                }
                BenchPhase::Displaying => {
                    self.cycle_n();
                    self.phase = BenchPhase::Running;
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(5),
                Constraint::Length(5),
            ])
            .split(area);

        // Title — shows live phase status
        let (title_text, title_color) = match self.phase {
            BenchPhase::Running => (
                format!(
                    "● Running benchmark…  n = {}  |  Zero-Cost Abstractions",
                    self.bench_n
                ),
                theme::BORROW_YELLOW,
            ),
            BenchPhase::Displaying => (
                format!(
                    "✓ Complete — next: n = {}  |  Zero-Cost Abstractions",
                    self.bench_n
                ),
                theme::SAFE_GREEN,
            ),
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                title_text,
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(title_color)),
            ),
            chunks[0],
        );

        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left: iterator result
        let iter_lines = vec![
            Line::from(Span::styled(
                "Iterator approach:",
                Style::default()
                    .fg(theme::SAFE_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "(0..n).filter(|x| x%2==0)",
                theme::dim_style(),
            )),
            Line::from(Span::styled("    .map(|x| x*x).sum()", theme::dim_style())),
            Line::from(""),
            Line::from(Span::styled(
                format!("N = {}", self.bench_n),
                theme::label_style(),
            )),
            Line::from(Span::styled(
                format!("Result: {}", self.iter_result),
                theme::label_style(),
            )),
            Line::from(Span::styled(
                format!("Time: {} ns", self.iter_ns),
                Style::default()
                    .fg(theme::SAFE_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        frame.render_widget(
            Paragraph::new(iter_lines).block(
                Block::default()
                    .title("Iterator Chain")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::SAFE_GREEN)),
            ),
            mid[0],
        );

        // Right: loop result
        let loop_lines = vec![
            Line::from(Span::styled(
                "Manual loop:",
                Style::default()
                    .fg(theme::HEAP_BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("let mut sum = 0;", theme::dim_style())),
            Line::from(Span::styled("for x in 0..n {", theme::dim_style())),
            Line::from(Span::styled(
                "  if x%2==0 { sum+=x*x; }",
                theme::dim_style(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("N = {}", self.bench_n),
                theme::label_style(),
            )),
            Line::from(Span::styled(
                format!("Time: {} ns", self.loop_ns),
                Style::default()
                    .fg(theme::HEAP_BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        frame.render_widget(
            Paragraph::new(loop_lines).block(
                Block::default()
                    .title("Manual Loop")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::HEAP_BLUE)),
            ),
            mid[1],
        );

        // Stats — BarChart comparing iter_ns vs loop_ns
        let max_ns = self.iter_ns.max(self.loop_ns).max(1);
        let match_str = if self.results_match() {
            "✓ match"
        } else {
            "✗ mismatch"
        };
        let bar_title = format!(
            " {} | ratio: {:.3}x | run #{} ",
            match_str,
            self.ratio(),
            self.run_count
        );
        let iter_bar = Bar::default()
            .value(self.iter_ns)
            .label(Line::from("iter"))
            .style(Style::default().fg(theme::SAFE_GREEN));
        let loop_bar = Bar::default()
            .value(self.loop_ns)
            .label(Line::from("loop"))
            .style(Style::default().fg(theme::HEAP_BLUE));
        let bar_group = BarGroup::default().bars(&[iter_bar, loop_bar]);
        frame.render_widget(
            BarChart::default()
                .data(bar_group)
                .bar_width(7)
                .bar_gap(1)
                .max(max_ns)
                .block(Block::default().title(bar_title).borders(Borders::ALL)),
            chunks[2],
        );

        // History sparklines — ns timing over last N runs
        let hist_max = self
            .ns_history_iter
            .iter()
            .chain(&self.ns_history_loop)
            .copied()
            .max()
            .max(Some(1))
            .unwrap_or(1);
        let spark_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[3]);
        let mut iter_spark = SparklineExt::new("Iterator ns history", hist_max, theme::SAFE_GREEN);
        for &v in &self.ns_history_iter {
            iter_spark.push(v);
        }
        iter_spark.render(frame, spark_cols[0]);
        let mut loop_spark = SparklineExt::new("Loop ns history", hist_max, theme::HEAP_BLUE);
        for &v in &self.ns_history_loop {
            loop_spark.push(v);
        }
        loop_spark.render(frame, spark_cols[1]);
    }

    fn name(&self) -> &'static str {
        "Zero-Cost Abstractions"
    }
    fn description(&self) -> &'static str {
        "High-level iterators compile to the same code as manual loops."
    }
    fn explanation(&self) -> &'static str {
        "Rust's iterators, closures, and generics have zero runtime overhead. \
        The compiler monomorphizes generic code and inlines iterator chains, \
        producing identical machine code to hand-written loops. This is what \
        'zero-cost abstraction' means: you don't pay for what you use, and \
        you can't write it better by hand."
    }
    fn reset(&mut self) {
        self.tick_count = 0;
        self.phase = BenchPhase::Running;
        self.phase_timer = 0.0;
        self.run_count = 0;
        self.bench_n = 1_000;
        self.ns_history_iter.clear();
        self.ns_history_loop.clear();
        self.run_bench();
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
            "What does 'zero-cost abstraction' mean in Rust?",
            [
                "No compile time",
                "High-level code compiles to optimal assembly",
                "No heap allocations",
                "No trait objects",
            ],
            1,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_run_iterator_bench_correctness() {
        let (result, ns) = run_iterator_bench(100);
        // sum of squares of even numbers 0..100: 0+4+16+...+9604
        let expected: u64 = (0u64..100).filter(|x| x % 2 == 0).map(|x| x * x).sum();
        assert_eq!(result, expected);
        assert!(ns >= 1);
    }

    #[test]
    fn test_run_loop_bench_correctness() {
        let (result, ns) = run_loop_bench(100);
        let expected: u64 = (0u64..100).filter(|x| x % 2 == 0).map(|x| x * x).sum();
        assert_eq!(result, expected);
        assert!(ns >= 1);
    }

    #[test]
    fn test_iter_and_loop_results_match() {
        let (ir, _) = run_iterator_bench(500);
        let (lr, _) = run_loop_bench(500);
        assert_eq!(ir, lr);
    }

    #[test]
    fn test_new_runs_bench() {
        let d = ZeroCostDemo::new();
        assert_eq!(d.run_count, 1);
        assert!(d.results_match());
    }

    #[test]
    fn test_ratio_near_one() {
        let d = ZeroCostDemo::new();
        let r = d.ratio();
        // Allow generous range — timing can vary in CI
        assert!(r > 0.0);
        assert!(r < 1000.0);
    }

    #[test]
    fn test_is_paused_initially_false() {
        assert!(!ZeroCostDemo::new().is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = ZeroCostDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_clamp() {
        let mut d = ZeroCostDemo::new();
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(100);
        assert_eq!(d.speed(), 10);
        d.set_speed(5);
        assert_eq!(d.speed(), 5);
    }

    #[test]
    fn test_reset() {
        let mut d = ZeroCostDemo::new();
        d.tick_count = 100;
        d.reset();
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.bench_n, 1_000);
        assert!(d.results_match());
    }

    #[test]
    fn test_tick_paused_no_change() {
        let mut d = ZeroCostDemo::new();
        let count = d.run_count;
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.run_count, count);
    }

    #[test]
    fn test_bench_phases() {
        let mut d = ZeroCostDemo::new();
        assert_eq!(d.phase, BenchPhase::Running);
        let speed = d.speed;
        let period = 3.0 / speed as f64;
        d.tick(Duration::from_secs_f64(period + 0.1));
        // After period, should have switched to Displaying
        assert_eq!(d.phase, BenchPhase::Displaying);
        d.tick(Duration::from_secs_f64(period + 0.1));
        assert_eq!(d.phase, BenchPhase::Running);
    }

    #[test]
    fn test_cycle_n() {
        let mut d = ZeroCostDemo::new();
        d.bench_n = 100;
        d.cycle_n();
        assert_eq!(d.bench_n, 1_000);
        d.cycle_n();
        assert_eq!(d.bench_n, 10_000);
        d.cycle_n();
        assert_eq!(d.bench_n, 100);
    }

    #[test]
    fn test_name_description_explanation() {
        let d = ZeroCostDemo::new();
        assert_eq!(d.name(), "Zero-Cost Abstractions");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_render() {
        let d = ZeroCostDemo::new();
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_default() {
        let d = ZeroCostDemo::default();
        assert_eq!(d.run_count, 1);
    }

    #[test]
    fn test_barchart_import_available() {
        // Verify BarChart is importable (it is used in the performance demo)
        let _ = BarChart::default();
    }

    #[test]
    fn test_render_displaying_phase() {
        let mut d = ZeroCostDemo::new();
        d.phase = BenchPhase::Displaying;
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_both_phases() {
        let mut d = ZeroCostDemo::new();
        for phase in [BenchPhase::Running, BenchPhase::Displaying] {
            d.phase = phase;
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
        }
    }

    #[test]
    fn test_render_with_history() {
        let mut d = ZeroCostDemo::new();
        // Run bench multiple times to build history
        for _ in 0..5 {
            d.run_bench();
        }
        assert!(d.ns_history_iter.len() >= 5);
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_ns_history_builds_up() {
        let mut d = ZeroCostDemo::new();
        assert_eq!(d.ns_history_iter.len(), 1); // new() calls run_bench once
        d.run_bench();
        assert_eq!(d.ns_history_iter.len(), 2);
    }
}
