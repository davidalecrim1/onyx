use vello::peniko::Color;

/// Visual tokens for consistent colors across all screens.
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub separator: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub typography: Typography,
}

/// Font size and spacing tokens.
pub struct Typography {
    pub title_size: f32,
    pub body_size: f32,
    pub small_size: f32,
    pub line_height_factor: f32,
}

impl Theme {
    /// Dark theme matching the existing hardcoded colors.
    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb8(28, 28, 32),
            surface: Color::from_rgb8(32, 32, 38),
            separator: Color::from_rgb8(50, 50, 60),
            accent: Color::from_rgb8(138, 92, 246),
            accent_dim: Color::from_rgb8(80, 56, 150),
            text_primary: Color::from_rgb8(230, 230, 230),
            text_secondary: Color::from_rgb8(140, 140, 155),
            typography: Typography {
                title_size: 48.0,
                body_size: 18.0,
                small_size: 16.0,
                line_height_factor: 1.2,
            },
        }
    }
}
