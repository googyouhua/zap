use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;

use markdown_parser::{FormattedText, FormattedTextFragment, FormattedTextLine};
use regex::Regex;
use settings::{Setting, ToggleableSetting};
use strum::IntoEnumIterator;
use warp_core::features::FeatureFlag;
use warpui::elements::{FormattedTextElement, HighlightedHyperlink};
use warpui::keymap::ContextPredicate;
use warpui::{
    elements::{
        Align, Container, CornerRadius, CrossAxisAlignment, Fill, Flex, Hoverable,
        MouseStateHandle, ParentElement, Radius, Shrinkable,
    },
    presenter::ChildView,
    ui_components::{
        components::{Coords, UiComponent, UiComponentStyles},
        switch::SwitchStateHandle,
    },
    Action, AppContext, Element, Entity, ModelHandle, SingletonEntity, TypedActionView, View,
    ViewContext, ViewHandle,
};

use crate::terminal::warpify::settings::{
    EnableSshWarpification, SshExtensionInstallMode, SshExtensionInstallModeSetting,
    UseSshTmuxWrapper, WarpifySettingsChangedEvent,
};
use crate::ui_components::blended_colors;

use crate::{
    appearance::Appearance,
    editor::{Event as EditorEvent, EditorView},
    report_if_error, send_telemetry_from_ctx,
    server::telemetry::TelemetryEvent,
    terminal::warpify::settings::WarpifySettings,
    view_components::{SubmittableTextInput, SubmittableTextInputEvent},
};

use super::settings_page::{
    render_body_item, render_dropdown_item, render_page_title, AdditionalInfo, Category,
    LocalOnlyIconState, MatchData, PageType, SettingsPageEvent, SettingsWidget, ToggleState,
    HEADER_PADDING,
};
use super::SettingsSection;
use warp_core::ui::theme::color::internal_colors;
use warpui::color::ColorU;
use super::{
    flags,
    settings_page::{
        add_setting, render_alternating_color_list, SettingsPageMeta, SettingsPageViewHandle,
    },
    SettingsAction, ToggleSettingActionPair,
};
use crate::view_components::dropdown::{Dropdown, DropdownItem};

pub fn init_actions_from_parent_view<T: Action + Clone>(
    app: &mut AppContext,
    context: &ContextPredicate,
    builder: fn(SettingsAction) -> T,
) {
    // Add all of the toggle settings from the Warpify Page that you want to show up on the Command Palette here.
    let mut toggle_binding_pairs = vec![];

    if FeatureFlag::SSHTmuxWrapper.is_enabled() {
        toggle_binding_pairs.push(ToggleSettingActionPair::new(
            &crate::t!("settings-warpify-ssh-tmux-toggle-binding-label"),
            builder(SettingsAction::WarpifyPageToggle(
                WarpifyPageAction::ToggleTmuxWarpification,
            )),
            context,
            flags::SSH_TMUX_WRAPPER_CONTEXT_FLAG,
        ));
    }

    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(toggle_binding_pairs, app);
}

const ITEM_VERTICAL_SPACING: f32 = 24.;
/// There's a built-in 10px margin below the text input.
const BUILT_IN_TEXT_INPUT_MARGIN: f32 = 10.;
const SPACE_AFTER_TEXT_INPUT: f32 = ITEM_VERTICAL_SPACING - BUILT_IN_TEXT_INPUT_MARGIN;

/// This page lets users configure when they get asked to warpify a session. Some shell commands
/// are recognized by default. Users can add new shell commands, or prevent the default ones from
/// asking. Users can also enable the SSH wrapper, and add hosts to a denylist.
/// This page is essentially the View for the SubshellSettings model, as well as the SshSettings
/// related to warpification.
pub struct WarpifyPageView {
    page: PageType<Self>,
    /// This needs to mirror the length of SubshellSettings::added_remove_button_states.
    remove_added_command_button_states: Vec<MouseStateHandle>,
    add_added_commands_editor: ViewHandle<SubmittableTextInput>,
    /// This needs to mirror the length of SubshellSettings::denylisted_remove_button_states.
    remove_denylisted_command_button_states: Vec<MouseStateHandle>,
    add_denylisted_commands_editor: ViewHandle<SubmittableTextInput>,

