use itertools::Itertools;
use lazy_static::lazy_static;
use std::{collections::HashSet, ops::Range};

use warpui::{
    elements::{
        Align, ConstrainedBox, Container, CornerRadius, Dismiss, DispatchEventResult, Empty,
        EventHandler, Fill, Flex, Hoverable, MouseStateHandle, ParentElement, Radius, SavePosition,
        ScrollStateHandle,
        Scrollable, ScrollableElement, Shrinkable, UniformList, UniformListState,
    },
    platform::Cursor,
    presenter::ChildView,
    ui_components::components::{UiComponent, UiComponentStyles},
    AppContext, Element, Entity, EventContext, FocusContext, ModelHandle, SingletonEntity,
    TypedActionView, View, ViewContext, ViewHandle, WeakViewHandle,
};

use crate::{
    appearance::Appearance,
    search::{
        external_secrets::view::styles,
        quick_credential::{
            quick_credential_data_source::QuickCredentialDataSource,
            searcher::{QuickCredentialSearchItemAction, QuickCredentialSearchMixer},
        },
        result_renderer::{QueryResultRenderer, QueryResultRendererStyles},
        search_bar::{SearchBar, SearchBarEvent, SearchBarState, SearchResultOrdering},
    },
};

lazy_static! {
    static ref QUERY_RESULT_RENDERER_STYLES: QueryResultRendererStyles =
        QueryResultRendererStyles {
            result_item_height_fn: |appearance| {
                styles::line_height_sensitive_vertical_padding(appearance)
                    + styles::name_font_size(appearance)
            },
            panel_drop_shadow: styles::panel_drop_shadow(),
            panel_corner_radius: CornerRadius::with_all(Radius::Pixels(styles::CORNER_RADIUS)),
            result_vertical_padding: 4.,
            ..Default::default()
        };
}

const DEFAULT_PLACEHOLDER_TEXT: &str = "Search for a credential";

enum PanelMode {
    Searching,
    SendModeSelection {
        credential: warp_quick_credential::QuickCredential,
    },
}

pub struct QuickCredentialPanel {
    scroll_state: ScrollStateHandle,
    list_state: UniformListState,
    search_bar: ViewHandle<SearchBar<QuickCredentialSearchItemAction>>,
    search_bar_state: ModelHandle<SearchBarState<QuickCredentialSearchItemAction>>,
    mixer: ModelHandle<QuickCredentialSearchMixer>,
    handle: WeakViewHandle<Self>,
    mode: PanelMode,
}

#[derive(Clone, Debug)]
pub enum QuickCredentialPanelAction {
    ResultClicked {
        result_index: usize,
        result_action: Box<QuickCredentialSearchItemAction>,
    },
    SendPasswordOnly(warp_quick_credential::QuickCredential),
    SendUsernameThenPassword(warp_quick_credential::QuickCredential),
    Close,
}

pub enum QuickCredentialPanelEvent {
    ItemSelected {
        credential: warp_quick_credential::QuickCredential,
    },
    Close,
    Open,
}

impl QuickCredentialPanel {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let appearance = Appearance::as_ref(ctx);
        let ui_font_family = appearance.ui_font_family();

        let search_bar_state = ctx.add_model(|_| {
            SearchBarState::new(SearchResultOrdering::TopDown).run_query_on_buffer_empty()
        });

        ctx.observe(&search_bar_state, |_, _, ctx| {
            ctx.notify();
        });

        let mixer = ctx.add_model(|_| QuickCredentialSearchMixer::new());

        let search_bar = ctx.add_typed_action_view(|ctx| {
            SearchBar::new(
                mixer.clone(),
                search_bar_state.clone(),
                DEFAULT_PLACEHOLDER_TEXT,
                |result_index, result| {
                    QueryResultRenderer::new(
                        result,
                        format!("QueryResultRenderer:{result_index}"),
                        |result_index, result_action, event_ctx| {
                            event_ctx.dispatch_typed_action(
                                QuickCredentialPanelAction::ResultClicked {
                                    result_index,
                                    result_action: Box::new(result_action),
                                },
                            )
                        },
                        *QUERY_RESULT_RENDERER_STYLES,
                    )
                },
                ctx,
            )
            .with_font_family(ui_font_family, ctx)
        });

