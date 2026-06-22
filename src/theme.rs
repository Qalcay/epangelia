use ratatui::style::Color;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub theme_fg: Color,
    pub theme_bg: Color,
    pub color_red: Color,
    pub color_green: Color,
    pub color_yellow: Color,
    pub color_cyan: Color,
    pub color_blue: Color,
    pub color_magenta: Color,
    pub color_gray: Color,
    pub color_white: Color,
}

impl Theme {
    pub fn light_mode() -> Theme {
        Theme {
            theme_fg: Color::Rgb(33, 33, 33),
            theme_bg: Color::Rgb(234, 202, 171),
            color_red: Color::Rgb(198, 22, 22),
            color_green: Color::Rgb(33, 121, 44),
            color_yellow: Color::Rgb(180, 128, 2),
            color_cyan: Color::Rgb(2, 128, 180),
            color_blue: Color::Rgb(44, 77, 189),
            color_magenta: Color::Rgb(167, 45, 167),
            color_gray: Color::Rgb(50, 50, 50),
            color_white: Color::Rgb(22, 22, 22),
        }
    }

    pub fn dark_mode() -> Theme {
        Theme {
            theme_fg: Color::Rgb(234, 234, 234),
            theme_bg: Color::Rgb(11, 11, 7),
            color_red: Color::Rgb(234, 45, 45),
            color_green: Color::Rgb(50, 202, 99),
            color_yellow: Color::Rgb(240, 128, 60),
            color_cyan: Color::Rgb(18, 220, 240),
            color_blue: Color::Rgb(99, 144, 222),
            color_magenta: Color::Rgb(189, 81, 198),
            color_gray: Color::Rgb(99, 99, 99),
            color_white: Color::Rgb(222, 222, 222),
        }
    }

    pub fn step_color(&self, step: u32) -> Color {
        match step {
            1..=2 => self.color_gray, 3..=4 => self.color_white, 5..=6 => self.color_green,
            7..=8 => self.color_blue, 9..=10 => self.color_magenta, _ => self.color_yellow,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark_mode()
    }
}
