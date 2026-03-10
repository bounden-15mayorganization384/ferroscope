use crate::{demos::Demo, theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

const STEPS: usize = 8;

/// Describes one animation step
#[derive(Debug, Clone)]
pub struct OwnershipStep {
    pub title: &'static str,
    pub code_lines: &'static [(&'static str, bool)], // (line, highlighted)
    pub explanation: &'static str,
    pub s1_state: VarState,
    pub s2_state: VarState,
    pub s3_state: VarState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarState {
    Hidden,
    Owned,
    Moved,    // grayed out
    Borrowed, // yellow
}

impl VarState {
    pub fn color(&self) -> Color {
        match self {
            VarState::Hidden => Color::Reset,
            VarState::Owned => crate::theme::SAFE_GREEN,
            VarState::Moved => crate::theme::TEXT_DIM,
            VarState::Borrowed => crate::theme::BORROW_YELLOW,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            VarState::Hidden => "",
            VarState::Owned => "✓ OWNED",
            VarState::Moved => "✗ MOVED",
            VarState::Borrowed => "& BORROWED",
        }
    }
}

#[derive(Debug)]
pub struct OwnershipDemo {
    paused: bool,
    speed: u8,
    pub tick_count: u64,
    pub step: usize,
    step_timer: f64,
    /// When true, renders a "Rust vs C++" split comparison instead of the normal view.
    pub vs_mode: bool,
}

impl OwnershipDemo {
    pub fn new() -> Self {
        Self {
            paused: false,
            speed: 1,
            tick_count: 0,
            step: 0,
            step_timer: 0.0,
            vs_mode: false,
        }
    }

    pub fn step_duration_secs(&self) -> f64 {
        2.0 / self.speed as f64
    }

    pub fn advance_step(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
    }

    pub fn step_info(&self) -> OwnershipStep {
        get_step(self.step)
    }

    /// Toggle the Rust vs C++ comparison view.
    pub fn toggle_vs_mode(&mut self) {
        self.vs_mode = !self.vs_mode;
    }
}

/// C++ use-after-free code lines used in the vs_mode comparison panel.
pub fn cpp_uaf_lines() -> &'static [&'static str] {
    &[
        "std::string* ptr = nullptr;",
        "{",
        "    std::string s = \"hello\";",
        "    ptr = &s;  // Borrowing address",
        "}  // s is destroyed here!",
        "std::cout << *ptr;  // USE-AFTER-FREE",
        "//  ^^ undefined behavior: ptr is dangling",
    ]
}

/// Equivalent Rust code that the borrow checker rejects at compile time.
pub fn rust_safe_lines() -> &'static [&'static str] {
    &[
        "let ptr: &String;",
        "{",
        "    let s = String::from(\"hello\");",
        "    ptr = &s;  // borrow of `s`",
        "}  // `s` dropped here while still borrowed",
        "println!(\"{}\", ptr);",
        "// error[E0597]: `s` does not live long enough",
    ]
}

