use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

const STEPS: usize = 7;

/// Visual state for a single thread in the animation.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadVizState {
    Spawning,
    Running,
    Waiting,
    Done,
}

impl ThreadVizState {
    #[allow(dead_code)]
    pub fn symbol(&self) -> &'static str {
        match self {
            ThreadVizState::Spawning => "⟳ SPAWNING",
            ThreadVizState::Running => "► RUNNING ",
            ThreadVizState::Waiting => "⧗ WAITING ",
            ThreadVizState::Done => "✓ DONE    ",
        }
    }

    #[allow(dead_code)]
    pub fn color(&self) -> ratatui::style::Color {
        match self {
            ThreadVizState::Spawning => theme::BORROW_YELLOW,
            ThreadVizState::Running => theme::SAFE_GREEN,
            ThreadVizState::Waiting => theme::CRAB_RED,
            ThreadVizState::Done => theme::TEXT_DIM,
        }
    }
}

/// Visualization of one thread.
#[derive(Debug, Clone)]
pub struct ThreadViz {
    pub id: usize,
    #[allow(dead_code)]
    pub label: String,
    pub state: ThreadVizState,
    pub progress: f64, // 0.0..=1.0
}

/// The main concurrency demo struct.
#[derive(Debug)]
pub struct ConcurrencyDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    pub threads: Vec<ThreadViz>,
    pub channel_msgs: Vec<String>,
    pub rayon_speedup: f64,
    pub mutex_contention_count: u64,
    /// Animation frame index for the data race visualization (step 6). Wraps 0..=15.
    pub data_race_frame: usize,
}

impl ConcurrencyDemo {
    pub fn new() -> Self {
        let mut d = Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            threads: Vec::new(),
            channel_msgs: Vec::new(),
            rayon_speedup: 0.0,
            mutex_contention_count: 0,
            data_race_frame: 0,
        };
        d.apply_step();
        d
    }

    pub fn step_duration_secs(&self) -> f64 {
        2.5 / self.speed as f64
    }

    fn apply_step(&mut self) {
        match self.step {
            0 => {
                // Initial: single main thread
                self.threads.clear();
                self.channel_msgs.clear();
                self.mutex_contention_count = 0;
                self.rayon_speedup = 0.0;
                self.threads.push(ThreadViz {
                    id: 0,
                    label: "main".into(),
                    state: ThreadVizState::Running,
                    progress: 0.0,
                });
            }
            1 => {
                // Spawn worker threads
                self.channel_msgs.push("thread::spawn() x4".into());
                for i in 1..=4 {
                    self.threads.push(ThreadViz {
                        id: i,
                        label: format!("worker-{}", i),
                        state: ThreadVizState::Spawning,
                        progress: 0.0,
                    });
                }
            }
            2 => {
                // Threads running — channel message passing
                for t in &mut self.threads {
                    if t.id > 0 {
                        t.state = ThreadVizState::Running;
                        t.progress = 0.4;
                    }
                }
                self.channel_msgs
                    .push("tx.send(\"data from worker-1\")".into());
                self.channel_msgs
                    .push("rx.recv() -> Ok(\"data from worker-1\")".into());
                self.channel_msgs
                    .push("Channel: ownership transferred, no copy".into());
            }
            3 => {
                // Mutex contention: some threads waiting for the lock
                self.mutex_contention_count = 3;
                for t in &mut self.threads {
                    match t.id {
                        2 | 4 => t.state = ThreadVizState::Waiting,
                        _ => {}
                    }
                }
                self.channel_msgs
                    .push("Mutex::lock() — T2 and T4 blocked".into());
                self.channel_msgs
                    .push("Rust: lock() returns MutexGuard (RAII)".into());
                self.channel_msgs
                    .push("Guard dropped = lock released automatically".into());
            }
            4 => {
                // Rayon parallel sort — all threads active
                for t in &mut self.threads {
                    t.state = ThreadVizState::Running;
                    t.progress = 0.8;
                }
                self.rayon_speedup = compute_rayon_speedup(10_000);
                self.channel_msgs.push(format!(
                    "rayon par_sort_unstable(10k): {:.2}x speedup",
                    self.rayon_speedup
                ));
                self.channel_msgs
                    .push("Work-stealing scheduler, zero-copy data sharing".into());
            }
            5 => {
                // All threads joined
                for t in &mut self.threads {
                    t.state = ThreadVizState::Done;
                    t.progress = 1.0;
                }
                self.channel_msgs
                    .push("thread.join() — all workers complete".into());
                self.channel_msgs
                    .push("No data races. No undefined behavior. Guaranteed.".into());
            }
            _ => {
                // Step 6: data race demo
                self.channel_msgs.push(
                    "⚠ Data Race Demo: Two threads read-modify-write counter without synchronization".into()
                );
            }
        }
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
        self.apply_step();
    }
}

