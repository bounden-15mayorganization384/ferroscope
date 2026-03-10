use std::time::Duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{demos::Demo, theme};

/// State of a simulated async task in the executor model.
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncTaskState {
    Pending,
    Polling,
    Ready,
    Done,
}

impl AsyncTaskState {
    pub fn label(&self) -> &'static str {
        match self {
            AsyncTaskState::Pending => "PENDING",
            AsyncTaskState::Polling => "POLLING",
            AsyncTaskState::Ready   => "READY  ",
            AsyncTaskState::Done    => "DONE   ",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            AsyncTaskState::Pending => theme::TEXT_DIM,
            AsyncTaskState::Polling => theme::BORROW_YELLOW,
            AsyncTaskState::Ready   => theme::SAFE_GREEN,
            AsyncTaskState::Done    => theme::ASYNC_PURPLE,
        }
    }
}

/// A single simulated async task.
#[derive(Debug, Clone)]
pub struct AsyncTask {
    pub id: usize,
    pub label: &'static str,
    pub polls_needed: u64,
    pub polls_done: u64,
    pub state: AsyncTaskState,
}

impl AsyncTask {
    pub fn new(id: usize, label: &'static str, polls_needed: u64) -> Self {
        Self {
            id,
            label,
            polls_needed,
            polls_done: 0,
            state: AsyncTaskState::Pending,
        }
    }

    /// Progress fraction 0.0..=1.0
    pub fn progress(&self) -> f64 {
        if self.polls_needed == 0 {
            return 1.0;
        }
        (self.polls_done as f64 / self.polls_needed as f64).min(1.0)
    }
}

/// The async/await visualization demo.
#[derive(Debug)]
pub struct AsyncDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    tick_acc: f64,
    pub tasks: Vec<AsyncTask>,
    pub completed_count: u64,
    pub total_polls: u64,
    pub cycle_count: u64,
}

impl AsyncDemo {
    pub fn new() -> Self {
        let mut d = Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            tick_acc: 0.0,
            tasks: Vec::new(),
            completed_count: 0,
            total_polls: 0,
            cycle_count: 0,
        };
        d.init_tasks();
        d
    }

    fn init_tasks(&mut self) {
        self.tasks = vec![
            AsyncTask::new(0, "HTTP request",  5),
            AsyncTask::new(1, "File I/O",      3),
            AsyncTask::new(2, "Timer",         2),
            AsyncTask::new(3, "DB query",      4),
            AsyncTask::new(4, "DNS lookup",    2),
            AsyncTask::new(5, "TCP accept",    6),
        ];
        self.completed_count = 0;
        self.total_polls = 0;
        self.cycle_count = 0;
    }

    /// Run one poll cycle across all tasks.
    fn run_cycle(&mut self) {
        let (newly_completed, polls) = simulate_poll_cycle(&mut self.tasks);
        self.completed_count += newly_completed;
        self.total_polls += polls;
        self.cycle_count += 1;

        // If all tasks are done, restart after a short pause
        if self.tasks.iter().all(|t| t.state == AsyncTaskState::Done) {
            self.init_tasks();
        }
    }

    /// Seconds between executor poll cycles (depends on speed)
    pub fn cycle_period_secs(&self) -> f64 {
        0.6 / self.speed as f64
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state == AsyncTaskState::Pending).count()
    }

    pub fn polling_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state == AsyncTaskState::Polling).count()
    }

    pub fn ready_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state == AsyncTaskState::Ready).count()
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state == AsyncTaskState::Done).count()
    }
}

/// Simulate one round of executor polling across all tasks.
///
/// Transitions:
/// - Pending  -> Polling  (polls_done += 1)
/// - Polling  -> Polling  (polls_done += 1; if polls_done >= polls_needed -> Ready)
/// - Ready    -> Done
/// - Done     -> Done (no change)
///
/// Returns `(newly_completed, total_polls_this_cycle)`.
pub fn simulate_poll_cycle(tasks: &mut Vec<AsyncTask>) -> (u64, u64) {
    let mut newly_completed = 0u64;
    let mut total_polls = 0u64;

    for task in tasks.iter_mut() {
        match task.state {
            AsyncTaskState::Pending => {
                task.state = AsyncTaskState::Polling;
                task.polls_done += 1;
                total_polls += 1;
            }
            AsyncTaskState::Polling => {
                task.polls_done += 1;
                total_polls += 1;
                if task.polls_done >= task.polls_needed {
                    task.state = AsyncTaskState::Ready;
                }
            }
            AsyncTaskState::Ready => {
                task.state = AsyncTaskState::Done;
                newly_completed += 1;
            }
            AsyncTaskState::Done => {
                // No action — already finished
            }
        }
    }

    (newly_completed, total_polls)
}