pub fn get_step(step: usize) -> OwnershipStep {
    const CODE: &[(&str, bool)] = &[
        ("fn main() {", false),
        ("    let s1 = String::from(\"hello\");", false),
        ("    let s2 = s1;  // s1 moved to s2", false),
        ("    // s1 is invalid here!", false),
        ("    let s3 = s2.clone();  // deep copy", false),
        ("    takes_ownership(s3);  // s3 moved", false),
        ("    // s3 dropped at end of fn", false),
        ("    let x = 5; let y = x;  // i32: Copy", false),
        ("}", false),
    ];

    match step % STEPS {
        0 => OwnershipStep {
            title: "Step 1/8: Ownership — every value has one owner",
            code_lines: &CODE[0..2],
            explanation: "let s1 = String::from(\"hello\") allocates on heap. s1 owns the data.",
            s1_state: VarState::Owned,
            s2_state: VarState::Hidden,
            s3_state: VarState::Hidden,
        },
        1 => OwnershipStep {
            title: "Step 2/8: Move semantics — ownership transfers",
            code_lines: &CODE[2..3],
            explanation: "let s2 = s1 moves ownership. s1 is no longer valid.",
            s1_state: VarState::Moved,
            s2_state: VarState::Owned,
            s3_state: VarState::Hidden,
        },
        2 => OwnershipStep {
            title: "Step 3/8: s1 is invalidated after the move",
            code_lines: &CODE[3..4],
            explanation: "The borrow checker enforces this at compile time. No use-after-move.",
            s1_state: VarState::Moved,
            s2_state: VarState::Owned,
            s3_state: VarState::Borrowed,
        },
        3 => OwnershipStep {
            title: "Step 4/8: Clone — explicit deep copy",
            code_lines: &CODE[4..5],
            explanation: "s2.clone() creates a full heap copy. Both s2 and s3 are valid.",
            s1_state: VarState::Moved,
            s2_state: VarState::Owned,
            s3_state: VarState::Owned,
        },
        4 => OwnershipStep {
            title: "Step 5/8: Passing ownership to a function",
            code_lines: &CODE[5..6],
            explanation: "takes_ownership(s3) moves s3 into the function. s3 is gone.",
            s1_state: VarState::Moved,
            s2_state: VarState::Owned,
            s3_state: VarState::Moved,
        },
        5 => OwnershipStep {
            title: "Step 6/8: Drop — value freed at end of function scope",
            code_lines: &CODE[6..7],
            explanation: "RAII: Drop::drop() fires automatically. Memory freed. No GC needed.",
            s1_state: VarState::Moved,
            s2_state: VarState::Owned,
            s3_state: VarState::Moved,
        },
        6 => OwnershipStep {
            title: "Step 7/8: Copy types — no move, value duplicated",
            code_lines: &CODE[7..8],
            explanation: "i32 implements Copy. let y = x does NOT move x — both are valid.",
            s1_state: VarState::Moved,
            s2_state: VarState::Owned,
            s3_state: VarState::Hidden,
        },
        _ => OwnershipStep {
            title: "Step 8/8: Summary — Ownership at a glance",
            code_lines: CODE,
            explanation: "Every value has one owner. Ownership can be moved or borrowed. When owner leaves scope, value is dropped. Zero GC needed.",
            s1_state: VarState::Hidden,
            s2_state: VarState::Hidden,
            s3_state: VarState::Hidden,
        },
    }
}

impl Default for OwnershipDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo for OwnershipDemo {
    fn tick(&mut self, dt: Duration) {
        if self.paused {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.step_timer += dt.as_secs_f64();
        if self.step_timer >= self.step_duration_secs() {
            self.advance_step();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if self.vs_mode {
            self.render_vs_mode(frame, area);
            return;
        }

        let info = self.step_info();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(Span::styled(
            info.title,
            Style::default()
                .fg(theme::RUST_ORANGE)
                .add_modifier(Modifier::BOLD),
        )))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::RUST_ORANGE)),
        );
        frame.render_widget(title, chunks[0]);

        // Main view: variable boxes + code panel
        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[1]);

        // Variable state boxes
        let var_lines: Vec<Line> = vec![
            var_box_line("s1", &info.s1_state, "String(\"hello\")"),
            Line::from(""),
            var_box_line("s2", &info.s2_state, "String(\"hello\")"),
            Line::from(""),
            var_box_line("s3", &info.s3_state, "String(\"hello\") [clone]"),
        ];
        let var_panel = Paragraph::new(var_lines)
            .block(Block::default().title("Variables").borders(Borders::ALL));
        frame.render_widget(var_panel, mid[0]);

        // Code panel
        let mut code_panel = crate::ui::widgets::CodePanel::new("Code");
        for (line, _) in CODE_ALL {
            let highlighted = info.code_lines.iter().any(|(l, _)| *l == *line);
            code_panel.push_line(*line, highlighted);
        }
        code_panel.render(frame, mid[1]);

        // Explanation
        let expl = Paragraph::new(info.explanation)
            .block(
                Block::default()
                    .title("What's happening")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORROW_YELLOW)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(expl, chunks[2]);
    }

    fn name(&self) -> &'static str {
        "Ownership & Borrowing"
    }
    fn description(&self) -> &'static str {
        "Rust's compile-time memory safety — no GC required."
    }
    fn explanation(&self) -> &'static str {
        "Every value in Rust has exactly one owner. When the owner goes out of scope, \
        the value is dropped (freed) automatically via the Drop trait. \
        Ownership can be transferred (moved) or temporarily lent (borrowed). \
        The borrow checker enforces these rules at compile time — preventing \
        use-after-free, double-free, and data races without any runtime overhead."
    }
    fn reset(&mut self) {
        self.step = 0;
        self.step_timer = 0.0;
        self.tick_count = 0;
        self.paused = false;
        self.vs_mode = false;
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
            "After `let s2 = s1;` with a String, what happens to s1?",
            [
                "s1 is copied",
                "s1 is moved and invalid",
                "s1 is borrowed",
                "s1 is cloned",
            ],
            1,
        ))
    }

    fn supports_step_control(&self) -> bool {
        true
    }

    fn step_forward(&mut self) {
        self.step = (self.step + 1) % STEPS;
        self.step_timer = 0.0;
    }

    fn step_back(&mut self) {
        self.step = (self.step + STEPS - 1) % STEPS;
        self.step_timer = 0.0;
    }

    fn toggle_vsmode(&mut self) {
        self.toggle_vs_mode();
    }
}

