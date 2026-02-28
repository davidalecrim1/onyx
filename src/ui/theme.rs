use vello::peniko::Color;

/// Visual tokens for consistent colors across all screens.
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub separator: Color,
    pub border: Color,
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
    /// Dark theme inspired by Zed's One Dark palette.
    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb8(0x28, 0x2c, 0x33),
            surface: Color::from_rgb8(0x2f, 0x34, 0x3e),
            surface_hover: Color::from_rgb8(0x36, 0x3c, 0x46),
            surface_active: Color::from_rgb8(0x45, 0x4a, 0x56),
            separator: Color::from_rgb8(0x46, 0x4b, 0x57),
            border: Color::from_rgb8(0x36, 0x3c, 0x46),
            accent: Color::from_rgb8(0x74, 0xad, 0xe8),
            accent_dim: Color::from_rgb8(0x56, 0x83, 0xb0),
            text_primary: Color::from_rgb8(0xdc, 0xe0, 0xe5),
            text_secondary: Color::from_rgb8(0xa9, 0xaf, 0xbc),
            typography: Typography {
                title_size: 48.0,
                body_size: 16.0,
                small_size: 14.0,
                line_height_factor: 1.4,
            },
        }
    }
}
