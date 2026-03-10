use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme;

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    Idle,
    Running,
    Waiting,
    Done,
}

impl ThreadState {
    pub fn label(&self) -> &'static str {
        match self {
            ThreadState::Idle => "IDLE   ",
            ThreadState::Running => "RUNNING",
            ThreadState::Waiting => "WAITING",
            ThreadState::Done => "DONE   ",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            ThreadState::Idle => theme::TEXT_DIM,
            ThreadState::Running => theme::SAFE_GREEN,
            ThreadState::Waiting => theme::BORROW_YELLOW,
            ThreadState::Done => theme::STACK_CYAN,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThreadLane {
    pub thread_id: usize,
    pub state: ThreadState,
    pub label: String,
    pub progress: f64,
}

impl ThreadLane {
    pub fn new(id: usize, label: impl Into<String>) -> Self {
        Self {
            thread_id: id,
            state: ThreadState::Idle,
            label: label.into(),
            progress: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct ThreadLaneChart {
    pub lanes: Vec<ThreadLane>,
}

impl ThreadLaneChart {
    pub fn new(count: usize) -> Self {
        let lanes = (0..count)
            .map(|i| ThreadLane::new(i, format!("Thread-{}", i)))
            .collect();
        Self { lanes }
    }

    pub fn set_state(&mut self, id: usize, state: ThreadState) {
        if let Some(lane) = self.lanes.iter_mut().find(|l| l.thread_id == id) {
            lane.state = state;
        }
    }

    pub fn set_progress(&mut self, id: usize, progress: f64) {
        if let Some(lane) = self.lanes.iter_mut().find(|l| l.thread_id == id) {
            lane.progress = progress.clamp(0.0, 1.0);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().title("Threads").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line> = self.lanes.iter().map(|lane| {
            let bar_w = (inner.width.saturating_sub(30) as f64 * lane.progress) as usize;
            let bar: String = "█".repeat(bar_w);
            Line::from(vec![
                Span::styled(
                    format!(" T{:02} ", lane.thread_id),
                    Style::default().fg(theme::TEXT_DIM),
                ),
                Span::styled(
                    format!("[{}] ", lane.state.label()),
                    Style::default().fg(lane.state.color()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<20}", lane.label), theme::dim_style()),
                Span::styled(bar, Style::default().fg(lane.state.color())),
            ])
        }).collect();

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_new() {
        let chart = ThreadLaneChart::new(4);
        assert_eq!(chart.lanes.len(), 4);
        for (i, lane) in chart.lanes.iter().enumerate() {
            assert_eq!(lane.thread_id, i);
            assert_eq!(lane.state, ThreadState::Idle);
            assert_eq!(lane.progress, 0.0);
        }
    }

    #[test]
    fn test_set_state_all_variants() {
        let mut chart = ThreadLaneChart::new(4);
        chart.set_state(0, ThreadState::Running);
        chart.set_state(1, ThreadState::Waiting);
        chart.set_state(2, ThreadState::Done);
        chart.set_state(3, ThreadState::Idle);
        assert_eq!(chart.lanes[0].state, ThreadState::Running);
        assert_eq!(chart.lanes[1].state, ThreadState::Waiting);
        assert_eq!(chart.lanes[2].state, ThreadState::Done);
        assert_eq!(chart.lanes[3].state, ThreadState::Idle);
    }

    #[test]
    fn test_set_state_out_of_bounds_no_panic() {
        let mut chart = ThreadLaneChart::new(2);
        chart.set_state(99, ThreadState::Running); // should not panic
    }

    #[test]
    fn test_set_progress() {
        let mut chart = ThreadLaneChart::new(2);
        chart.set_progress(0, 0.75);
        assert!((chart.lanes[0].progress - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_progress_clamp() {
        let mut chart = ThreadLaneChart::new(2);
        chart.set_progress(0, -1.0);
        assert_eq!(chart.lanes[0].progress, 0.0);
        chart.set_progress(0, 2.0);
        assert_eq!(chart.lanes[0].progress, 1.0);
    }

    #[test]
    fn test_set_progress_out_of_bounds_no_panic() {
        let mut chart = ThreadLaneChart::new(2);
        chart.set_progress(99, 0.5); // should not panic
    }

    #[test]
    fn test_thread_state_labels() {
        assert_eq!(ThreadState::Idle.label(), "IDLE   ");
        assert_eq!(ThreadState::Running.label(), "RUNNING");
        assert_eq!(ThreadState::Waiting.label(), "WAITING");
        assert_eq!(ThreadState::Done.label(), "DONE   ");
    }

    #[test]
    fn test_thread_state_colors() {
        // Just ensure all arms return a color without panic
        let _ = ThreadState::Idle.color();
        let _ = ThreadState::Running.color();
        let _ = ThreadState::Waiting.color();
        let _ = ThreadState::Done.color();
    }

    #[test]
    fn test_render() {
        let mut chart = ThreadLaneChart::new(4);
        chart.set_state(0, ThreadState::Running);
        chart.set_progress(0, 0.5);
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| chart.render(f, f.area())).unwrap();
    }
}