        ctx.subscribe_to_view(&search_bar, |me, _handle, event, ctx| {
            me.handle_search_bar_event(event, ctx);
        });

        ctx.subscribe_to_model(&search_bar_state, |me, _handle, event, ctx| {
            me.handle_search_bar_event(event, ctx);
        });

        Self {
            search_bar,
            search_bar_state,
            mixer,
            handle: ctx.handle(),
            scroll_state: Default::default(),
            list_state: Default::default(),
            mode: PanelMode::Searching,
        }
    }

    pub fn setup(&mut self, ctx: &mut ViewContext<Self>) {
        self.mode = PanelMode::Searching;
        self.mixer.update(ctx, |mixer, ctx| {
            mixer.reset(ctx);
            if let Ok(data_source) = QuickCredentialDataSource::new() {
                mixer.add_sync_source(data_source, HashSet::new());
            }
            ctx.notify();
        });

        self.search_bar.update(ctx, |search_bar, ctx| {
            search_bar.reset(None, None, SearchResultOrdering::TopDown, ctx);
            ctx.notify();
        });

        ctx.emit(QuickCredentialPanelEvent::Open);
    }

    pub fn close(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.emit(QuickCredentialPanelEvent::Close);
    }

    fn handle_search_bar_event(
        &mut self,
        event: &SearchBarEvent<QuickCredentialSearchItemAction>,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            SearchBarEvent::Close => self.close(ctx),
            SearchBarEvent::BufferCleared { .. } => {}
            SearchBarEvent::ResultAccepted { action, .. } => {
                self.handle_result_selected(action.clone(), ctx);
            }
            SearchBarEvent::ResultSelected { index } => {
                self.list_state.scroll_to(*index);
                ctx.notify();
            }
            SearchBarEvent::QueryFilterChanged { .. } => {}
            SearchBarEvent::SelectionUpdateInZeroState { .. } => {}
            SearchBarEvent::EnterInZeroState { .. } => {}
        }
    }

    fn handle_result_selected(
        &mut self,
        result_action: QuickCredentialSearchItemAction,
        ctx: &mut ViewContext<Self>,
    ) {
        match result_action {
            QuickCredentialSearchItemAction::SelectCredential(credential) => {
                self.mode = PanelMode::SendModeSelection { credential };
                ctx.notify();
            }
        }
    }

    fn handle_send_password_only(
        &mut self,
        credential: warp_quick_credential::QuickCredential,
        ctx: &mut ViewContext<Self>,
    ) {
        let mut credential = credential;
        credential.send_mode = warp_quick_credential::SendMode::PasswordOnly;
        ctx.emit(QuickCredentialPanelEvent::ItemSelected { credential });
        self.close(ctx);
    }

    fn handle_send_username_then_password(
        &mut self,
        credential: warp_quick_credential::QuickCredential,
        ctx: &mut ViewContext<Self>,
    ) {
        let mut credential = credential;
        credential.send_mode = warp_quick_credential::SendMode::UsernameThenPassword;
        ctx.emit(QuickCredentialPanelEvent::ItemSelected { credential });
        self.close(ctx);
    }

    fn render_no_results(&self, appearance: &Appearance) -> Box<dyn Element> {
        let text = appearance
            .ui_builder()
            .span(crate::t!("common-no-results-found"))
            .with_style(UiComponentStyles {
                font_size: Some(appearance.monospace_font_size()),
                font_family_id: Some(appearance.ui_font_family()),
                font_color: Some(appearance.theme().nonactive_ui_text_color().into()),
                ..Default::default()
            })
            .build()
            .finish();

        let vertical_padding = styles::line_height_sensitive_vertical_padding(appearance);
        Container::new(
            ConstrainedBox::new(Align::new(text).finish())
                .with_height(
                    appearance.monospace_font_size() + vertical_padding - styles::TOP_PADDING,
                )
                .finish(),
        )
        .with_margin_bottom(styles::TOP_PADDING)
        .finish()
    }

    fn render_present_results(
        &self,
        appearance: &Appearance,
        selected_index: usize,
        query_result_renderers: &[QueryResultRenderer<QuickCredentialSearchItemAction>],
    ) -> Box<dyn Element> {
        let view_handle = self.handle.clone();
        let build_items = move |range: Range<usize>, app: &AppContext| {
            let secrets_view = view_handle
                .upgrade(app)
                .expect("View handle should upgradeable.")
                .as_ref(app);
            let query_result_renderers = secrets_view
                .search_bar_state
                .as_ref(app)
                .query_result_renderers();
            match query_result_renderers {
                Some(query_result_renderers) => {
                    let query_result_iter = if range.end == 1 {
                        query_result_renderers[range.start..].iter()
                    } else {
                        query_result_renderers[range.start..range.end].iter()
                    };
                    query_result_iter
                        .enumerate()
                        .map(|(result_index, result_renderer)| {
                            let result_index = result_index + range.start;
                            SavePosition::new(
                                result_renderer.render(
                                    result_index,
                                    result_index == selected_index,
                                    app,
                                ),
                                result_renderer.position_id.as_str(),
                            )
                            .finish()
                        })
                        .collect_vec()
                        .into_iter()
                }
                None => Vec::new().into_iter(),
            }
        };

        let scrollable_results = Scrollable::vertical(
            self.scroll_state.clone(),
            UniformList::new(
                self.list_state.clone(),
                query_result_renderers.len(),
                build_items,
            )
            .finish_scrollable(),
            styles::SCROLLBAR_WIDTH,
            appearance.theme().nonactive_ui_detail().into(),
            appearance.theme().active_ui_detail().into(),
            Fill::None,
        )
        .finish();

        ConstrainedBox::new(scrollable_results)
            .with_max_height(styles::VIEW_HEIGHT)
            .finish()
    }

    fn render_results(&self, appearance: &Appearance, app: &AppContext) -> Box<dyn Element> {
        let query_result_renderers = self.search_bar_state.as_ref(app).query_result_renderers();
        let selected_index = self.search_bar_state.as_ref(app).selected_index();
        match (query_result_renderers, selected_index) {
            (Some(query_result_renderers), _) if query_result_renderers.is_empty() => {
                self.render_no_results(appearance)
            }
            (Some(query_result_renderers), Some(selected_index)) => {
                self.render_present_results(appearance, selected_index, query_result_renderers)
            }
            _ => Empty::new().finish(),
        }
    }

    fn render_input_area(&self, appearance: &Appearance) -> Box<dyn Element> {
        Container::new(ChildView::new(&self.search_bar).finish())
            .with_background(styles::search_bar_overlay(appearance))
            .with_corner_radius(CornerRadius::with_top(Radius::Pixels(
                styles::CORNER_RADIUS,
            )))
            .with_border(styles::panel_border(appearance).with_sides(true, true, false, true))
            .with_uniform_padding(12.)
            .finish()
    }

    fn render_clickable_row(
        label: &str,
        action: QuickCredentialPanelAction,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let mouse_state = MouseStateHandle::default();
        let action_clone = action.clone();
        let label_owned = label.to_owned();
        let ui_font_family = appearance.ui_font_family();
        let font_size = appearance.ui_font_subheading();
        let theme = appearance.theme();
        EventHandler::new(
            Hoverable::new(mouse_state, move |mouse_state| {
                let is_hovered = mouse_state.is_hovered();
                let text_color = if is_hovered {
                    theme.active_ui_text_color()
                } else {
                    theme.nonactive_ui_text_color()
                };
                let bg_color = if is_hovered {
                    let fill: warpui::elements::Fill =
                        theme.accent().with_opacity(20).into();
                    fill.start_color()
                } else {
                    pathfinder_color::ColorU::transparent_black()
                };
                Container::new(
                    appearance
                        .ui_builder()
                        .span(label_owned.clone())
                        .with_style(UiComponentStyles {
                            font_size: Some(font_size),
                            font_family_id: Some(ui_font_family),
                            font_color: Some(text_color.into()),
                            ..Default::default()
                        })
                        .build()
                        .finish(),
                )
                .with_uniform_padding(12.)
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(
                    styles::CORNER_RADIUS,
                )))
                .with_background_color(bg_color)
                .finish()
            })
            .with_cursor(Cursor::PointingHand)
            .finish(),
        )
        .on_left_mouse_down(|_, _, _| DispatchEventResult::StopPropagation)
        .on_left_mouse_up(move |event_ctx: &mut EventContext, _, _| {
            event_ctx.dispatch_typed_action(action_clone.clone());
            DispatchEventResult::StopPropagation
        })
        .finish()
    }

    fn render_send_mode_selection(
        &self,
        credential: &warp_quick_credential::QuickCredential,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let credential_pw = credential.clone();
        let credential_both = credential.clone();

        let send_password = Self::render_clickable_row(
            "Send Password Only",
            QuickCredentialPanelAction::SendPasswordOnly(credential_pw),
            appearance,
        );

        let send_both = Self::render_clickable_row(
            "Send Username + Password",
            QuickCredentialPanelAction::SendUsernameThenPassword(credential_both),
            appearance,
        );

        let panel = Flex::column()
            .with_child(send_password)
            .with_child(send_both)
            .finish();

        Container::new(
            ConstrainedBox::new(panel)
                .with_max_width(styles::VIEW_WIDTH)
                .finish(),
        )
        .with_background(styles::panel_background_fill(appearance))
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(styles::CORNER_RADIUS)))
        .with_border(styles::panel_border(appearance))
        .with_drop_shadow(styles::panel_drop_shadow())
        .with_uniform_padding(20.)
        .finish()
    }
}