    remove_denylisted_ssh_button_states: Vec<MouseStateHandle>,
    edit_denylisted_ssh_button_states: Vec<MouseStateHandle>,
    add_denylisted_ssh_editor: ViewHandle<SubmittableTextInput>,

    pending_edit_ssh_host_index: Option<usize>,

    ssh_extension_install_mode_dropdown: ViewHandle<Dropdown<WarpifyPageAction>>,
}

impl WarpifyPageView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let warpify_settings_handle = WarpifySettings::handle(ctx);

        ctx.observe(&warpify_settings_handle, Self::update_button_states);
        ctx.subscribe_to_model(&warpify_settings_handle, move |me, model, event, ctx| {
            me.update_button_states(model, ctx);
            if matches!(
                event,
                WarpifySettingsChangedEvent::SshExtensionInstallModeSetting { .. }
            ) {
                me.update_dropdown(ctx);
            }
            ctx.notify();
        });

        // Added commands can be specified by regex, while denied commands are strictly exact
        // match.
        let add_added_commands_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|regex| Regex::new(regex).is_ok());
            input.set_placeholder_text(crate::t!("settings-warpify-command-placeholder"), ctx);
            input
        });

        ctx.subscribe_to_view(
            &add_added_commands_editor,
            Self::handle_added_command_editor_event,
        );

        let add_denylisted_commands_editor = ctx.add_typed_action_view(|ctx| {
            let mut input = SubmittableTextInput::new(ctx);
            input.set_placeholder_text(crate::t!("settings-warpify-command-placeholder"), ctx);
            input
        });

        ctx.subscribe_to_view(
            &add_denylisted_commands_editor,
            Self::handle_denylisted_command_editor_event,
        );

        let add_denylisted_ssh_editor = ctx.add_typed_action_view(|ctx| {
            let mut input = SubmittableTextInput::new(ctx);
            input.set_placeholder_text(crate::t!("settings-warpify-host-placeholder"), ctx);
            input
        });

        ctx.subscribe_to_view(
            &add_denylisted_ssh_editor,
            Self::handle_denylisted_ssh_editor_event,
        );

        // Subscribe to the inner EditorView's Blurred to discard edit when clicking other inputs.
        let deny_ssh_editor_handle = add_denylisted_ssh_editor.read(ctx, |editor, _| {
            editor.editor().clone()
        });
        ctx.subscribe_to_view(
            &deny_ssh_editor_handle,
            Self::handle_denylisted_ssh_blur_event,
        );

        let ssh_extension_install_mode_dropdown =
            Self::create_ssh_extension_install_mode_dropdown(ctx);

        let mut instance = Self {
            page: Self::build_page(ctx),
            remove_added_command_button_states: Default::default(),
            add_added_commands_editor,
            remove_denylisted_command_button_states: Default::default(),
            add_denylisted_commands_editor,
            remove_denylisted_ssh_button_states: Default::default(),
            edit_denylisted_ssh_button_states: Default::default(),
            add_denylisted_ssh_editor,
            pending_edit_ssh_host_index: None,
            ssh_extension_install_mode_dropdown,
        };

        instance.update_button_states(warpify_settings_handle, ctx);
        instance
    }

    fn build_page(ctx: &mut ViewContext<Self>) -> PageType<Self> {
        let mut categories = vec![
            Category::new("", vec![Box::new(TitleWidget::default())]),
            Category::new(
                Box::leak(crate::t!("settings-warpify-section-subshells").into_boxed_str()),
                vec![Box::new(SubshellsWidget::default())],
            )
            .with_subtitle(Box::leak(
                crate::t!("settings-warpify-section-subshells-subtitle").into_boxed_str(),
            )),
        ];

        let warpify_settings = WarpifySettings::as_ref(ctx);
        if FeatureFlag::SSHTmuxWrapper.is_enabled()
            && warpify_settings
                .enable_ssh_warpification
                .is_supported_on_current_platform()
        {
            categories.push(
                Category::new(
                    Box::leak(crate::t!("settings-warpify-section-ssh").into_boxed_str()),
                    vec![Box::new(SSHWidget::default())],
                )
                .with_subtitle(Box::leak(
                    crate::t!("settings-warpify-section-ssh-subtitle").into_boxed_str(),
                )),
            );
        }
        PageType::new_categorized(categories, None)
    }

    /// This method ensures each command in the SubshellSettings has a matching button state for
    /// its delete button in the View.
    fn update_button_states(
        &mut self,
        warpify_settings_handle: ModelHandle<WarpifySettings>,
        ctx: &mut ViewContext<Self>,
    ) {
        let warpify_settings = warpify_settings_handle.as_ref(ctx);
        self.remove_denylisted_command_button_states = warpify_settings
            .subshell_command_denylist
            .iter()
            .map(|_| Default::default())
            .collect();
        self.remove_added_command_button_states = warpify_settings
            .added_subshell_commands
            .iter()
            .map(|_| Default::default())
            .collect();
        self.remove_denylisted_ssh_button_states = warpify_settings
            .ssh_hosts_denylist
            .iter()
            .map(|_| Default::default())
            .collect();
        self.edit_denylisted_ssh_button_states = warpify_settings
            .ssh_hosts_denylist
            .iter()
            .map(|_| Default::default())
            .collect();
        ctx.notify();
    }

    /// Syncs the install-mode dropdown selection with the current
    /// `WarpifySettings::ssh_extension_install_mode` value (e.g. after it
    /// was changed from the SSH remote server choice view).
    fn update_dropdown(&mut self, ctx: &mut ViewContext<Self>) {
        let current_mode = *WarpifySettings::as_ref(ctx)
            .ssh_extension_install_mode
            .value();
        self.ssh_extension_install_mode_dropdown
            .update(ctx, |dropdown, ctx| {
                dropdown.set_selected_by_action(
                    WarpifyPageAction::SetSshExtensionInstallMode(current_mode),
                    ctx,
                );
            });
    }

    fn handle_added_command_editor_event(
        &mut self,
        _handle: ViewHandle<SubmittableTextInput>,
        event: &SubmittableTextInputEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            SubmittableTextInputEvent::Submit(new_command) => {
                WarpifySettings::handle(ctx).update(ctx, |warpify_settings, ctx| {
                    warpify_settings.add_subshell_command(new_command, ctx);
                });

                send_telemetry_from_ctx!(TelemetryEvent::AddAddedSubshellCommand, ctx);
            }
            SubmittableTextInputEvent::Escape => ctx.emit(SettingsPageEvent::FocusModal),
        }
    }

    fn handle_denylisted_command_editor_event(
        &mut self,
        _handle: ViewHandle<SubmittableTextInput>,
        event: &SubmittableTextInputEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            SubmittableTextInputEvent::Submit(new_command) => {
                WarpifySettings::handle(ctx).update(ctx, |warpify_settings, ctx| {
                    warpify_settings.denylist_subshell_command(new_command, ctx);
                });

                send_telemetry_from_ctx!(TelemetryEvent::AddDenylistedSubshellCommand, ctx);
            }
            SubmittableTextInputEvent::Escape => ctx.emit(SettingsPageEvent::FocusModal),
        }
    }

    fn handle_denylisted_ssh_blur_event(
        &mut self,
        _handle: ViewHandle<EditorView>,
        event: &EditorEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        if matches!(event, EditorEvent::Blurred) {
            log::info!("[DENYLIST] Blurred, pending={:?}", self.pending_edit_ssh_host_index);
            self.discard_denylist_edit(ctx);
        }
    }

    fn handle_denylisted_ssh_editor_event(
        &mut self,
        _handle: ViewHandle<SubmittableTextInput>,
        event: &SubmittableTextInputEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            SubmittableTextInputEvent::Submit(new_command) => {
                let edit_index = self.pending_edit_ssh_host_index.take();
                let host = new_command.trim().to_string();
                log::info!("[DENYLIST] Submit: host={:?}, edit_idx={:?}", host, edit_index);
                if host.is_empty() {
                    return;
                }
                WarpifySettings::handle(ctx).update(ctx, |warpify_settings, ctx| {
                    if let Some(idx) = edit_index {
                        let mut new_list = warpify_settings.ssh_hosts_denylist.to_vec();
                        if idx < new_list.len() {
                            new_list[idx] = host;
                        }
                        warpify_settings.ssh_hosts_denylist
                            .set_value(new_list, ctx)
                            .expect("ssh_hosts_denylist failed to serialize");
                    } else {
                        warpify_settings.denylist_ssh_host(&host, ctx);
                    }
                });

                send_telemetry_from_ctx!(TelemetryEvent::AddDenylistedSshTmuxWrapperHost, ctx);
                ctx.notify();
            }
            SubmittableTextInputEvent::Escape => {
                log::info!("[DENYLIST] Escape, pending={:?}", self.pending_edit_ssh_host_index);
                let edit_idx = self.pending_edit_ssh_host_index.take();
                if let Some(original) = edit_idx.and_then(|idx| {
                    WarpifySettings::as_ref(ctx).ssh_hosts_denylist.get(idx).cloned()
                }) {
                    self.add_denylisted_ssh_editor.update(ctx, |editor, ctx| {
                        editor.editor().update(ctx, |e, ctx| {
                            e.system_reset_buffer_text(&original, ctx);
                        });
                    });
                }
                ctx.emit(SettingsPageEvent::FocusModal);
            }
        }
    }

    fn remove_denylisted_command(&self, index: usize, ctx: &mut ViewContext<Self>) {
        send_telemetry_from_ctx!(TelemetryEvent::RemoveDenylistedSubshellCommand, ctx);
        WarpifySettings::handle(ctx).update(ctx, |warpify, ctx| {
            warpify.remove_denylisted_subshell_command(index, ctx)
        });
    }

    fn remove_added_command(&self, index: usize, ctx: &mut ViewContext<Self>) {
        send_telemetry_from_ctx!(TelemetryEvent::RemoveAddedSubshellCommand, ctx);
        WarpifySettings::handle(ctx).update(ctx, |warpify, ctx| {
            warpify.remove_added_subshell_command(index, ctx)
        });
    }

    fn remove_denylisted_ssh_host(&self, index: usize, ctx: &mut ViewContext<Self>) {
        send_telemetry_from_ctx!(TelemetryEvent::RemoveDenylistedSshTmuxWrapperHost, ctx);
        WarpifySettings::handle(ctx).update(ctx, |warpify, ctx| {
            warpify.remove_denylisted_ssh_host(index, ctx)
        });
    }
}