impl Default for AsyncDemo {
    fn default() -> Self { Self::new() }
}

impl Demo for AsyncDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused { return; }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.tick_acc += dt.as_secs_f64();
        if self.tick_acc >= self.cycle_period_secs() {
            self.tick_acc = 0.0;
            self.run_cycle();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // title
                Constraint::Min(10),    // task list + explanation
                Constraint::Length(4),  // stats bar
            ])
            .split(area);

        // Title
        frame.render_widget(
            Paragraph::new(Span::styled(
                "Async/Await — Cooperative multitasking without threads",
                Style::default().fg(theme::RUST_ORANGE).add_modifier(Modifier::BOLD),
            ))
            .block(Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(theme::RUST_ORANGE))),
            chunks[0],
        );

        // Task list | Explanation
        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Task list with progress gauges
        let task_items: Vec<ListItem> = self.tasks.iter().map(|t| {
            let color = t.state.color();
            let pct = (t.progress() * 100.0) as u64;
            let bar_len = (t.progress() * 16.0) as usize;
            let bar_empty = 16usize.saturating_sub(bar_len);
            let bar = format!("[{}{}]", "█".repeat(bar_len), "░".repeat(bar_empty));
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  [{:>2}] {:14} ", t.id, t.label),
                    Style::default().fg(color),
                ),
                Span::styled(
                    t.state.label(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} {:3}%  polls: {}/{}", bar, pct, t.polls_done, t.polls_needed),
                    Style::default().fg(color),
                ),
            ]))
        }).collect();

        frame.render_widget(
            List::new(task_items)
                .block(Block::default().title("Simulated Executor Tasks").borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::ASYNC_PURPLE))),
            mid[0],
        );

        // Executor state breakdown
        let expl_lines = vec![
            Line::from(Span::styled("State Machine Model:", Style::default().fg(theme::BORROW_YELLOW).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(
                format!("  Pending : {:2}", self.pending_count()),
                Style::default().fg(theme::TEXT_DIM),
            )),
            Line::from(Span::styled(
                format!("  Polling : {:2}", self.polling_count()),
                Style::default().fg(theme::BORROW_YELLOW),
            )),
            Line::from(Span::styled(
                format!("  Ready   : {:2}", self.ready_count()),
                Style::default().fg(theme::SAFE_GREEN),
            )),
            Line::from(Span::styled(
                format!("  Done    : {:2}", self.done_count()),
                Style::default().fg(theme::ASYNC_PURPLE),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Future::poll() -> Poll::Pending",
                theme::dim_style(),
            )),
            Line::from(Span::styled(
                "Future::poll() -> Poll::Ready(v)",
                theme::dim_style(),
            )),
            Line::from(Span::styled(
                "Waker notifies executor to re-poll",
                theme::dim_style(),
            )),
        ];
        frame.render_widget(
            Paragraph::new(expl_lines)
                .block(Block::default().title("Executor State").borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::HEAP_BLUE)))
                .wrap(ratatui::widgets::Wrap { trim: true }),
            mid[1],
        );

        // Stats bar
        let stats = Line::from(vec![
            Span::styled(
                format!(" cycle #{:4}  ", self.cycle_count),
                Style::default().fg(theme::ASYNC_PURPLE),
            ),
            Span::styled(
                format!("total polls: {:6}  ", self.total_polls),
                Style::default().fg(theme::BORROW_YELLOW),
            ),
            Span::styled(
                format!("tasks completed: {:4}  ", self.completed_count),
                Style::default().fg(theme::SAFE_GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "No threads blocked. No stack per task.",
                theme::dim_style(),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(stats).block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }

    fn name(&self) -> &'static str { "Async/Await" }
    fn description(&self) -> &'static str { "Cooperative multitasking — futures and the state machine model." }
    fn explanation(&self) -> &'static str {
        "Rust's async/await desugars to state machines, not OS threads. \
        Each async fn becomes a Future: a struct implementing poll(). \
        The executor drives futures by calling poll() — if the result is \
        Poll::Pending, the future is parked and the executor moves on. \
        A Waker is registered so the OS (or IO driver) can notify the executor \
        when the future is ready to make progress. This enables millions of \
        concurrent tasks with near-zero overhead per task."
    }
    fn reset(&mut self) {
        self.tick_count = 0;
        self.tick_acc = 0.0;
        self.paused = false;
        self.init_tasks();
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
        let d = AsyncDemo::new();
        assert_eq!(d.name(), "Async/Await");
        assert!(!d.description().is_empty());
        assert!(!d.explanation().is_empty());
    }

    #[test]
    fn test_initial_tasks_count() {
        let d = AsyncDemo::new();
        assert_eq!(d.tasks.len(), 6);
    }

    #[test]
    fn test_initial_tasks_labels() {
        let d = AsyncDemo::new();
        let labels: Vec<&str> = d.tasks.iter().map(|t| t.label).collect();
        assert!(labels.contains(&"HTTP request"));
        assert!(labels.contains(&"File I/O"));
        assert!(labels.contains(&"Timer"));
        assert!(labels.contains(&"DB query"));
        assert!(labels.contains(&"DNS lookup"));
        assert!(labels.contains(&"TCP accept"));
    }

    #[test]
    fn test_initial_state_all_pending() {
        let d = AsyncDemo::new();
        for task in &d.tasks {
            assert_eq!(task.state, AsyncTaskState::Pending);
        }
    }

    #[test]
    fn test_is_paused_initially_false() {
        assert!(!AsyncDemo::new().is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = AsyncDemo::new();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed_and_clamp() {
        let mut d = AsyncDemo::new();
        d.set_speed(3);
        assert_eq!(d.speed(), 3);
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset_reinitializes_tasks() {
        let mut d = AsyncDemo::new();
        d.total_polls = 100;
        d.completed_count = 50;
        d.reset();
        assert_eq!(d.total_polls, 0);
        assert_eq!(d.completed_count, 0);
        assert_eq!(d.tasks.len(), 6);
        for t in &d.tasks {
            assert_eq!(t.state, AsyncTaskState::Pending);
        }
    }

    #[test]
    fn test_tick_paused_no_change() {
        let mut d = AsyncDemo::new();
        d.paused = true;
        d.tick(Duration::from_secs(100));
        assert_eq!(d.tick_count, 0);
        assert_eq!(d.total_polls, 0);
    }

    #[test]
    fn test_tick_advances_cycle() {
        let mut d = AsyncDemo::new();
        d.set_speed(10);
        // cycle_period = 0.6 / 10 = 0.06s
        d.tick(Duration::from_secs_f64(0.07));
        assert!(d.total_polls > 0, "Should have run at least one poll cycle");
    }

    // --- simulate_poll_cycle tests ---

    #[test]
    fn test_simulate_poll_cycle_pending_to_polling() {
        let mut tasks = vec![AsyncTask::new(0, "test", 3)];
        let (completed, polls) = simulate_poll_cycle(&mut tasks);
        assert_eq!(completed, 0);
        assert_eq!(polls, 1);
        assert_eq!(tasks[0].state, AsyncTaskState::Polling);
        assert_eq!(tasks[0].polls_done, 1);
    }

    #[test]
    fn test_simulate_poll_cycle_polling_increments() {
        let mut tasks = vec![AsyncTask::new(0, "test", 3)];
        tasks[0].state = AsyncTaskState::Polling;
        tasks[0].polls_done = 1;
        let (completed, polls) = simulate_poll_cycle(&mut tasks);
        assert_eq!(completed, 0);
        assert_eq!(polls, 1);
        assert_eq!(tasks[0].polls_done, 2);
        assert_eq!(tasks[0].state, AsyncTaskState::Polling);
    }

    #[test]
    fn test_simulate_poll_cycle_polling_to_ready() {
        let mut tasks = vec![AsyncTask::new(0, "test", 2)];
        tasks[0].state = AsyncTaskState::Polling;
        tasks[0].polls_done = 1; // one more poll makes it 2 = polls_needed
        let (completed, polls) = simulate_poll_cycle(&mut tasks);
        assert_eq!(polls, 1);
        assert_eq!(tasks[0].state, AsyncTaskState::Ready);
    }

    #[test]
    fn test_simulate_poll_cycle_ready_to_done() {
        let mut tasks = vec![AsyncTask::new(0, "test", 1)];
        tasks[0].state = AsyncTaskState::Ready;
        let (completed, polls) = simulate_poll_cycle(&mut tasks);
        assert_eq!(completed, 1);
        assert_eq!(polls, 0);
        assert_eq!(tasks[0].state, AsyncTaskState::Done);
    }

    #[test]
    fn test_simulate_poll_cycle_done_stays_done() {
        let mut tasks = vec![AsyncTask::new(0, "test", 1)];
        tasks[0].state = AsyncTaskState::Done;
        let (completed, polls) = simulate_poll_cycle(&mut tasks);
        assert_eq!(completed, 0);
        assert_eq!(polls, 0);
        assert_eq!(tasks[0].state, AsyncTaskState::Done);
    }

    #[test]
    fn test_simulate_poll_cycle_multiple_tasks() {
        let mut tasks = vec![
            AsyncTask::new(0, "a", 1),
            AsyncTask::new(1, "b", 2),
        ];
        // Run enough cycles to complete both
        let mut total_completed = 0u64;
        for _ in 0..10 {
            let (c, _) = simulate_poll_cycle(&mut tasks);
            total_completed += c;
        }
        assert!(total_completed >= 2);
        for t in &tasks {
            assert_eq!(t.state, AsyncTaskState::Done);
        }
    }

    #[test]
    fn test_async_task_progress() {
        let mut t = AsyncTask::new(0, "test", 4);
        assert_eq!(t.progress(), 0.0);
        t.polls_done = 2;
        assert!((t.progress() - 0.5).abs() < 0.001);
        t.polls_done = 4;
        assert_eq!(t.progress(), 1.0);
    }

    #[test]
    fn test_async_task_progress_zero_needed() {
        let t = AsyncTask::new(0, "test", 0);
        assert_eq!(t.progress(), 1.0);
    }

    #[test]
    fn test_async_task_state_labels() {
        assert!(!AsyncTaskState::Pending.label().is_empty());
        assert!(!AsyncTaskState::Polling.label().is_empty());
        assert!(!AsyncTaskState::Ready.label().is_empty());
        assert!(!AsyncTaskState::Done.label().is_empty());
    }

    #[test]
    fn test_async_task_state_colors() {
        let _ = AsyncTaskState::Pending.color();
        let _ = AsyncTaskState::Polling.color();
        let _ = AsyncTaskState::Ready.color();
        let _ = AsyncTaskState::Done.color();
    }

    #[test]
    fn test_pending_polling_ready_done_counts() {
        let d = AsyncDemo::new();
        // Initially all pending
        assert_eq!(d.pending_count(), 6);
        assert_eq!(d.polling_count(), 0);
        assert_eq!(d.ready_count(), 0);
        assert_eq!(d.done_count(), 0);
    }

    #[test]
    fn test_cycle_period_varies_with_speed() {
        let mut d = AsyncDemo::new();
        d.set_speed(1);
        let slow = d.cycle_period_secs();
        d.set_speed(10);
        let fast = d.cycle_period_secs();
        assert!(fast < slow);
    }

    #[test]
    fn test_tasks_restart_when_all_done() {
        let mut d = AsyncDemo::new();
        // Force all tasks to Done
        for t in &mut d.tasks {
            t.state = AsyncTaskState::Done;
        }
        // run_cycle() should detect all-done and call init_tasks()
        // Call tick with enough time to trigger a cycle
        d.set_speed(10);
        d.tick(Duration::from_secs_f64(0.07));
        // Tasks should have been re-initialized (all Pending)
        assert!(d.tasks.iter().all(|t| t.state != AsyncTaskState::Done)
            || d.tasks.iter().all(|t| t.state == AsyncTaskState::Pending)
            || d.tasks.len() == 6);
    }

    #[test]
    fn test_render() {
        let d = AsyncDemo::new();
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_render_after_several_cycles() {
        let mut d = AsyncDemo::new();
        d.set_speed(10);
        for _ in 0..20 {
            d.tick(Duration::from_secs_f64(0.1));
        }
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_default() {
        let d = AsyncDemo::default();
        assert_eq!(d.tasks.len(), 6);
    }
}