impl OwnershipDemo {
    /// Renders the "Rust vs C++" use-after-free comparison split view.
    fn render_vs_mode(&self, frame: &mut Frame, area: Rect) {
        let vs_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(15)])
            .split(area);

        // Title bar
        frame.render_widget(
            Paragraph::new(Span::styled(
                "Rust vs C++ — Use-After-Free (press V to toggle)",
                Style::default()
                    .fg(theme::RUST_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::RUST_ORANGE)),
            ),
            vs_chunks[0],
        );

        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vs_chunks[1]);

        // Left: C++ danger panel
        let mut cpp_lines: Vec<Line> = cpp_uaf_lines()
            .iter()
            .map(|s| Line::from(Span::styled(*s, Style::default().fg(theme::CRAB_RED))))
            .collect();
        cpp_lines.push(Line::from(""));
        cpp_lines.push(Line::from(Span::styled(
            "❌ COMPILES — Undefined Behavior",
            Style::default()
                .fg(theme::CRAB_RED)
                .add_modifier(Modifier::BOLD),
        )));

        frame.render_widget(
            Paragraph::new(cpp_lines).block(
                Block::default()
                    .title("C++ (DANGER)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::CRAB_RED)),
            ),
            mid[0],
        );

        // Right: Rust safe panel
        let mut rust_lines: Vec<Line> = rust_safe_lines()
            .iter()
            .map(|s| Line::from(Span::styled(*s, Style::default().fg(theme::SAFE_GREEN))))
            .collect();
        rust_lines.push(Line::from(""));
        rust_lines.push(Line::from(Span::styled(
            "✓ COMPILE ERROR — Bug caught at compile time",
            Style::default()
                .fg(theme::SAFE_GREEN)
                .add_modifier(Modifier::BOLD),
        )));

        frame.render_widget(
            Paragraph::new(rust_lines).block(
                Block::default()
                    .title("Rust (SAFE)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::SAFE_GREEN)),
            ),
            mid[1],
        );
    }
}

fn var_box_line<'a>(name: &'static str, state: &VarState, value: &'static str) -> Line<'a> {
    if *state == VarState::Hidden {
        return Line::from(Span::styled(
            format!("  {}: <not yet declared>", name),
            theme::dim_style(),
        ));
    }
    Line::from(vec![
        Span::styled(format!("  {}: ", name), Style::default().fg(state.color())),
        Span::styled(value, Style::default().fg(state.color())),
        Span::styled(
            format!("  [{}]", state.label()),
            Style::default()
                .fg(state.color())
                .add_modifier(Modifier::DIM),
        ),
    ])
}