impl Entity for WarpifyPageView {
    type Event = SettingsPageEvent;
}

fn build_sub_sub_title(title: String, appearance: &Appearance) -> Container {
    appearance
        .ui_builder()
        .span(title)
        .with_style(UiComponentStyles {
            font_size: Some(appearance.ui_font_body()),
            ..Default::default()
        })
        .build()
}

const SSH_EXTENSION_DROPDOWN_WIDTH: f32 = 250.;

impl WarpifyPageView {
    /// Discards any in-progress denylist edit: clears editor and exits edit mode.
    fn discard_denylist_edit(&mut self, ctx: &mut ViewContext<Self>) {
        self.pending_edit_ssh_host_index = None;
        self.add_denylisted_ssh_editor.update(ctx, |editor, ctx| {
            editor.editor().update(ctx, |e, ctx| {
                e.clear_buffer(ctx);
            });
        });
    }

    fn create_ssh_extension_install_mode_dropdown(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<Dropdown<WarpifyPageAction>> {
        let items: Vec<DropdownItem<WarpifyPageAction>> = SshExtensionInstallMode::iter()
            .map(|mode| {
                DropdownItem::new(
                    mode.display_name(),
                    WarpifyPageAction::SetSshExtensionInstallMode(mode),
                )
            })
            .collect();

        let current_mode = *WarpifySettings::as_ref(ctx)
            .ssh_extension_install_mode
            .value();
        let enable_ssh_warpification = *WarpifySettings::as_ref(ctx)
            .enable_ssh_warpification
            .value();

        ctx.add_typed_action_view(move |ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(SSH_EXTENSION_DROPDOWN_WIDTH);
            dropdown.set_menu_width(SSH_EXTENSION_DROPDOWN_WIDTH, ctx);
            dropdown.add_items(items, ctx);
            dropdown.set_selected_by_action(
                WarpifyPageAction::SetSshExtensionInstallMode(current_mode),
                ctx,
            );
            if !enable_ssh_warpification {
                dropdown.set_disabled(ctx);
            }
            dropdown
        })
    }