/// Animation frame strings illustrating a non-atomic data race in progress.
pub fn data_race_frames() -> &'static [&'static str] {
    &[
        "Thread A reads counter = 5",
        "Thread B reads counter = 5  <- same value!",
        "Thread A: computing 5 + 1...",
        "Thread B: computing 5 + 1...",
        "Thread A writes counter = 6",
        "Thread B writes counter = 6  <- LOST UPDATE!",
        "Expected: counter = 7  |  Actual: counter = 6",
        "DATA RACE: non-atomic read-modify-write lost 1 update",
    ]
}

/// Compute speedup of rayon parallel sort vs sequential sort on n random u32s.
/// Returns the speedup ratio (sequential_ns / parallel_ns), clamped to >= 0.1.
pub fn compute_rayon_speedup(n: usize) -> f64 {
    use rand::Rng;
    use rayon::prelude::*;

    let mut rng = rand::thread_rng();
    let data: Vec<u32> = (0..n).map(|_| rng.gen()).collect();

    // Sequential sort
    let mut seq = data.clone();
    let t0 = Instant::now();
    seq.sort_unstable();
    let seq_ns = t0.elapsed().as_nanos() as f64;

    // Parallel sort (rayon)
    let mut par = data;
    let t1 = Instant::now();
    par.par_sort_unstable();
    let par_ns = t1.elapsed().as_nanos() as f64;

    if par_ns < 1.0 {
        return 1.0;
    }
    (seq_ns / par_ns).max(0.1)
}

pub fn step_title(step: usize) -> &'static str {
    match step % STEPS {
        0 => "Step 1/7: Single-threaded — main thread only",
        1 => "Step 2/7: thread::spawn — worker threads created",
        2 => "Step 3/7: Channels — message passing between threads",
        3 => "Step 4/7: Mutex — shared state with contention",
        4 => "Step 5/7: Rayon — data parallelism with par_sort",
        5 => "Step 6/7: Join — all threads complete safely",
        _ => "Step 7/7: Data Race — why Rust makes them impossible",
    }
}

