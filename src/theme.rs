use ratatui::style::{Color, Modifier, Style};

pub const RUST_ORANGE: Color = Color::Rgb(222, 99, 29);
pub const CRAB_RED: Color = Color::Rgb(180, 40, 40);
pub const SAFE_GREEN: Color = Color::Rgb(50, 200, 100);
pub const BORROW_YELLOW: Color = Color::Rgb(220, 180, 50);
pub const HEAP_BLUE: Color = Color::Rgb(60, 120, 220);
pub const STACK_CYAN: Color = Color::Rgb(50, 200, 210);
pub const ASYNC_PURPLE: Color = Color::Rgb(150, 80, 220);
#[allow(dead_code)]
pub const DARK_BG: Color = Color::Rgb(20, 20, 30);
#[allow(dead_code)]
pub const PANEL_BG: Color = Color::Rgb(30, 30, 45);
pub const TEXT_PRIMARY: Color = Color::White;
pub const TEXT_DIM: Color = Color::Rgb(150, 150, 165);

#[allow(dead_code)]
pub fn title_style() -> Style {
    Style::default()
        .fg(RUST_ORANGE)
        .add_modifier(Modifier::BOLD)
}

pub fn label_style() -> Style {
    Style::default().fg(TEXT_PRIMARY)
}

#[allow(dead_code)]
pub fn highlight_style() -> Style {
    Style::default().fg(SAFE_GREEN).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

#[allow(dead_code)]
pub fn error_style() -> Style {
    Style::default().fg(CRAB_RED).add_modifier(Modifier::BOLD)
}

// ─── Rainbow / Konami helpers ─────────────────────────────────────────────────

/// Convert HSV (h: 0..360, s: 0..1, v: 0..1) to (r, g, b) bytes.
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

/// Returns a fully-saturated rainbow `Color::Rgb` cycling at 3°/tick (120 tick period).
pub fn konami_color(tick: u64) -> ratatui::style::Color {
    let h = (tick * 3) % 360;
    let (r, g, b) = hsv_to_rgb(h as f64, 1.0, 1.0);
    ratatui::style::Color::Rgb(r, g, b)
}

/// Same as `konami_color` but with a per-character hue offset for marquee effects.
pub fn konami_color_offset(tick: u64, offset: u64) -> ratatui::style::Color {
    konami_color(tick.wrapping_add(offset * 8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        assert_eq!(RUST_ORANGE, Color::Rgb(222, 99, 29));
        assert_eq!(SAFE_GREEN, Color::Rgb(50, 200, 100));
        assert_eq!(BORROW_YELLOW, Color::Rgb(220, 180, 50));
        assert_eq!(HEAP_BLUE, Color::Rgb(60, 120, 220));
        assert_eq!(STACK_CYAN, Color::Rgb(50, 200, 210));
        assert_eq!(ASYNC_PURPLE, Color::Rgb(150, 80, 220));
        assert_eq!(DARK_BG, Color::Rgb(20, 20, 30));
        assert_eq!(PANEL_BG, Color::Rgb(30, 30, 45));
        assert_eq!(TEXT_PRIMARY, Color::White);
        assert_eq!(TEXT_DIM, Color::Rgb(150, 150, 165));
    }

    #[test]
    fn test_style_functions() {
        let ts = title_style();
        assert_eq!(ts.fg, Some(RUST_ORANGE));
        let ls = label_style();
        assert_eq!(ls.fg, Some(TEXT_PRIMARY));
        let hs = highlight_style();
        assert_eq!(hs.fg, Some(SAFE_GREEN));
        let ds = dim_style();
        assert_eq!(ds.fg, Some(TEXT_DIM));
        let es = error_style();
        assert_eq!(es.fg, Some(CRAB_RED));
    }
}