    /// Renders a title, a list of items that can be removed, and an input field to add new items.
    fn build_input_list<
        ListItem: Display,
        SettingsPageAction: Action + Clone,
        F: Fn(usize) -> SettingsPageAction,
        T: View,
    >(
        &self,
        title: String,
        patterns: &[ListItem],
        mouse_states: &[MouseStateHandle],
        create_action: F,
        handle: &ViewHandle<T>,
        appearance: &Appearance,
    ) -> Container {
        let mut column = Flex::column();
        let mut title = build_sub_sub_title(title, appearance);

        if !patterns.is_empty() {
            title = title.with_padding_bottom(BUILT_IN_TEXT_INPUT_MARGIN);
        }

        column.add_child(title.finish());

        render_alternating_color_list(
            &mut column,
            patterns,
            mouse_states,
            create_action,
            appearance,
        );

        Container::new(
            column
                .with_child(
                    Container::new(ChildView::new(handle).finish())
                        .with_margin_bottom(SPACE_AFTER_TEXT_INPUT)
                        .finish(),
                )
                .finish(),
        )
    }
}

impl View for WarpifyPageView {
    fn ui_name() -> &'static str {
        "WarpifyPageView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WarpifyPageAction {
    RemoveAddedCommand(usize),
    RemoveDenylistedCommand(usize),
    RemoveDenylistedSshHost(usize),
    EditDenylistedSshHost(usize),
    /// If disabled, auto-Warpification and the SSH Warpification prompt will be disabled.
    ToggleTmuxWarpification,
    ToggleSshWarpification,
    /// Set the SSH extension installation mode (always ask / always install / always skip).
    SetSshExtensionInstallMode(SshExtensionInstallMode),
    OpenUrl(String),
}