impl Default for ConcurrencyDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for ConcurrencyDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
        // Animate running thread progress bars
        for t in &mut self.threads {
            if t.state == ThreadVizState::Running {
                t.progress = (t.progress + dt.as_secs_f64() * 0.25).min(1.0);
            }
        }
        // Increment data race animation frame when on step 6
        if self.step == 6 {
            self.data_race_frame = (self.data_race_frame + 1) % 16;
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
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::RUST_ORANGE)),
            ),
            chunks[0],
        );

        if self.step == 6 {
            // Data race demo: two-panel layout
            let dr_panels = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // Left panel: animated data race sequence
            let frame_idx = self.data_race_frame % data_race_frames().len();
            let race_text = data_race_frames()[frame_idx];
            let left_lines = vec![
                Line::from(Span::styled(
                    "Two unsynchronized threads race on a shared counter:",
                    theme::dim_style(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    race_text,
                    Style::default()
                        .fg(theme::CRAB_RED)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Result: lost update / torn read — undefined behavior",
                    Style::default().fg(theme::CRAB_RED),
                )),
            ];
            frame.render_widget(
                Paragraph::new(left_lines).block(
                    Block::default()
                        .title("Data Race — Thread Interleaving")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::CRAB_RED)),
                ),
                dr_panels[0],
            );

            // Right panel: Rust compile error
            let right_lines = vec![
                Line::from(Span::styled(
                    "In Rust: this code does NOT compile.",
                    Style::default()
                        .fg(theme::SAFE_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Arc<Mutex<T>> required.",
                    Style::default().fg(theme::SAFE_GREEN),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "error[E0502]: cannot borrow `counter` as mutable",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(Span::styled(
                    "because it is also borrowed as immutable",
                    Style::default().fg(theme::BORROW_YELLOW),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "The type system prevents data races at compile time.",
                    theme::dim_style(),
                )),
            ];
            frame.render_widget(
                Paragraph::new(right_lines).block(
                    Block::default()
                        .title("Rust: Compile Error")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::SAFE_GREEN)),
                ),
                dr_panels[1],
            );
        } else {
            // Mid: thread panel | event log
            let mid = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(chunks[1]);

            let mut chart = crate::ui::widgets::ThreadLaneChart::new(self.threads.len());
            for thread in &self.threads {
                let ws = match thread.state {
                    ThreadVizState::Spawning | ThreadVizState::Running => {
                        crate::ui::widgets::ThreadState::Running
                    }
                    ThreadVizState::Waiting => crate::ui::widgets::ThreadState::Waiting,
                    ThreadVizState::Done => crate::ui::widgets::ThreadState::Done,
                };
                chart.set_state(thread.id, ws);
                chart.set_progress(thread.id, thread.progress);
            }
            chart.render(frame, mid[0]);

            // Channel / event log
            let log_items: Vec<ListItem> = self
                .channel_msgs
                .iter()
                .map(|m| {
                    ListItem::new(Line::from(Span::styled(
                        format!("  > {}", m),
                        theme::dim_style(),
                    )))
                })
                .collect();
            frame.render_widget(
                List::new(log_items).block(
                    Block::default()
                        .title("Events / Channel Log")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::HEAP_BLUE)),
                ),
                mid[1],
            );
        }

        // Stats bar
        let speedup_color = if self.rayon_speedup >= 1.5 {
            theme::SAFE_GREEN
        } else if self.rayon_speedup > 0.0 {
            theme::BORROW_YELLOW
        } else {
            theme::TEXT_DIM
        };
        let speedup_str = if self.rayon_speedup > 0.0 {
            format!("{:.2}x", self.rayon_speedup)
        } else {
            "pending".into()
        };
        let stats = Line::from(vec![
            Span::styled(
                format!(" threads: {}  ", self.threads.len()),
                Style::default().fg(theme::ASYNC_PURPLE),
            ),
            Span::styled(
                format!("mutex contention: {}  ", self.mutex_contention_count),
                Style::default().fg(theme::CRAB_RED),
            ),
            Span::styled(
                format!("rayon speedup: {}", speedup_str),
                Style::default()
                    .fg(speedup_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(stats).block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }

    fn name(&self) -> &'static str {
        "Fearless Concurrency"
    }
    fn description(&self) -> &'static str {
        "Thread safety enforced at compile time — data races are impossible."
    }
    fn explanation(&self) -> &'static str {
        "Rust's ownership and type system makes data races impossible at compile time. \
        The Send and Sync traits control what can cross thread boundaries. \
        Channels (mpsc) enable safe message passing. Mutex<T> guards shared state. \
        Rayon provides data parallelism with a work-stealing scheduler. \
        Arc<T> enables shared ownership across threads — all without a GC."
    }
    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.threads.clear();
        self.channel_msgs.clear();
        self.rayon_speedup = 0.0;
        self.mutex_contention_count = 0;
        self.data_race_frame = 0;
        self.paused = false;
        self.apply_step();
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
            "What prevents data races in Rust at compile time?",
            [
                "Garbage collector",
                "The GIL",
                "Ownership and Send/Sync traits",
                "Runtime locks",
            ],
            2,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_name_description_explanation() {
        let d = ConcurrencyDemo::new();
        assert_eq!(d.name(), "Fearless Concurrency");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() {
        assert!(!ConcurrencyDemo::new().is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = ConcurrencyDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = ConcurrencyDemo::new();
        d.set_speed(5);
        assert_eq!(d.speed(), 5);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(200);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = ConcurrencyDemo::new();
        d.step = 4;
        d.tick_count = 999;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert!(!d.is_paused());
        // After reset, step 0 is applied: one thread (main) present
        assert_eq!(d.threads.len(), 1);
    }

    #[test]
    fn test_tick_paused_no_advance() {
        let mut d = ConcurrencyDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_tick_advances_step() {
        let mut d = ConcurrencyDemo::new();
        d.set_speed(10);
        // step_duration = 2.5 / 10 = 0.25s
        d.tick(Duration::from_secs_f64(0.3));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_all_seven_steps() {
        let mut d = ConcurrencyDemo::new();
        for i in 0..STEPS {
            let t = step_title(i);
            assert!(!t.is_empty());
            d.advance_step();
        }
        // Should wrap back to 0
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_step_1_spawns_threads() {
        let mut d = ConcurrencyDemo::new();
        assert_eq!(d.threads.len(), 1); // main only
        d.advance_step(); // step 1
        assert_eq!(d.threads.len(), 5); // main + 4 workers
    }

    #[test]
    fn test_step_2_threads_running() {
        let mut d = ConcurrencyDemo::new();
        d.advance_step(); // step 1: spawn
        d.advance_step(); // step 2: running
        let running = d
            .threads
            .iter()
            .filter(|t| t.state == ThreadVizState::Running)
            .count();
        assert!(running >= 1);
    }

    #[test]
    fn test_step_3_mutex_contention() {
        let mut d = ConcurrencyDemo::new();
        for _ in 0..3 {
            d.advance_step();
        }
        assert!(d.mutex_contention_count > 0);
        let waiting = d
            .threads
            .iter()
            .filter(|t| t.state == ThreadVizState::Waiting)
            .count();
        assert!(waiting > 0);
    }

    #[test]
    fn test_step_5_done_state() {
        let mut d = ConcurrencyDemo::new();
        for _ in 0..5 {
            d.advance_step();
        }
        let done = d
            .threads
            .iter()
            .filter(|t| t.state == ThreadVizState::Done)
            .count();
        assert!(done > 0);
    }

    #[test]
    fn test_thread_viz_state_all_variants() {
        assert!(!ThreadVizState::Spawning.symbol().is_empty());
        assert!(!ThreadVizState::Running.symbol().is_empty());
        assert!(!ThreadVizState::Waiting.symbol().is_empty());
        assert!(!ThreadVizState::Done.symbol().is_empty());
        let _ = ThreadVizState::Spawning.color();
        let _ = ThreadVizState::Running.color();
        let _ = ThreadVizState::Waiting.color();
        let _ = ThreadVizState::Done.color();
    }

    #[test]
    fn test_compute_rayon_speedup_returns_positive() {
        let speedup = compute_rayon_speedup(10_000);
        assert!(speedup >= 0.1, "speedup {} should be >= 0.1", speedup);
        // Very generous upper bound — on single-core CI, rayon may be slower
        assert!(speedup < 500.0, "speedup {} should be < 500.0", speedup);
    }

    #[test]
    fn test_compute_rayon_speedup_small_n() {
        // Very small n — results should still be a valid float
        let speedup = compute_rayon_speedup(100);
        assert!(speedup.is_finite());
        assert!(speedup > 0.0);
    }

    #[test]
    fn test_step_duration_varies_with_speed() {
        let mut d = ConcurrencyDemo::new();
        d.set_speed(1);
        let slow = d.step_duration_secs();
        d.set_speed(10);
        let fast = d.step_duration_secs();
        assert!(fast < slow);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = ConcurrencyDemo::new();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = ConcurrencyDemo::default();
        assert_eq!(d.step, 0);
        assert!(!d.threads.is_empty());
    }

    #[test]
    fn test_tick_count_increments() {
        let mut d = ConcurrencyDemo::new();
        d.tick(Duration::from_millis(10));
        assert_eq!(d.tick_count, 1);
        d.tick(Duration::from_millis(10));
        assert_eq!(d.tick_count, 2);
    }

    #[test]
    fn test_channel_msgs_accumulate() {
        let mut d = ConcurrencyDemo::new();
        for _ in 0..3 {
            d.advance_step();
        }
        assert!(!d.channel_msgs.is_empty());
    }

    // ── New tests ─────────────────────────────────────────────────────────────

    #[test]
    fn test_step_6_data_race_title() {
        let t = step_title(6);
        assert!(
            t.contains("Data Race"),
            "expected 'Data Race' in title, got: {}",
            t
        );
        assert!(!t.is_empty());
    }

    #[test]
    fn test_data_race_frames_nonempty() {
        let frames = data_race_frames();
        assert!(
            frames.len() >= 4,
            "expected >= 4 frames, got {}",
            frames.len()
        );
        for f in frames {
            assert!(!f.is_empty());
        }
    }

    #[test]
    fn test_data_race_frame_increments_on_step_6() {
        let mut d = ConcurrencyDemo::new();
        // Advance to step 6 (6 advance_step calls from step 0)
        for _ in 0..6 {
            d.advance_step();
        }
        assert_eq!(d.step, 6);
        assert_eq!(d.data_race_frame, 0);
        d.tick(Duration::from_millis(10));
        assert!(
            d.data_race_frame > 0,
            "data_race_frame should have incremented"
        );
    }

    #[test]
    fn test_data_race_frame_no_increment_before_step_6() {
        let mut d = ConcurrencyDemo::new();
        // step is 0; tick 10 times with small dt so step doesn't advance
        for _ in 0..10 {
            d.tick(Duration::from_millis(10));
        }
        assert_eq!(
            d.data_race_frame, 0,
            "data_race_frame should stay 0 on step 0"
        );
    }

    #[test]
    fn test_render_step_6() {
        let mut d = ConcurrencyDemo::new();
        for _ in 0..6 {
            d.advance_step();
        }
        assert_eq!(d.step, 6);
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }
}