impl Entity for QuickCredentialPanel {
    type Event = QuickCredentialPanelEvent;
}

impl View for QuickCredentialPanel {
    fn ui_name() -> &'static str {
        "QuickCredentialPanel"
    }

    fn on_focus(&mut self, focus_ctx: &FocusContext, ctx: &mut ViewContext<Self>) {
        if focus_ctx.is_self_focused() {
            match self.mode {
                PanelMode::Searching => {
                    ctx.focus(&self.search_bar);
                }
                PanelMode::SendModeSelection { .. } => {}
            }
        }
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        match &self.mode {
            PanelMode::Searching => {
                let appearance = Appearance::as_ref(app);

                let panel_children = vec![
                    self.render_input_area(appearance),
                    Shrinkable::new(
                        1.,
                        Container::new(self.render_results(appearance, app))
                            .with_padding_top(styles::TOP_PADDING)
                            .with_background(styles::panel_background_fill(appearance))
                            .with_corner_radius(CornerRadius::with_bottom(Radius::Pixels(
                                styles::CORNER_RADIUS,
                            )))
                            .with_border(
                                styles::panel_border(appearance).with_sides(false, true, true, true),
                            )
                            .finish(),
                    )
                    .finish(),
                ];

                let panel_contents =
                    ConstrainedBox::new(Flex::column().with_children(panel_children).finish())
                        .with_max_width(styles::VIEW_WIDTH)
                        .finish();

                Dismiss::new(
                    Container::new(panel_contents)
                        .with_drop_shadow(styles::panel_drop_shadow())
                        .finish(),
                )
                .on_dismiss(|ctx, _app| {
                    ctx.dispatch_typed_action(QuickCredentialPanelAction::Close);
                })
                .finish()
            }
            PanelMode::SendModeSelection { credential } => {
                let appearance = Appearance::as_ref(app);
                let selection_view = self.render_send_mode_selection(credential, appearance);
                Dismiss::new(
                    Container::new(
                        ConstrainedBox::new(Align::new(selection_view).finish()).finish(),
                    )
                    .finish(),
                )
                .on_dismiss(|ctx, _app| {
                    ctx.dispatch_typed_action(QuickCredentialPanelAction::Close);
                })
                .finish()
            }
        }
    }
}

impl TypedActionView for QuickCredentialPanel {
    type Action = QuickCredentialPanelAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            QuickCredentialPanelAction::Close => self.close(ctx),
            QuickCredentialPanelAction::ResultClicked { result_action, .. } => {
                self.handle_result_selected(*result_action.clone(), ctx)
            }
            QuickCredentialPanelAction::SendPasswordOnly(credential) => {
                self.handle_send_password_only(credential.clone(), ctx);
            }
            QuickCredentialPanelAction::SendUsernameThenPassword(credential) => {
                self.handle_send_username_then_password(credential.clone(), ctx);
            }
        }
    }
}
