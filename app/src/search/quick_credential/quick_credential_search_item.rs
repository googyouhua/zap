use ordered_float::OrderedFloat;
use warp_core::ui::icons::Icon;
use warpui::{
    elements::{ConstrainedBox, Container, Flex, ParentElement, Text},
    AppContext, Element, SingletonEntity,
};

use crate::search::item::{IconLocation, SearchItem};
use crate::search::result_renderer::ItemHighlightState;
use crate::{appearance::Appearance, search::external_secrets::view::styles};

use super::searcher::QuickCredentialSearchItemAction;

const ICON_SIZE: f32 = 16.;

#[derive(Clone, Debug)]
pub struct QuickCredentialSearchItem {
    pub credential: warp_quick_credential::QuickCredential,
}

impl SearchItem for QuickCredentialSearchItem {
    type Action = QuickCredentialSearchItemAction;

    fn render_icon(
        &self,
        highlight_state: ItemHighlightState,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        Container::new(
            ConstrainedBox::new(
                Icon::Key
                    .to_warpui_icon(highlight_state.icon_fill(appearance).into())
                    .finish(),
            )
            .with_width(ICON_SIZE)
            .with_height(ICON_SIZE)
            .finish(),
        )
        .with_margin_right(12.)
        .finish()
    }

    fn icon_location(&self, appearance: &Appearance) -> IconLocation {
        let name_size = styles::name_font_size(appearance) * appearance.line_height_ratio();
        IconLocation::Top {
            margin_top: name_size - ICON_SIZE,
        }
    }

    fn render_item(
        &self,
        highlight_state: ItemHighlightState,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);

        let label = self.credential.label.clone();
        let username = self.credential.username.clone();

        let label_text = Text::new_inline(
            label,
            appearance.ui_font_family(),
            styles::name_font_size(appearance),
        )
        .with_color(highlight_state.main_text_fill(appearance).into_solid())
        .finish();

        let username_text = Text::new_inline(
            username,
            appearance.monospace_font_family(),
            appearance.monospace_font_size(),
        )
        .with_color(highlight_state.sub_text_fill(appearance).into_solid())
        .finish();

        Container::new(
            Flex::column()
                .with_child(label_text)
                .with_child(username_text)
                .finish(),
        )
        .with_padding_top(2.)
        .with_padding_bottom(2.)
        .finish()
    }

    fn render_details(&self, _ctx: &AppContext) -> Option<Box<dyn Element>> {
        None
    }

    fn score(&self) -> OrderedFloat<f64> {
        OrderedFloat(0.0)
    }

    fn accept_result(&self) -> QuickCredentialSearchItemAction {
        QuickCredentialSearchItemAction::SelectCredential(self.credential.clone())
    }

    fn execute_result(&self) -> QuickCredentialSearchItemAction {
        self.accept_result()
    }

    fn accessibility_label(&self) -> String {
        format!(
            "Credential: {} ({})",
            self.credential.label, self.credential.username
        )
    }
}
