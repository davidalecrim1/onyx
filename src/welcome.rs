use crate::text::{draw_text, measure_text};
use crate::ui::{Button, DrawContext, HitId, HitSink, Rect};

const BUTTON_WIDTH: f32 = 160.0;
const BUTTON_HEIGHT: f32 = 40.0;
const BUTTON_GAP: f32 = 10.0;

pub const HIT_CREATE_VAULT: HitId = HitId(1);
pub const HIT_OPEN_VAULT: HitId = HitId(2);

/// Action returned by hit-testing the welcome screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WelcomeAction {
    CreateVault,
    OpenVault,
    None,
}

impl WelcomeAction {
    /// Maps a hit id to the corresponding action.
    pub fn from_hit(id: HitId) -> Self {
        if id == HIT_CREATE_VAULT {
            Self::CreateVault
        } else if id == HIT_OPEN_VAULT {
            Self::OpenVault
        } else {
            Self::None
        }
    }
}

/// Compact centered launcher — title, subtitle, two action buttons.
pub struct WelcomeScreen;

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl WelcomeScreen {
    /// Builds a welcome screen.
    pub fn new() -> Self {
        Self
    }

    /// Draws a compact centered launcher. All coordinates are in logical pixels.
    pub fn render(
        &self,
        ctx: &mut DrawContext,
        hits: &mut HitSink,
        window_width: f32,
        window_height: f32,
    ) {
        let cluster_height = BUTTON_HEIGHT * 2.0 + BUTTON_GAP;
        let cluster_top = window_height * 0.45 - cluster_height / 2.0;

        let title = "Onyx";
        let title_size = ctx.theme.typography.title_size;
        let title_metrics = measure_text(ctx.text, title, title_size);
        draw_text(
            ctx.scene,
            ctx.text,
            title,
            title_size,
            (
                (window_width - title_metrics.width) / 2.0,
                cluster_top - 80.0,
            ),
            ctx.theme.text_primary,
        );

        let tagline = "Your calm writing space";
        let body_size = ctx.theme.typography.body_size;
        let tagline_metrics = measure_text(ctx.text, tagline, body_size);
        draw_text(
            ctx.scene,
            ctx.text,
            tagline,
            body_size,
            (
                (window_width - tagline_metrics.width) / 2.0,
                cluster_top - 32.0,
            ),
            ctx.theme.text_secondary,
        );

        let total_buttons_width = BUTTON_WIDTH * 2.0 + BUTTON_GAP;
        let buttons_left = (window_width - total_buttons_width) / 2.0;
        let buttons_y = cluster_top;

        let create_bounds = Rect::new(buttons_left, buttons_y, BUTTON_WIDTH, BUTTON_HEIGHT);
        Button::new("Create vault", create_bounds)
            .accent(true)
            .hit_id(HIT_CREATE_VAULT)
            .paint(ctx, hits);

        let open_bounds = Rect::new(
            buttons_left + BUTTON_WIDTH + BUTTON_GAP,
            buttons_y,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
        );
        Button::new("Open vault", open_bounds)
            .hit_id(HIT_OPEN_VAULT)
            .paint(ctx, hits);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::TextSystem;
    use crate::ui::Theme;
    use vello::Scene;

    fn render_welcome() -> HitSink {
        let mut scene = Scene::new();
        let mut text_system = TextSystem::new();
        let theme = Theme::dark();
        let mut ctx = DrawContext {
            scene: &mut scene,
            text: &mut text_system,
            theme: &theme,
        };
        let mut hits = HitSink::new();
        let screen = WelcomeScreen::new();
        screen.render(&mut ctx, &mut hits, 1200.0, 800.0);
        hits
    }

    #[test]
    fn hit_test_create_button() {
        let hits = render_welcome();
        let cluster_top = 800.0 * 0.45 - (BUTTON_HEIGHT * 2.0 + BUTTON_GAP) / 2.0;
        let total_width = BUTTON_WIDTH * 2.0 + BUTTON_GAP;
        let create_center_x = (1200.0 - total_width) / 2.0 + BUTTON_WIDTH / 2.0;
        let create_center_y = cluster_top + BUTTON_HEIGHT / 2.0;
        let result = hits.test(create_center_x, create_center_y);
        assert_eq!(result, Some(HIT_CREATE_VAULT));
    }

    #[test]
    fn hit_test_empty_area() {
        let hits = render_welcome();
        assert_eq!(hits.test(0.0, 0.0), None);
    }

    #[test]
    fn welcome_action_from_hit() {
        assert_eq!(
            WelcomeAction::from_hit(HIT_CREATE_VAULT),
            WelcomeAction::CreateVault
        );
        assert_eq!(
            WelcomeAction::from_hit(HIT_OPEN_VAULT),
            WelcomeAction::OpenVault
        );
        assert_eq!(WelcomeAction::from_hit(HitId(99)), WelcomeAction::None);
    }
}
