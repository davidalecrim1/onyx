use crate::text::TextSystem;
use crate::ui::theme::Theme;
use vello::Scene;

/// Bundles rendering dependencies into a single reference passed to paint methods.
pub struct DrawContext<'a> {
    pub scene: &'a mut Scene,
    pub text: &'a mut TextSystem,
    pub theme: &'a Theme,
}
