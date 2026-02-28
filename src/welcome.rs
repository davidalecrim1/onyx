use taffy::style_helpers::{length, zero, TaffyMaxContent};
use taffy::{AlignItems, FlexDirection, Size, Style, TaffyTree};

use crate::ui::{Align, Button, DrawContext, HitId, HitSink, Label, Rect};

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

    /// Draws a compact centered launcher using Taffy for layout.
    pub fn render(
        &self,
        ctx: &mut DrawContext,
        hits: &mut HitSink,
        bounds: Rect,
    ) -> Result<(), taffy::TaffyError> {
        let title_size = ctx.theme.typography.title_size;
        let title_height = title_size * ctx.theme.typography.line_height_factor;
        let title_padding = 32.0;
        let total_content_height = title_height + title_padding + BUTTON_HEIGHT;
        let padding_top = bounds.height * 0.45 - total_content_height / 2.0;

        let mut tree: TaffyTree<()> = TaffyTree::new();

        let title_node = tree.new_leaf(Style {
            size: Size {
                width: length(bounds.width),
                height: length(title_height),
            },
            ..Default::default()
        })?;

        let spacer_node = tree.new_leaf(Style {
            size: Size {
                width: length(0.0),
                height: length(title_padding),
            },
            ..Default::default()
        })?;

        let create_node = tree.new_leaf(Style {
            size: Size {
                width: length(BUTTON_WIDTH),
                height: length(BUTTON_HEIGHT),
            },
            ..Default::default()
        })?;

        let open_node = tree.new_leaf(Style {
            size: Size {
                width: length(BUTTON_WIDTH),
                height: length(BUTTON_HEIGHT),
            },
            ..Default::default()
        })?;

        let button_row = tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Row,
                gap: Size {
                    width: length(BUTTON_GAP),
                    height: zero(),
                },
                ..Default::default()
            },
            &[create_node, open_node],
        )?;

        let content_column = tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                align_items: Some(AlignItems::Center),
                size: Size {
                    width: length(bounds.width),
                    height: taffy::Dimension::auto(),
                },
                ..Default::default()
            },
            &[title_node, spacer_node, button_row],
        )?;

        let root = tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(bounds.width),
                    height: length(bounds.height),
                },
                padding: taffy::Rect {
                    top: length(padding_top),
                    left: zero(),
                    right: zero(),
                    bottom: zero(),
                },
                ..Default::default()
            },
            &[content_column],
        )?;

        tree.compute_layout(root, Size::MAX_CONTENT)?;

        let root_layout = tree.layout(root)?;
        let root_rect = Rect::from_layout(root_layout, bounds.x, bounds.y);

        let content_layout = tree.layout(content_column)?;
        let content_rect = Rect::from_layout(content_layout, root_rect.x, root_rect.y);

        let title_layout = tree.layout(title_node)?;
        let title_rect = Rect::from_layout(title_layout, content_rect.x, content_rect.y);

        let button_row_layout = tree.layout(button_row)?;
        let button_row_rect = Rect::from_layout(button_row_layout, content_rect.x, content_rect.y);

        let create_layout = tree.layout(create_node)?;
        let create_rect = Rect::from_layout(create_layout, button_row_rect.x, button_row_rect.y);

        let open_layout = tree.layout(open_node)?;
        let open_rect = Rect::from_layout(open_layout, button_row_rect.x, button_row_rect.y);

        Label::new("Onyx", title_size, ctx.theme.text_primary)
            .align(Align::Center)
            .paint(ctx, title_rect);

        Button::new("Create vault", create_rect)
            .accent(true)
            .hit_id(HIT_CREATE_VAULT)
            .paint(ctx, hits);

        Button::new("Open vault", open_rect)
            .hit_id(HIT_OPEN_VAULT)
            .paint(ctx, hits);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::TextSystem;
    use crate::ui::Theme;
    use vello::Scene;

    const TEST_BOUNDS: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 1200.0,
        height: 800.0,
    };

    fn render_welcome() -> HitSink {
        let mut scene = Scene::new();
        let mut text_system = TextSystem::new();
        let theme = Theme::dark();
        let mut ctx = DrawContext {
            scene: &mut scene,
            text: &mut text_system,
            theme: &theme,
            cursor_position: (0.0, 0.0),
        };
        let mut hits = HitSink::new();
        let screen = WelcomeScreen::new();
        screen
            .render(&mut ctx, &mut hits, TEST_BOUNDS)
            .expect("layout should not fail");
        hits
    }

    fn buttons_y() -> f32 {
        let theme = Theme::dark();
        let title_height = theme.typography.title_size * theme.typography.line_height_factor;
        let title_padding = 32.0;
        let total_height = title_height + title_padding + BUTTON_HEIGHT;
        let top = TEST_BOUNDS.height * 0.45 - total_height / 2.0;
        top + title_height + title_padding
    }

    #[test]
    fn hit_test_create_button() {
        let hits = render_welcome();
        let total_width = BUTTON_WIDTH * 2.0 + BUTTON_GAP;
        let create_center_x = (TEST_BOUNDS.width - total_width) / 2.0 + BUTTON_WIDTH / 2.0;
        let create_center_y = buttons_y() + BUTTON_HEIGHT / 2.0;
        assert_eq!(
            hits.test(create_center_x, create_center_y),
            Some(HIT_CREATE_VAULT)
        );
    }

    #[test]
    fn hit_test_open_button() {
        let hits = render_welcome();
        let total_width = BUTTON_WIDTH * 2.0 + BUTTON_GAP;
        let open_center_x = (TEST_BOUNDS.width - total_width) / 2.0
            + BUTTON_WIDTH
            + BUTTON_GAP
            + BUTTON_WIDTH / 2.0;
        let open_center_y = buttons_y() + BUTTON_HEIGHT / 2.0;
        assert_eq!(
            hits.test(open_center_x, open_center_y),
            Some(HIT_OPEN_VAULT)
        );
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
