use crate::text::{draw_text, measure_text, TextSystem};
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

const BUTTON_WIDTH: f32 = 160.0;
const BUTTON_HEIGHT: f32 = 32.0;
const BUTTON_RADIUS: f64 = 5.0;
const BUTTON_GAP: f32 = 10.0;

const ACCENT: Color = Color::from_rgb8(138, 92, 246);
const ACCENT_DIM: Color = Color::from_rgb8(80, 56, 150);
const TEXT_PRIMARY: Color = Color::from_rgb8(230, 230, 230);
const TEXT_SECONDARY: Color = Color::from_rgb8(140, 140, 155);

/// Action returned by hit-testing the welcome screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WelcomeAction {
    CreateVault,
    OpenVault,
    None,
}

/// Clickable region stored for hit testing.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn contains(&self, point_x: f32, point_y: f32) -> bool {
        point_x >= self.x
            && point_x <= self.x + self.width
            && point_y >= self.y
            && point_y <= self.y + self.height
    }
}

/// Compact centered launcher — title, subtitle, two action buttons.
pub struct WelcomeScreen {
    pub create_button: Rect,
    pub open_button: Rect,
}

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl WelcomeScreen {
    /// Builds a welcome screen with default (zero) button positions.
    pub fn new() -> Self {
        Self {
            create_button: Rect {
                x: -1.0,
                y: -1.0,
                width: 0.0,
                height: 0.0,
            },
            open_button: Rect {
                x: -1.0,
                y: -1.0,
                width: 0.0,
                height: 0.0,
            },
        }
    }

    /// Draws a compact centered launcher. All coordinates are in logical pixels.
    pub fn render(
        &mut self,
        scene: &mut Scene,
        text_system: &mut TextSystem,
        window_width: f32,
        window_height: f32,
    ) {
        // Vertical anchor: centre of button cluster sits at 45% of height (slightly above centre).
        let cluster_height = BUTTON_HEIGHT * 2.0 + BUTTON_GAP;
        let cluster_top = window_height * 0.45 - cluster_height / 2.0;

        // App name
        let title = "Onyx";
        let title_metrics = measure_text(text_system, title, 48.0);
        draw_text(
            scene,
            text_system,
            title,
            48.0,
            (
                (window_width - title_metrics.width) / 2.0,
                cluster_top - 80.0,
            ),
            TEXT_PRIMARY,
        );

        // Tagline
        let tagline = "Your calm writing space";
        let tagline_metrics = measure_text(text_system, tagline, 18.0);
        draw_text(
            scene,
            text_system,
            tagline,
            18.0,
            (
                (window_width - tagline_metrics.width) / 2.0,
                cluster_top - 32.0,
            ),
            TEXT_SECONDARY,
        );

        // Buttons sit side by side, centred horizontally.
        let total_buttons_width = BUTTON_WIDTH * 2.0 + BUTTON_GAP;
        let buttons_left = (window_width - total_buttons_width) / 2.0;

        let create_x = buttons_left;
        let open_x = buttons_left + BUTTON_WIDTH + BUTTON_GAP;
        let buttons_y = cluster_top;

        self.create_button = Rect {
            x: create_x,
            y: buttons_y,
            width: BUTTON_WIDTH,
            height: BUTTON_HEIGHT,
        };
        self.open_button = Rect {
            x: open_x,
            y: buttons_y,
            width: BUTTON_WIDTH,
            height: BUTTON_HEIGHT,
        };

        draw_button(scene, text_system, self.create_button, "Create vault", true);
        draw_button(scene, text_system, self.open_button, "Open vault", false);

        // Keyboard hint
        let hint = "C · create    O · open";
        let hint_metrics = measure_text(text_system, hint, 14.0);
        draw_text(
            scene,
            text_system,
            hint,
            14.0,
            (
                (window_width - hint_metrics.width) / 2.0,
                buttons_y + BUTTON_HEIGHT + 20.0,
            ),
            TEXT_SECONDARY,
        );
    }

    /// Returns the action for a click at the given logical-pixel position.
    pub fn hit_test(&self, point_x: f32, point_y: f32) -> WelcomeAction {
        if self.create_button.contains(point_x, point_y) {
            WelcomeAction::CreateVault
        } else if self.open_button.contains(point_x, point_y) {
            WelcomeAction::OpenVault
        } else {
            WelcomeAction::None
        }
    }
}

fn draw_button(
    scene: &mut Scene,
    text_system: &mut TextSystem,
    rect: Rect,
    label: &str,
    is_accent: bool,
) {
    let button_rect = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.width) as f64,
        (rect.y + rect.height) as f64,
        BUTTON_RADIUS,
    );

    let fill_color = if is_accent { ACCENT } else { ACCENT_DIM };
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(fill_color),
        None,
        &button_rect,
    );

    let label_metrics = measure_text(text_system, label, 18.0);
    let label_x = rect.x + (rect.width - label_metrics.width) / 2.0;
    let label_y = rect.y + (rect.height - label_metrics.height) / 2.0;
    draw_text(
        scene,
        text_system,
        label,
        18.0,
        (label_x, label_y),
        TEXT_PRIMARY,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_are_centred_horizontally() {
        let mut screen = WelcomeScreen::new();
        let mut text_system = TextSystem::new();
        let mut scene = Scene::new();

        screen.render(&mut scene, &mut text_system, 1200.0, 800.0);

        let create_mid = screen.create_button.x + screen.create_button.width / 2.0;
        let open_mid = screen.open_button.x + screen.open_button.width / 2.0;
        let window_mid = 1200.0 / 2.0;

        // The midpoint between both buttons should equal the window centre.
        assert!(((create_mid + open_mid) / 2.0 - window_mid).abs() < 1.0);
    }

    #[test]
    fn hit_test_create_button() {
        let mut screen = WelcomeScreen::new();
        let mut text_system = TextSystem::new();
        let mut scene = Scene::new();
        screen.render(&mut scene, &mut text_system, 1200.0, 800.0);

        let cx = screen.create_button.x + screen.create_button.width / 2.0;
        let cy = screen.create_button.y + screen.create_button.height / 2.0;
        assert_eq!(screen.hit_test(cx, cy), WelcomeAction::CreateVault);
    }

    #[test]
    fn hit_test_open_button() {
        let mut screen = WelcomeScreen::new();
        let mut text_system = TextSystem::new();
        let mut scene = Scene::new();
        screen.render(&mut scene, &mut text_system, 1200.0, 800.0);

        let cx = screen.open_button.x + screen.open_button.width / 2.0;
        let cy = screen.open_button.y + screen.open_button.height / 2.0;
        assert_eq!(screen.hit_test(cx, cy), WelcomeAction::OpenVault);
    }

    #[test]
    fn hit_test_empty_area() {
        let screen = WelcomeScreen::new();
        assert_eq!(screen.hit_test(0.0, 0.0), WelcomeAction::None);
    }
}