impl TypedActionView for WarpifyPageView {
    type Action = WarpifyPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        use WarpifyPageAction::*;
        // Any action other than starting an edit cancels the in-progress edit.
        if !matches!(action, EditDenylistedSshHost(_)) {
            if self.pending_edit_ssh_host_index.is_some() {
                log::info!("[DENYLIST] handle_action discard: {:?}", action);
            }
            self.discard_denylist_edit(ctx);
        }
        match action {
            RemoveDenylistedCommand(index) => self.remove_denylisted_command(*index, ctx),
            RemoveAddedCommand(index) => self.remove_added_command(*index, ctx),
            ToggleSshWarpification => {
                WarpifySettings::handle(ctx).update(ctx, |ssh_settings, ctx| {
                    report_if_error!(ssh_settings
                        .enable_ssh_warpification
                        .toggle_and_save_value(ctx));
                    send_telemetry_from_ctx!(
                        TelemetryEvent::ToggleSshWarpification {
                            enabled: *ssh_settings.enable_ssh_warpification.value(),
                        },
                        ctx
                    );
                });
                let enabled = *WarpifySettings::as_ref(ctx)
                    .enable_ssh_warpification
                    .value();
                self.ssh_extension_install_mode_dropdown
                    .update(ctx, |dropdown, ctx| {
                        if enabled {
                            dropdown.set_enabled(ctx);
                        } else {
                            dropdown.set_disabled(ctx);
                        }
                    });
            }
            ToggleTmuxWarpification => {
                WarpifySettings::handle(ctx).update(ctx, |ssh_settings, ctx| {
                    report_if_error!(ssh_settings.use_ssh_tmux_wrapper.toggle_and_save_value(ctx));
                    send_telemetry_from_ctx!(
                        TelemetryEvent::ToggleSshTmuxWrapper {
                            enabled: *ssh_settings.use_ssh_tmux_wrapper.value(),
                        },
                        ctx
                    );
                });
            }
            SetSshExtensionInstallMode(mode) => {
                WarpifySettings::handle(ctx).update(ctx, |warpify_settings, ctx| {
                    report_if_error!(warpify_settings
                        .ssh_extension_install_mode
                        .set_value(*mode, ctx));
                    send_telemetry_from_ctx!(
                        TelemetryEvent::SetSshExtensionInstallMode {
                            mode: mode.telemetry_name(),
                        },
                        ctx
                    );
                });
            }
            WarpifyPageAction::RemoveDenylistedSshHost(index) => {
                self.remove_denylisted_ssh_host(*index, ctx);
            }
            WarpifyPageAction::EditDenylistedSshHost(index) => {
                let host = WarpifySettings::as_ref(ctx)
                    .ssh_hosts_denylist
                    .get(*index)
                    .cloned();
                if let Some(host) = host {
                    self.add_denylisted_ssh_editor.update(ctx, |editor, ctx| {
                        editor.editor().update(ctx, |e, ctx| {
                            e.system_reset_buffer_text(&host, ctx);
                        });
                    });
                    ctx.focus(&self.add_denylisted_ssh_editor);
                    self.pending_edit_ssh_host_index = Some(*index);
                }
            }
            OpenUrl(url) => {
                ctx.open_url(url.as_str());
            }
        }
    }
}