const CODE_ALL: &[(&str, bool)] = &[
    ("fn main() {", false),
    ("    let s1 = String::from(\"hello\");", false),
    ("    let s2 = s1;", false),
    ("    // s1 is invalid here!", false),
    ("    let s3 = s2.clone();", false),
    ("    takes_ownership(s3);", false),
    ("    // s3 dropped at end of fn", false),
    ("    let x = 5; let y = x;", false),
    ("}", false),
];

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn new_demo() -> OwnershipDemo {
        OwnershipDemo::new()
    }

    #[test]
    fn test_demo_trait_name() {
        assert_eq!(new_demo().name(), "Ownership & Borrowing");
    }

    #[test]
    fn test_demo_trait_description() {
        assert!(!new_demo().description().is_empty());
    }

    #[test]
    fn test_demo_trait_explanation() {
        assert!(!new_demo().explanation().is_empty());
    }

    #[test]
    fn test_is_paused_initially_false() {
        assert!(!new_demo().is_paused());
    }

    #[test]
    fn test_toggle_pause() {
        let mut d = new_demo();
        d.toggle_pause();
        assert!(d.is_paused());
        d.toggle_pause();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_set_speed() {
        let mut d = new_demo();
        d.set_speed(5);
        assert_eq!(d.speed(), 5);
    }

    #[test]
    fn test_set_speed_clamp_low() {
        let mut d = new_demo();
        d.set_speed(0);
        assert_eq!(d.speed(), 1);
    }

    #[test]
    fn test_set_speed_clamp_high() {
        let mut d = new_demo();
        d.set_speed(255);
        assert_eq!(d.speed(), 10);
    }

    #[test]
    fn test_reset() {
        let mut d = new_demo();
        d.step = 5;
        d.tick_count = 100;
        d.reset();
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
        assert!(!d.is_paused());
    }

    #[test]
    fn test_tick_advances_timer() {
        let mut d = new_demo();
        d.tick(Duration::from_secs_f64(0.1));
        assert!(d.step_timer > 0.0 || d.step == 1);
        assert_eq!(d.tick_count, 1);
    }

    #[test]
    fn test_tick_paused_no_advance() {
        let mut d = new_demo();
        d.paused = true;
        d.tick(Duration::from_secs(10));
        assert_eq!(d.step, 0);
        assert_eq!(d.tick_count, 0);
    }

    #[test]
    fn test_tick_advances_step_after_duration() {
        let mut d = new_demo();
        d.set_speed(10);
        // step_duration = 2.0 / 10 = 0.2s
        d.tick(Duration::from_secs_f64(0.3));
        assert_eq!(d.step, 1);
    }

    #[test]
    fn test_step_wraps_around() {
        let mut d = new_demo();
        d.step = STEPS - 1;
        d.advance_step();
        assert_eq!(d.step, 0);
    }

    #[test]
    fn test_all_steps_have_info() {
        for i in 0..STEPS {
            let info = get_step(i);
            assert!(!info.title.is_empty());
            assert!(!info.explanation.is_empty());
        }
    }

    #[test]
    fn test_var_state_colors_all_variants() {
        let _ = VarState::Hidden.color();
        let _ = VarState::Owned.color();
        let _ = VarState::Moved.color();
        let _ = VarState::Borrowed.color();
    }

    #[test]
    fn test_var_state_labels_all_variants() {
        assert_eq!(VarState::Hidden.label(), "");
        assert!(!VarState::Owned.label().is_empty());
        assert!(!VarState::Moved.label().is_empty());
        assert!(!VarState::Borrowed.label().is_empty());
    }

    #[test]
    fn test_step_duration_changes_with_speed() {
        let mut d = new_demo();
        d.set_speed(1);
        let slow = d.step_duration_secs();
        d.set_speed(10);
        let fast = d.step_duration_secs();
        assert!(fast < slow);
    }

    #[test]
    fn test_render_all_steps() {
        let mut d = new_demo();
        for _ in 0..STEPS {
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| d.render(f, f.area())).unwrap();
            d.advance_step();
        }
    }

    #[test]
    fn test_default() {
        let d = OwnershipDemo::default();
        assert_eq!(d.step, 0);
    }

    // ── New tests ─────────────────────────────────────────────────────────────

    #[test]
    fn test_vs_mode_default_false() {
        let d = OwnershipDemo::new();
        assert!(!d.vs_mode);
    }

    #[test]
    fn test_toggle_vs_mode() {
        let mut d = OwnershipDemo::new();
        d.toggle_vs_mode();
        assert!(d.vs_mode);
        d.toggle_vs_mode();
        assert!(!d.vs_mode);
    }

    #[test]
    fn test_render_vs_mode() {
        let mut d = OwnershipDemo::new();
        d.vs_mode = true;
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| d.render(f, f.area())).unwrap();
    }

    #[test]
    fn test_cpp_uaf_lines_nonempty() {
        let lines = cpp_uaf_lines();
        assert!(lines.len() >= 5, "expected >= 5 lines, got {}", lines.len());
        for line in lines {
            assert!(
                !line.is_empty(),
                "cpp_uaf_lines should have no empty entries"
            );
        }
    }

    #[test]
    fn test_rust_safe_lines_nonempty() {
        let lines = rust_safe_lines();
        assert!(lines.len() >= 5, "expected >= 5 lines, got {}", lines.len());
        for line in lines {
            assert!(
                !line.is_empty(),
                "rust_safe_lines should have no empty entries"
            );
        }
    }
}
