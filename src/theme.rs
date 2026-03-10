use ratatui::style::{Color, Modifier, Style};

pub const RUST_ORANGE: Color = Color::Rgb(222, 99, 29);
pub const CRAB_RED: Color = Color::Rgb(180, 40, 40);
pub const SAFE_GREEN: Color = Color::Rgb(50, 200, 100);
pub const BORROW_YELLOW: Color = Color::Rgb(220, 180, 50);
pub const HEAP_BLUE: Color = Color::Rgb(60, 120, 220);
pub const STACK_CYAN: Color = Color::Rgb(50, 200, 210);
pub const ASYNC_PURPLE: Color = Color::Rgb(150, 80, 220);
pub const DARK_BG: Color = Color::Rgb(20, 20, 30);
pub const PANEL_BG: Color = Color::Rgb(30, 30, 45);
pub const TEXT_PRIMARY: Color = Color::White;
pub const TEXT_DIM: Color = Color::Rgb(150, 150, 165);

pub fn title_style() -> Style {
    Style::default().fg(RUST_ORANGE).add_modifier(Modifier::BOLD)
}

pub fn label_style() -> Style {
    Style::default().fg(TEXT_PRIMARY)
}

pub fn highlight_style() -> Style {
    Style::default().fg(SAFE_GREEN).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn error_style() -> Style {
    Style::default().fg(CRAB_RED).add_modifier(Modifier::BOLD)
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