impl SettingsPageMeta for WarpifyPageView {
    fn section() -> SettingsSection {
        SettingsSection::Warpify
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<WarpifyPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<WarpifyPageView>) -> Self {
        SettingsPageViewHandle::Warpify(view_handle)
    }
}

#[derive(Default)]
struct TitleWidget {
    learn_more_highlight_index: HighlightedHyperlink,
}

impl TitleWidget {
    fn render_top_of_page(&self, appearance: &Appearance, _app: &AppContext) -> Box<dyn Element> {
        let warpify_description = vec![
            FormattedTextFragment::plain_text(crate::t!("settings-warpify-description-prefix")),
            FormattedTextFragment::hyperlink(
                crate::t!("settings-warpify-learn-more"),
                "",
            ),
        ];

        let warpify_description = FormattedTextElement::new(
            FormattedText::new([FormattedTextLine::Line(warpify_description)]),
            appearance.ui_font_body(),
            appearance.ui_font_family(),
            appearance.ui_font_family(),
            blended_colors::text_sub(appearance.theme(), appearance.theme().surface_1()),
            self.learn_more_highlight_index.clone(),
        )
        .with_heading_to_font_size_multipliers(appearance.heading_font_size_multipliers().clone())
        .with_hyperlink_font_color(appearance.theme().accent().into_solid())
        .register_default_click_handlers(|url, _, ctx| {
            ctx.open_url(&url.url);
        })
        .finish();

        Flex::column()
            .with_child(render_page_title(
                &crate::t!("settings-warpify-page-title"),
                appearance,
            ))
            .with_child(warpify_description)
            .finish()
    }
}

impl SettingsWidget for TitleWidget {
    type View = WarpifyPageView;

    fn search_terms(&self) -> &str {
        "ssh subshell warpify session"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        Container::new(self.render_top_of_page(appearance, app))
            .with_margin_bottom(ITEM_VERTICAL_SPACING)
            .finish()
    }
}

#[derive(Default)]
struct SubshellsWidget {}

impl SubshellsWidget {
    fn render_subshells_section(
        &self,
        view: &WarpifyPageView,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let mut column = Flex::column();

        let warpify_settings = WarpifySettings::as_ref(app);

        column.add_child(
            view.build_input_list(
                crate::t!("settings-warpify-added-commands"),
                &warpify_settings.added_subshell_commands,
                &view.remove_added_command_button_states,
                WarpifyPageAction::RemoveAddedCommand,
                &view.add_added_commands_editor,
                appearance,
            )
            .finish(),
        );

        column.add_child(
            view.build_input_list(
                crate::t!("settings-warpify-denylisted-commands"),
                &warpify_settings.subshell_command_denylist,
                &view.remove_denylisted_command_button_states,
                WarpifyPageAction::RemoveDenylistedCommand,
                &view.add_denylisted_commands_editor,
                appearance,
            )
            .with_margin_bottom(-BUILT_IN_TEXT_INPUT_MARGIN)
            .finish(),
        );

        column.finish()
    }
}

impl SettingsWidget for SubshellsWidget {
    type View = WarpifyPageView;

    fn search_terms(&self) -> &str {
        "warpify subshell"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        Container::new(self.render_subshells_section(view, appearance, app))
            .with_margin_bottom(ITEM_VERTICAL_SPACING)
            .finish()
    }
}

#[derive(Default)]
struct SSHWidget {
    tmux_warpification_switch_state: SwitchStateHandle,
    enable_ssh_warpification_switch_state: SwitchStateHandle,
    additional_info_mouse_state: MouseStateHandle,
    local_only_icon_tooltip_states: RefCell<HashMap<String, MouseStateHandle>>,
}

impl SettingsWidget for SSHWidget {
    type View = WarpifyPageView;

