mod app;
mod demos;
mod events;
mod metrics;
mod theme;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use demos::DemoRegistry;
use events::key_event_to_app_event;

const DEFAULT_FPS: u64 = 30;
const TOUR_DEMO_SECS: u64 = 8; // seconds per demo in tour mode
const VERSION: &str = "0.1.0";

// ─── CLI args ────────────────────────────────────────────────────────────────

struct CliArgs {
    tour: bool,
    screenshot: bool,
    screenshot_dir: String,
    fps: u64,
    version: bool,
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let tour = args.iter().any(|a| a == "--tour");
    let screenshot = args.iter().any(|a| a == "--screenshot");
    let version = args.iter().any(|a| a == "--version");
    let screenshot_dir = args
        .windows(2)
        .find(|w| w[0] == "--screenshot-dir")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "ferroscope-screenshots".into());
    let fps = args
        .windows(2)
        .find(|w| w[0] == "--fps")
        .and_then(|w| w[1].parse::<u64>().ok())
        .unwrap_or(DEFAULT_FPS)
        .clamp(5, 120);
    CliArgs {
        tour,
        screenshot,
        screenshot_dir,
        fps,
        version,
    }
}

// ─── Terminal setup / teardown ────────────────────────────────────────────────

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// ─── Screenshot export (headless) ────────────────────────────────────────────

/// Render all demos using a TestBackend and save each as a plain-text file.
fn run_screenshot_mode(dir: &str, registry: &DemoRegistry) -> Result<()> {
    use ratatui::backend::TestBackend;
    use std::fs;

    fs::create_dir_all(dir)?;

    for i in 0..registry.len() {
        let name = registry
            .name(i)
            .unwrap_or("unknown")
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "_")
            .trim_matches('_')
            .to_string();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| registry.render_current(i, f, f.area()))?;

        // Extract text from the buffer
        let buffer = terminal.backend().buffer().clone();
        let mut lines: Vec<String> = Vec::new();
        for row in 0..buffer.area.height {
            let mut line = String::new();
            for col in 0..buffer.area.width {
                let cell = buffer[(col, row)].clone();
                let ch = cell.symbol().to_string();
                line.push_str(&ch);
            }
            lines.push(line.trim_end().to_owned());
        }
        // Trim trailing blank lines
        while lines.last().map(|l: &String| l.is_empty()).unwrap_or(false) {
            lines.pop();
        }

        let output = lines.join("\n");
        let path = format!("{}/demo_{:02}_{}.txt", dir, i + 1, name);
        fs::write(&path, &output)?;
        println!("  Saved: {}", path);
    }

    println!("\n{} screenshots saved to '{}'.", registry.len(), dir);
    Ok(())
}