    fn search_terms(&self) -> &str {
        "warpify ssh"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let mut column = Flex::column();
        let ui_builder = appearance.ui_builder();
        let description_text_color = appearance
            .theme()
            .sub_text_color(appearance.theme().surface_2());

        let enable_ssh_warpification = *WarpifySettings::as_ref(app)
            .enable_ssh_warpification
            .value();

        let should_prompt_ssh_tmux_wrapper =
            *WarpifySettings::as_ref(app).use_ssh_tmux_wrapper.value();

        add_setting(
            &mut column,
            &WarpifySettings::as_ref(app).enable_ssh_warpification,
            move || {
                render_body_item::<WarpifyPageAction>(
                    crate::t!("settings-warpify-enable-ssh"),
                    None,
                    LocalOnlyIconState::for_setting(
                        EnableSshWarpification::storage_key(),
                        EnableSshWarpification::sync_to_cloud(),
                        &mut self.local_only_icon_tooltip_states.borrow_mut(),
                        app,
                    ),
                    ToggleState::Enabled,
                    appearance,
                    ui_builder
                        .switch(self.enable_ssh_warpification_switch_state.clone())
                        .check(enable_ssh_warpification)
                        .build()
                        .on_click(move |ctx, _, _| {
                            ctx.dispatch_typed_action(WarpifyPageAction::ToggleSshWarpification);
                        })
                        .finish(),
                    None,
                )
            },
        );

        if FeatureFlag::SshRemoteServer.is_enabled() {
            let label_color_override = if !enable_ssh_warpification {
                Some(appearance.theme().disabled_ui_text_color())
            } else {
                None
            };
            add_setting(
                &mut column,
                &WarpifySettings::as_ref(app).ssh_extension_install_mode,
                move || {
                    let install_ssh_label = crate::t!("settings-warpify-install-ssh-extension");
                    let install_ssh_desc =
                        crate::t!("settings-warpify-install-ssh-extension-description");
                    Container::new(render_dropdown_item(
                        appearance,
                        &install_ssh_label,
                        Some(&install_ssh_desc),
                        None,
                        LocalOnlyIconState::for_setting(
                            SshExtensionInstallModeSetting::storage_key(),
                            SshExtensionInstallModeSetting::sync_to_cloud(),
                            &mut self.local_only_icon_tooltip_states.borrow_mut(),
                            app,
                        ),
                        label_color_override,
                        &view.ssh_extension_install_mode_dropdown,
                    ))
                    .with_padding_bottom(HEADER_PADDING)
                    .finish()
                },
            );
        }

        add_setting(
            &mut column,
            &WarpifySettings::as_ref(app).use_ssh_tmux_wrapper,
            move || {
                let mut column = Flex::column();

                column.add_child(render_body_item::<WarpifyPageAction>(
                    crate::t!("settings-warpify-use-tmux"),
                    Some(AdditionalInfo {
                        mouse_state: self.additional_info_mouse_state.clone(),
                        on_click_action: Some(WarpifyPageAction::OpenUrl(
                            "".into(),
                        )),
                        secondary_text: None,
                        tooltip_override_text: None,
                    }),
                    LocalOnlyIconState::for_setting(
                        UseSshTmuxWrapper::storage_key(),
                        UseSshTmuxWrapper::sync_to_cloud(),
                        &mut self.local_only_icon_tooltip_states.borrow_mut(),
                        app,
                    ),
                    enable_ssh_warpification.into(),
                    appearance,
                    ui_builder
                        .switch(self.tmux_warpification_switch_state.clone())
                        .check(should_prompt_ssh_tmux_wrapper)
                        .with_disabled(!enable_ssh_warpification)
                        .build()
                        .on_click(move |ctx, _, _| {
                            if !enable_ssh_warpification {
                                return;
                            }

                            ctx.dispatch_typed_action(WarpifyPageAction::ToggleTmuxWarpification);
                        })
                        .finish(),
                    None,
                ));

                column.add_child(
                    ui_builder
                        .paragraph(crate::t!("settings-warpify-tmux-description"))
                        .with_style(UiComponentStyles {
                            font_color: Some(description_text_color.into_solid()),
                            margin: Some(
                                Coords::default()
                                    .top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                                    .bottom(styles::DESCRIPTION_LINE_MARGIN_BOTTOM),
                            ),
                            ..Default::default()
                        })
                        .build()
                        .finish(),
                );

                column.finish()
            },
        );

        if enable_ssh_warpification {
            let mut list_column = Flex::column();
            let mut title = build_sub_sub_title(
                crate::t!("settings-warpify-denylisted-hosts"),
                appearance,
            );
            if !WarpifySettings::as_ref(app).ssh_hosts_denylist.is_empty() {
                title = title.with_padding_bottom(BUILT_IN_TEXT_INPUT_MARGIN);
            }
            list_column.add_child(title.finish());

            let edit_index = view.pending_edit_ssh_host_index;
            for (i, host) in WarpifySettings::as_ref(app).ssh_hosts_denylist.iter().enumerate() {
                if edit_index == Some(i) {
                    list_column.add_child(
                        Container::new(ChildView::new(&view.add_denylisted_ssh_editor).finish())
                            .finish(),
                    );
                } else {
                    let background: Fill = if i % 2 == 0 {
                        Fill::Solid(internal_colors::fg_overlay_1(appearance.theme()).into())
                    } else {
                        Fill::Solid(ColorU::transparent_black())
                    };
                    list_column.add_child(render_ssh_denylist_item(
                        appearance,
                        background,
                        host,
                        view.remove_denylisted_ssh_button_states[i].clone(),
                        view.edit_denylisted_ssh_button_states[i].clone(),
                        i,
                    ));
                }
            }

            if edit_index.is_none() {
                list_column.add_child(
                    Container::new(ChildView::new(&view.add_denylisted_ssh_editor).finish())
                        .with_margin_bottom(SPACE_AFTER_TEXT_INPUT)
                        .finish(),
                );
            }

            column.add_child(Container::new(list_column.finish()).finish());
        }

        column.finish()
    }
}

const SSH_LIST_CLOSE_BUTTON_DIAMETER: f32 = 20.0;
const SSH_LIST_ITEM_PADDING: f32 = 8.0;
const SSH_LIST_ITEM_PADDING_BOTTOM: f32 = 10.0;

fn render_ssh_denylist_item(
    appearance: &Appearance,
    background: impl Into<Fill>,
    host: &str,
    remove_mouse_state: MouseStateHandle,
    edit_mouse_state: MouseStateHandle,
    index: usize,
) -> Box<dyn Element> {
    let background = background.into();
    let font_color = appearance.theme().foreground();

    let remove_button = appearance
        .ui_builder()
        .close_button(SSH_LIST_CLOSE_BUTTON_DIAMETER, remove_mouse_state)
        .build()
        .on_click(move |ctx, _, _| ctx.dispatch_typed_action(WarpifyPageAction::RemoveDenylistedSshHost(index)))
        .finish();

    let label = Hoverable::new(edit_mouse_state, |_| {
        appearance
            .ui_builder()
            .wrappable_text(host.to_string(), true)
            .with_style(UiComponentStyles {
                font_color: Some(font_color.into_solid()),
                font_family_id: Some(appearance.monospace_font_family()),
                font_size: Some(appearance.ui_font_size()),
                ..Default::default()
            })
            .build()
            .finish()
    })
    .on_click(move |ctx, _, _| ctx.dispatch_typed_action(WarpifyPageAction::EditDenylistedSshHost(index)))
    .finish();

    Container::new(
        Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_children([
                Shrinkable::new(1., Align::new(label).left().finish()).finish(),
                Container::new(remove_button)
                    .with_margin_left(SSH_LIST_ITEM_PADDING)
                    .finish(),
            ])
            .finish(),
    )
    .with_background(background)
    .with_uniform_padding(SSH_LIST_ITEM_PADDING)
    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
    .with_padding_bottom(SSH_LIST_ITEM_PADDING_BOTTOM)
    .finish()
}

mod styles {
    // Apply a negative margin to the description text so it appears closer to the main
    // settings option text.
    pub const DESCRIPTION_NEGATIVE_MARGIN_OFFSET: f32 = -8.;

    /// The space after a description.
    pub const DESCRIPTION_LINE_MARGIN_BOTTOM: f32 = 18.;

    /// Because we hide the SSH settings if the SSH wrapper is disabled, we need to add a margin
    /// to the bottom to make it clear that toggling this item will reveal more settings,
    /// even at smaller window sizes. We picked an offset that cuts off the first item
    /// to imply the user should scroll to see more.
    pub const MINIMUM_SCROLL_OFFSET_AFTER_SSH: f32 = 40.;
}