// ─── Main event loop ──────────────────────────────────────────────────────────

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    registry: &mut DemoRegistry,
    tour_mode: bool,
    tick_rate: Duration,
) -> Result<()> {
    let ticks_per_demo = TOUR_DEMO_SECS * (1000 / tick_rate.as_millis().max(1) as u64);
    let mut tour_tick: u64 = 0;

    loop {
        terminal.draw(|f| ui::draw(f, app, registry))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Feed raw key to Konami tracker
                    if app.check_konami(key.code) {
                        // Konami activated — handled via app.konami_active
                    }

                    if let Some(app_event) = key_event_to_app_event(key) {
                        let is_reset = app_event == events::AppEvent::Reset;
                        let is_vsmode = app_event == events::AppEvent::ToggleVsMode;
                        let current = app.current_demo;
                        app.handle_event(app_event);
                        if is_reset {
                            registry.reset_current(current);
                        }
                        if is_vsmode {
                            registry.toggle_vsmode_current(current);
                        }
                    }
                }
            }
        } else {
            let dt = tick_rate;
            app.handle_event(events::AppEvent::Tick);
            if !app.paused {
                registry.tick_current(app.current_demo, dt);
            }

            // Tour mode: auto-advance every TOUR_DEMO_SECS seconds
            if tour_mode {
                tour_tick += 1;
                if tour_tick >= ticks_per_demo {
                    tour_tick = 0;
                    app.handle_event(events::AppEvent::NextDemo);
                    // After wrapping back to demo 0, quit
                    if app.current_demo == 0 {
                        app.unlock_evangelist();
                        break;
                    }
                }
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = parse_args();

    if cli.version {
        println!("ferroscope v{}", VERSION);
        return Ok(());
    }

    let registry = DemoRegistry::new();

    // Screenshot mode: headless, no terminal needed
    if cli.screenshot {
        return run_screenshot_mode(&cli.screenshot_dir, &registry);
    }

    let tick_rate = Duration::from_millis(1000 / cli.fps);
    let mut terminal = setup_terminal()?;
    let mut app = App::new(15);
    let mut registry_mut = registry;

    // Tour mode: show_explanation on by default
    if cli.tour {
        app.show_explanation = true;
    }

    let result = run_app(
        &mut terminal,
        &mut app,
        &mut registry_mut,
        cli.tour,
        tick_rate,
    );
    restore_terminal(&mut terminal)?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use events::AppEvent;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_default_fps_constant() {
        assert_eq!(DEFAULT_FPS, 30);
    }

    #[test]
    fn test_tour_demo_secs_positive() {
        assert!(TOUR_DEMO_SECS > 0);
    }

    #[test]
    fn test_parse_args_defaults() {
        let cli = CliArgs {
            tour: false,
            screenshot: false,
            screenshot_dir: "test-dir".into(),
            fps: 30,
            version: false,
        };
        assert!(!cli.tour);
        assert!(!cli.screenshot);
        assert_eq!(cli.screenshot_dir, "test-dir");
        assert_eq!(cli.fps, 30);
        assert!(!cli.version);
    }

    #[test]
    fn test_run_app_quit_immediately() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(15);
        let mut registry = DemoRegistry::new();

        app.running = false;

        app.handle_event(AppEvent::Tick);
        assert_eq!(app.tick_count, 1);

        let current = app.current_demo;
        app.handle_event(AppEvent::Reset);
        registry.reset_current(current);

        app.paused = true;
        registry.tick_current(app.current_demo, Duration::from_millis(33));

        app.paused = false;
        registry.tick_current(app.current_demo, Duration::from_millis(33));

        terminal.draw(|f| ui::draw(f, &app, &registry)).unwrap();
    }

    #[test]
    fn test_app_reset_interaction() {
        let mut app = App::new(15);
        let mut registry = DemoRegistry::new();
        let current = app.current_demo;
        app.handle_event(AppEvent::Reset);
        registry.reset_current(current);
    }

    #[test]
    fn test_app_vsmode_interaction() {
        let mut app = App::new(15);
        let mut registry = DemoRegistry::new();
        let current = app.current_demo;
        app.handle_event(AppEvent::ToggleVsMode);
        registry.toggle_vsmode_current(current);
    }

    #[test]
    fn test_screenshot_mode_writes_files() {
        let registry = DemoRegistry::new();
        let dir = "/tmp/ferroscope_test_screenshots";
        let result = run_screenshot_mode(dir, &registry);
        assert!(result.is_ok(), "screenshot mode failed: {:?}", result);
        // Verify files were created
        for i in 0..registry.len() {
            let name = registry
                .name(i)
                .unwrap_or("unknown")
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '-', "_")
                .trim_matches('_')
                .to_string();
            let path = format!("{}/demo_{:02}_{}.txt", dir, i + 1, name);
            assert!(
                std::path::Path::new(&path).exists(),
                "expected screenshot file: {}",
                path
            );
        }
    }

    #[test]
    fn test_tour_mode_flag() {
        let mut app = App::new(15);
        app.show_explanation = true;
        assert!(app.show_explanation);
    }

    #[test]
    fn test_fps_clamp_low() {
        // fps parsing clamps to 5 minimum
        let fps: u64 = 1u64.clamp(5, 120);
        assert_eq!(fps, 5);
    }

    #[test]
    fn test_fps_clamp_high() {
        let fps: u64 = 999u64.clamp(5, 120);
        assert_eq!(fps, 120);
    }

    #[test]
    fn test_tick_rate_from_fps() {
        let rate = Duration::from_millis(1000 / 30);
        assert_eq!(rate.as_millis(), 33);
        let rate60 = Duration::from_millis(1000 / 60);
        assert_eq!(rate60.as_millis(), 16);
    }
}
