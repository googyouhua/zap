use std::collections::HashMap;

use warpui::{
    elements::{
        ChildView, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, Element,
        Expanded, Flex, MainAxisSize, MouseStateHandle,
        ParentElement, Radius, Text,
    },
    fonts::{Properties, Weight},
    modals::{AlertDialogWithCallbacks, ModalButton},
    ui_components::{
        button::ButtonVariant,
        components::{Coords, UiComponent, UiComponentStyles},
    },
    AppContext, Entity, SingletonEntity, TypedActionView, UpdateModel, UpdateView, View,
    ViewContext, ViewHandle,
};

use crate::ssh_manager::{
    OneKeyCredentialsChangedEvent, OneKeyCredentialsChangedNotifier,
};
use warp_onekey::{OneKeyCredential, OneKeyKind, PromptTriggerRule, SendMode};

use super::settings_page::{
    render_page_title, render_sub_header_with_description,
    MatchData, PageType, SettingsPageEvent, SettingsPageMeta,
    SettingsWidget,
};
use super::SettingsSection;
use crate::appearance::Appearance;
use crate::editor::{EditorView, SingleLineEditorOptions, TextOptions};
use crate::report_if_error;

const FORM_HALF_GAP: f32 = 8.;
const FORM_ROW_GAP: f32 = 16.;
const CHIP_GAP: f32 = 6.;

#[derive(Debug, Clone)]
pub enum OneKeyPageAction {
    ShowAddForm,
    ShowEditForm(String),
    CancelForm,
    SaveForm,
    ShowDeleteConfirmation(String),
    SetLabel(String),
    SetUsername(String),
    SetPassword(String),
    SetNotes(String),
    SetKind(OneKeyKind),
    SetKeyPath(String),
    RefreshList,
    BeginAddKeyword(SendMode),
    CommitAddKeyword,
    CancelAddKeyword,
    RemoveKeyword(String),
    ResetKeywords(SendMode),
}

#[derive(Debug, Clone, Default)]
enum PageMode {
    #[default]
    List,
    AddForm,
    EditForm(String),
}

pub struct OneKeyPageView {
    page: PageType<Self>,
    credentials: Vec<OneKeyCredential>,
    mode: PageMode,
    edit_label: String,
    edit_username: String,
    edit_password: String,
    edit_notes: String,
    edit_kind: OneKeyKind,
    edit_key_path: String,
    label_editor: ViewHandle<EditorView>,
    username_editor: ViewHandle<EditorView>,
    password_editor: ViewHandle<EditorView>,
    notes_editor: ViewHandle<EditorView>,
    key_path_editor: ViewHandle<EditorView>,
    button_states: HashMap<String, MouseStateHandle>,
    add_button_state: MouseStateHandle,
    save_button_state: MouseStateHandle,
    cancel_button_state: MouseStateHandle,
    trigger_rules: Vec<PromptTriggerRule>,
    adding_keyword_mode: Option<SendMode>,
    keyword_editor: ViewHandle<EditorView>,
    remove_keyword_button_states: HashMap<String, MouseStateHandle>,
    reset_password_state: MouseStateHandle,
    reset_username_state: MouseStateHandle,
    add_keyword_password_state: MouseStateHandle,
    add_keyword_username_state: MouseStateHandle,
    ok_keyword_state: MouseStateHandle,
    cancel_keyword_state: MouseStateHandle,
}

impl OneKeyPageView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let label_editor = build_editor(ctx, "Label".to_string());
        let username_editor = build_editor(ctx, "Username".to_string());
        let password_editor = build_password_editor(ctx);
        let notes_editor = build_editor(ctx, "Notes (optional)".to_string());
        let key_path_editor = build_editor(ctx, "Path to SSH key".to_string());
        let keyword_editor = build_editor(ctx, "Type keyword...".to_string());

        let credentials = load_credentials();
        let mut button_states = HashMap::new();
        for c in &credentials {
            button_states.entry(format!("edit_{}", c.id)).or_default();
            button_states.entry(format!("delete_{}", c.id)).or_default();
        }

        let trigger_rules = load_or_init_rules();
        let mut remove_keyword_button_states = HashMap::new();
        for r in &trigger_rules {
            remove_keyword_button_states
                .entry(r.id.clone())
                .or_default();
        }

        let me = Self {
            page: PageType::new_monolith(OneKeyWidget::default(), None, false),
            credentials,
            mode: PageMode::List,
            edit_label: String::new(),
            edit_username: String::new(),
            edit_password: String::new(),
            edit_notes: String::new(),
            edit_kind: OneKeyKind::Password,
            edit_key_path: String::new(),
            label_editor,
            username_editor,
            password_editor,
            notes_editor,
            key_path_editor,
            button_states,
            add_button_state: MouseStateHandle::default(),
            save_button_state: MouseStateHandle::default(),
            cancel_button_state: MouseStateHandle::default(),
            trigger_rules,
            adding_keyword_mode: None,
            keyword_editor,
            remove_keyword_button_states,
            reset_password_state: MouseStateHandle::default(),
            reset_username_state: MouseStateHandle::default(),
            add_keyword_password_state: MouseStateHandle::default(),
            add_keyword_username_state: MouseStateHandle::default(),
            ok_keyword_state: MouseStateHandle::default(),
            cancel_keyword_state: MouseStateHandle::default(),
        };

        ctx.subscribe_to_model(
            &OneKeyCredentialsChangedNotifier::handle(ctx),
            |me, _, event, ctx| match event {
                OneKeyCredentialsChangedEvent::CredentialsChanged => {
                    me.refresh_list();
                    ctx.notify();
                }
            },
        );

        me
    }

    fn populate(&mut self, credential: &OneKeyCredential, ctx: &mut ViewContext<Self>) {
        self.label_editor.update(ctx, |editor, ctx| {
            editor.set_buffer_text(&credential.label, ctx);
        });
        self.username_editor.update(ctx, |editor, ctx| {
            editor.set_buffer_text(&credential.username, ctx);
        });
        self.password_editor.update(ctx, |editor, ctx| {
            editor.set_buffer_text(&credential.password, ctx);
        });
        self.notes_editor.update(ctx, |editor, ctx| {
            editor.set_buffer_text(&credential.notes, ctx);
        });
        self.key_path_editor.update(ctx, |editor, ctx| {
            if let Some(ref kp) = credential.key_path {
                editor.set_buffer_text(kp, ctx);
            }
        });
        self.edit_label = credential.label.clone();
        self.edit_username = credential.username.clone();
        self.edit_password = credential.password.to_string();
        self.edit_notes = credential.notes.clone();
        self.edit_kind = credential.kind;
        self.edit_key_path = credential.key_path.clone().unwrap_or_default();
    }

    fn sync_edit_fields(&mut self, ctx: &mut ViewContext<Self>) {
        self.edit_label = self.label_editor.as_ref(ctx).buffer_text(ctx);
        self.edit_username = self.username_editor.as_ref(ctx).buffer_text(ctx);
        self.edit_password = self.password_editor.as_ref(ctx).buffer_text(ctx);
        self.edit_notes = self.notes_editor.as_ref(ctx).buffer_text(ctx);
        self.edit_key_path = self.key_path_editor.as_ref(ctx).buffer_text(ctx);
    }

    fn refresh_list(&mut self) {
        self.credentials = load_credentials();
        self.button_states.retain(|k, _| {
            self.credentials.iter().any(|c| {
                k == &format!("edit_{}", c.id) || k == &format!("delete_{}", c.id)
            })
        });
        for c in &self.credentials {
            self.button_states
                .entry(format!("edit_{}", c.id))
                .or_default();
            self.button_states
                .entry(format!("delete_{}", c.id))
                .or_default();
        }
    }

    fn refresh_rules(&mut self) {
        self.trigger_rules = load_rules();
        self.remove_keyword_button_states.clear();
        for r in &self.trigger_rules {
            self.remove_keyword_button_states
                .entry(r.id.clone())
                .or_default();
        }
    }

    fn render_trigger_keywords_section(&self, appearance: &Appearance) -> Box<dyn Element> {
        let mut section = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Start);

        section.add_child(render_sub_header_with_description(
            appearance,
            "Auto-fill Trigger Keywords",
            "When terminal output matches a keyword (case-insensitive), the credential is auto-sent with the matching mode.",
        ));

        let password_only_rules: Vec<_> = self
            .trigger_rules
            .iter()
            .filter(|r| r.send_mode == SendMode::PasswordOnly)
            .collect();
        let username_rules: Vec<_> = self
            .trigger_rules
            .iter()
            .filter(|r| r.send_mode == SendMode::UsernameThenPassword)
            .collect();

        section.add_child(Self::render_keyword_group(
            appearance,
            "Only Password",
            &password_only_rules,
            SendMode::PasswordOnly,
            &self.remove_keyword_button_states,
            &self.adding_keyword_mode,
            &self.keyword_editor,
            &self.add_keyword_password_state,
            &self.ok_keyword_state,
            &self.cancel_keyword_state,
            &self.reset_password_state,
        ));

        section.add_child(Self::render_keyword_group(
            appearance,
            "Username + Password",
            &username_rules,
            SendMode::UsernameThenPassword,
            &self.remove_keyword_button_states,
            &self.adding_keyword_mode,
            &self.keyword_editor,
            &self.add_keyword_username_state,
            &self.ok_keyword_state,
            &self.cancel_keyword_state,
            &self.reset_username_state,
        ));

        Container::new(section.finish())
            .with_background(appearance.theme().surface_1())
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
            .with_uniform_padding(10.)
            .with_margin_bottom(16.)
            .finish()
    }

    fn render_keyword_group(
        appearance: &Appearance,
        group_label: &str,
        rules: &[&PromptTriggerRule],
        mode: SendMode,
        remove_keyword_button_states: &HashMap<String, MouseStateHandle>,
        adding_keyword_mode: &Option<SendMode>,
        keyword_editor: &ViewHandle<EditorView>,
        add_button_state: &MouseStateHandle,
        ok_button_state: &MouseStateHandle,
        cancel_button_state: &MouseStateHandle,
        reset_button_state: &MouseStateHandle,
    ) -> Box<dyn Element> {
        let mut group = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Start);

        let is_adding = adding_keyword_mode.map_or(false, |am| am == mode);

        // header row: label + +Add button + Reset button
        let mut header = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Text::new_inline(
                    format!("{}: ", group_label),
                    appearance.ui_font_family(),
                    appearance.ui_font_size(),
                )
                .with_color(appearance.theme().active_ui_text_color().into())
                .finish(),
            );

        if is_adding {
            header.add_child(
                Container::new(
                        ConstrainedBox::new(ChildView::new(keyword_editor).finish())
                        .with_width(200.)
                        .finish(),
                )
                .with_margin_right(CHIP_GAP)
                .finish(),
            );
            header.add_child(
                Container::new(
                    appearance
                        .ui_builder()
                        .button(ButtonVariant::Accent, ok_button_state.clone())
                        .with_style(UiComponentStyles {
                            font_size: Some(appearance.ui_font_size()),
                            padding: Some(Coords::uniform(3.)),
                            ..Default::default()
                        })
                        .with_text_label("OK".to_string())
                        .build()
                        .on_click(|ctx, _, _| {
                            ctx.dispatch_typed_action(
                                OneKeyPageAction::CommitAddKeyword,
                            );
                        })
                        .finish(),
                )
                .with_margin_right(4.)
                .finish(),
            );
            header.add_child(
                Container::new(
                    appearance
                        .ui_builder()
                        .button(ButtonVariant::Text, cancel_button_state.clone())
                        .with_style(UiComponentStyles {
                            font_size: Some(appearance.ui_font_size()),
                            padding: Some(Coords::uniform(3.)),
                            ..Default::default()
                        })
                        .with_text_label("Cancel".to_string())
                        .build()
                        .on_click(|ctx, _, _| {
                            ctx.dispatch_typed_action(
                                OneKeyPageAction::CancelAddKeyword,
                            );
                        })
                        .finish(),
                )
                .with_margin_right(4.)
                .finish(),
            );
        } else {
            header.add_child(
                appearance
                    .ui_builder()
                    .button(ButtonVariant::Accent, add_button_state.clone())
                    .with_style(UiComponentStyles {
                        font_size: Some(appearance.ui_font_size()),
                        padding: Some(Coords::uniform(3.)),
                        ..Default::default()
                    })
                    .with_text_label("+ Add".to_string())
                    .build()
                    .on_click(move |ctx, _, _| {
                        ctx.dispatch_typed_action(
                            OneKeyPageAction::BeginAddKeyword(mode),
                        );
                    })
                    .finish(),
            );
            if adding_keyword_mode.is_none() {
                header.add_child(
                    Container::new(
                        appearance
                            .ui_builder()
                            .button(ButtonVariant::Text, reset_button_state.clone())
                            .with_style(UiComponentStyles {
                                font_size: Some(appearance.ui_font_size()),
                                padding: Some(Coords::uniform(3.)),
                                ..Default::default()
                            })
                            .with_text_label("Reset".to_string())
                            .build()
                            .on_click(move |ctx, _, _| {
                                ctx.dispatch_typed_action(
                                    OneKeyPageAction::ResetKeywords(mode),
                                );
                            })
                            .finish(),
                    )
                    .with_margin_left(8.)
                    .finish(),
                );
            }
        }
        group.add_child(header.finish());

        // render keywords as chips
        if !rules.is_empty() {
            let mut chips = Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center);
            for rule in rules {
                let mouse = remove_keyword_button_states
                    .get(&rule.id)
                    .cloned()
                    .unwrap_or_default();
                let id = rule.id.clone();
                chips.add_child(
                    Container::new(
                        Flex::row()
                            .with_cross_axis_alignment(CrossAxisAlignment::Center)
                            .with_child(
                                Text::new_inline(
                                    rule.keyword.clone(),
                                    appearance.ui_font_family(),
                                    appearance.ui_font_size(),
                                )
                                .with_color(appearance.theme().active_ui_text_color().into())
                                .finish(),
                            )
                            .with_child(
                                Container::new(
                                    appearance
                                        .ui_builder()
                                        .button(ButtonVariant::Text, mouse)
                                        .with_style(UiComponentStyles {
                                            font_size: Some(appearance.ui_font_size()),
                                            padding: Some(Coords::uniform(2.)),
                                            ..Default::default()
                                        })
                                        .with_text_label("×".to_string())
                                        .build()
                                        .on_click(move |ctx, _, _| {
                                            ctx.dispatch_typed_action(
                                                OneKeyPageAction::RemoveKeyword(id.clone()),
                                            );
                                        })
                                        .finish(),
                                )
                                .with_margin_left(4.)
                                .finish(),
                            )
                            .finish(),
                    )
                    .with_background(appearance.theme().surface_2())
                    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(3.)))
                    .with_uniform_padding(6.)
                    .with_margin_right(CHIP_GAP)
                    .with_margin_bottom(CHIP_GAP)
                    .finish(),
                );
            }
            group.add_child(
                Container::new(chips.finish())
                    .with_margin_top(6.)
                    .finish(),
            );
        }

        group.finish()
    }

    fn render_list_mode(&self, appearance: &Appearance) -> Box<dyn Element> {
        let mut content = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(render_page_title("OneKey Credentials", appearance))
            .with_child(render_sub_header_with_description(
                appearance,
                "OneKey Credentials",
                "Manage saved credentials for quick input.",
            ));

        content.add_child(self.render_trigger_keywords_section(appearance));

        if self.credentials.is_empty() {
            content.add_child(
                Container::new(
                    Text::new(
                        "No saved credentials. Click \"Add\" to create one.",
                        appearance.ui_font_family(),
                        appearance.ui_font_body(),
                    )
                    .with_color(appearance.theme().nonactive_ui_text_color().into())
                    .finish(),
                )
                .with_margin_bottom(12.)
                .finish(),
            );
        } else {
            for credential in &self.credentials {
                let id = credential.id.clone();
                let edit_mouse = self
                    .button_states
                    .get(&format!("edit_{}", id))
                    .cloned()
                    .unwrap_or_default();
                let delete_mouse = self
                    .button_states
                    .get(&format!("delete_{}", id))
                    .cloned()
                    .unwrap_or_default();

                let item = Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_child(
                        Expanded::new(
                            1.,
                            Flex::column()
                                .with_child(
                                    Text::new_inline(
                                        credential.label.clone(),
                                        appearance.ui_font_family(),
                                        appearance.ui_font_body(),
                                    )
                                    .with_style(Properties::default().weight(Weight::Semibold))
                                    .with_color(appearance.theme().active_ui_text_color().into())
                                    .finish(),
                                )
                                .with_child(
                                    Text::new_inline(
                                        credential.username.clone(),
                                        appearance.ui_font_family(),
                                        appearance.ui_font_size(),
                                    )
                                    .with_color(
                                        appearance.theme().nonactive_ui_text_color().into(),
                                    )
                                    .finish(),
                                )
                                .finish(),
                        )
                        .finish(),
                    )
                    .with_child(render_small_button(
                        appearance,
                        "Edit".to_string(),
                        edit_mouse,
                        OneKeyPageAction::ShowEditForm(id.clone()),
                    ))
                    .with_child(
                        Container::new(render_small_button(
                            appearance,
                            "Delete".to_string(),
                            delete_mouse,
                            OneKeyPageAction::ShowDeleteConfirmation(id.clone()),
                        ))
                        .with_margin_left(4.)
                        .finish(),
                    );

                content.add_child(
                    Container::new(item.finish())
                        .with_background(appearance.theme().surface_1())
                        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
                        .with_uniform_padding(10.)
                        .with_margin_bottom(6.)
                        .finish(),
                );
            }
        }

        content.add_child(
            Container::new(
                appearance
                    .ui_builder()
                    .button(
                        ButtonVariant::Accent,
                        self.add_button_state.clone(),
                    )
                    .with_text_label("+ Add Credential".to_string())
                    .build()
                    .on_click(|ctx, _, _| {
                        ctx.dispatch_typed_action(OneKeyPageAction::ShowAddForm);
                    })
                    .finish(),
            )
            .with_margin_top(8.)
            .finish(),
        );

        content.finish()
    }

    fn render_kind_toggle(&self, appearance: &Appearance) -> Box<dyn Element> {
        let password_variant = if self.edit_kind == OneKeyKind::Password {
            ButtonVariant::Accent
        } else {
            ButtonVariant::Text
        };
        let key_variant = if self.edit_kind == OneKeyKind::Key {
            ButtonVariant::Accent
        } else {
            ButtonVariant::Text
        };

        Flex::column()
            .with_child(
                Text::new_inline(
                    "Kind".to_string(),
                    appearance.ui_font_family(),
                    appearance.ui_font_size(),
                )
                .with_color(appearance.theme().active_ui_text_color().into())
                .finish(),
            )
            .with_child(
                Container::new(
                    Flex::row()
                        .with_child(
                            appearance
                                .ui_builder()
                                .button(password_variant, MouseStateHandle::default())
                                .with_text_label("Password".to_string())
                                .build()
                                .on_click(|ctx, _, _| {
                                    ctx.dispatch_typed_action(
                                        OneKeyPageAction::SetKind(OneKeyKind::Password),
                                    );
                                })
                                .finish(),
                        )
                        .with_child(
                            Container::new(
                                appearance
                                    .ui_builder()
                                    .button(key_variant, MouseStateHandle::default())
                                    .with_text_label("SSH Key".to_string())
                                    .build()
                                    .on_click(|ctx, _, _| {
                                        ctx.dispatch_typed_action(
                                            OneKeyPageAction::SetKind(OneKeyKind::Key),
                                        );
                                    })
                                    .finish(),
                            )
                            .with_margin_left(4.)
                            .finish(),
                        )
                        .finish(),
                )
                .with_margin_top(4.)
                .with_margin_bottom(FORM_HALF_GAP)
                .finish(),
            )
            .finish()
    }

    fn render_form_mode(
        &self,
        is_edit: bool,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let title = if is_edit {
            "Edit Credential"
        } else {
            "Add Credential"
        };

        let mut content = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(render_page_title(title, appearance));

        content.add_child(render_field_with_editor(
            appearance,
            "Label".to_string(),
            &self.label_editor,
        ));

        content.add_child(render_field_with_editor(
            appearance,
            "Username".to_string(),
            &self.username_editor,
        ));

        content.add_child(render_field_with_editor(
            appearance,
            "Password".to_string(),
            &self.password_editor,
        ));

        content.add_child(render_field_with_editor(
            appearance,
            "Notes".to_string(),
            &self.notes_editor,
        ));

        content.add_child(self.render_kind_toggle(appearance));

        if self.edit_kind == OneKeyKind::Key {
            content.add_child(render_field_with_editor(
                appearance,
                "SSH Key Path".to_string(),
                &self.key_path_editor,
            ));
        }

        content.add_child(
            Container::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(
                        appearance
                            .ui_builder()
                            .button(ButtonVariant::Accent, self.save_button_state.clone())
                            .with_text_label("Save".to_string())
                            .build()
                            .on_click(|ctx, _, _| {
                                ctx.dispatch_typed_action(OneKeyPageAction::SaveForm);
                            })
                            .finish(),
                    )
                    .with_child(
                        Container::new(
                            appearance
                                .ui_builder()
                                .button(ButtonVariant::Text, self.cancel_button_state.clone())
                                .with_text_label("Cancel".to_string())
                                .build()
                                .on_click(|ctx, _, _| {
                                    ctx.dispatch_typed_action(
                                        OneKeyPageAction::CancelForm,
                                    );
                                })
                                .finish(),
                        )
                        .with_margin_left(6.)
                        .finish(),
                    )
                    .finish(),
            )
            .with_margin_top(12.)
            .finish(),
        );

        content.finish()
    }

}

impl Entity for OneKeyPageView {
    type Event = SettingsPageEvent;
}

impl TypedActionView for OneKeyPageView {
    type Action = OneKeyPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            OneKeyPageAction::ShowAddForm => {
                self.mode = PageMode::AddForm;
                self.edit_label = String::new();
                self.edit_username = String::new();
                self.edit_password = String::new();
                self.edit_notes = String::new();
                self.edit_kind = OneKeyKind::Password;
                self.edit_key_path = String::new();
                self.populate(
                    &OneKeyCredential {
                        id: String::new(),
                        label: String::new(),
                        username: String::new(),
                        notes: String::new(),
                        password: Default::default(),
                        kind: OneKeyKind::Password,
                        key_path: None,
                    },
                    ctx,
                );
                ctx.notify();
            }
            OneKeyPageAction::ShowEditForm(credential_id) => {
                if let Some(credential) = self
                    .credentials
                    .iter()
                    .find(|c| c.id == *credential_id)
                    .cloned()
                {
                    self.mode = PageMode::EditForm(credential.id.clone());
                    self.populate(&credential, ctx);
                    ctx.notify();
                }
            }
            OneKeyPageAction::CancelForm => {
                self.mode = PageMode::List;
                ctx.notify();
            }
            OneKeyPageAction::SaveForm => {
                self.sync_edit_fields(ctx);
                match &self.mode {
                    PageMode::AddForm => {
                        let key_path = if self.edit_kind == OneKeyKind::Key
                            && !self.edit_key_path.is_empty()
                        {
                            Some(self.edit_key_path.clone())
                        } else {
                            None
                        };
                        let credential = OneKeyCredential {
                            id: String::new(),
                            label: std::mem::take(&mut self.edit_label),
                            username: std::mem::take(&mut self.edit_username),
                            notes: std::mem::take(&mut self.edit_notes),
                            password: std::mem::take(&mut self.edit_password).into(),
                            kind: self.edit_kind,
                            key_path,
                        };
                        report_if_error!(warp_onekey::create(&credential));
                    }
                    PageMode::EditForm(credential_id) => {
                        let key_path = if self.edit_kind == OneKeyKind::Key
                            && !self.edit_key_path.is_empty()
                        {
                            Some(self.edit_key_path.clone())
                        } else {
                            None
                        };
                        let credential = OneKeyCredential {
                            id: credential_id.clone(),
                            label: std::mem::take(&mut self.edit_label),
                            username: std::mem::take(&mut self.edit_username),
                            notes: std::mem::take(&mut self.edit_notes),
                            password: std::mem::take(&mut self.edit_password).into(),
                            kind: self.edit_kind,
                            key_path,
                        };
                        report_if_error!(warp_onekey::update(&credential));
                    }
                    _ => {}
                }
                OneKeyCredentialsChangedNotifier::handle(ctx).update(ctx, |_, ctx| {
                    ctx.emit(OneKeyCredentialsChangedEvent::CredentialsChanged);
                });
                self.mode = PageMode::List;
                self.refresh_list();
                ctx.notify();
            }
            OneKeyPageAction::ShowDeleteConfirmation(credential_id) => {
                let id = credential_id.clone();
                let view_handle = ctx.handle();
                let label = self
                    .credentials
                    .iter()
                    .find(|c| c.id == *credential_id)
                    .map(|c| c.label.clone())
                    .unwrap_or_default();
                let dialog = AlertDialogWithCallbacks::for_app(
                    format!("Delete \"{label}\"?"),
                    "This action cannot be undone.",
                    vec![
                        ModalButton::for_app("Delete", {
                            let view_handle = view_handle.clone();
                            move |app| {
                                let _ = warp_onekey::delete(&id);
                                let notifier =
                                    OneKeyCredentialsChangedNotifier::handle(&*app);
                                app.update_model(&notifier, |_, ctx| {
                                    ctx.emit(
                                        OneKeyCredentialsChangedEvent::CredentialsChanged,
                                    );
                                });
                                if let Some(handle) = view_handle.upgrade(app) {
                                    app.update_view(&handle, |me, ctx| {
                                        me.refresh_list();
                                        ctx.notify();
                                    });
                                }
                            }
                        }),
                        ModalButton::for_app("Cancel", |_| {}),
                    ],
                    |_| {},
                );
                let window_id = ctx.window_id();
                let workspace = ctx
                    .views_of_type::<crate::workspace::Workspace>(window_id)
                    .and_then(|workspaces| workspaces.first().cloned());
                if let Some(workspace) = workspace {
                    workspace.update(ctx, |view, ctx| {
                        view.show_native_modal(dialog, ctx);
                    });
                }
            }
            OneKeyPageAction::SetLabel(label) => {
                self.edit_label = label.clone();
            }
            OneKeyPageAction::SetUsername(username) => {
                self.edit_username = username.clone();
            }
            OneKeyPageAction::SetPassword(password) => {
                self.edit_password = password.clone();
            }
            OneKeyPageAction::SetNotes(notes) => {
                self.edit_notes = notes.clone();
            }
            OneKeyPageAction::SetKind(kind) => {
                self.edit_kind = *kind;
                if *kind == OneKeyKind::Password {
                    self.edit_key_path.clear();
                    self.key_path_editor.update(ctx, |editor, ctx| {
                        editor.clear_buffer(ctx);
                    });
                }
                ctx.notify();
            }
            OneKeyPageAction::SetKeyPath(key_path) => {
                self.edit_key_path = key_path.clone();
            }
            OneKeyPageAction::RefreshList => {
                self.refresh_list();
                ctx.notify();
            }
            OneKeyPageAction::BeginAddKeyword(mode) => {
                self.adding_keyword_mode = Some(*mode);
                self.keyword_editor.update(ctx, |editor, ctx| {
                    editor.clear_buffer(ctx);
                });
                ctx.notify();
            }
            OneKeyPageAction::CommitAddKeyword => {
                let keyword = self.keyword_editor.as_ref(ctx).buffer_text(ctx);
                if !keyword.is_empty() {
                    if let Some(mode) = self.adding_keyword_mode {
                        report_if_error!(warp_onekey::add_rule(&keyword, mode));
                    }
                }
                self.adding_keyword_mode = None;
                self.refresh_rules();
                ctx.notify();
            }
            OneKeyPageAction::CancelAddKeyword => {
                self.adding_keyword_mode = None;
                ctx.notify();
            }
            OneKeyPageAction::RemoveKeyword(rule_id) => {
                report_if_error!(warp_onekey::remove_rule(rule_id));
                self.refresh_rules();
                ctx.notify();
            }
            OneKeyPageAction::ResetKeywords(mode) => {
                report_if_error!(warp_onekey::reset_rules_for_mode(*mode));
                self.refresh_rules();
                ctx.notify();
            }
        }
    }
}

impl View for OneKeyPageView {
    fn ui_name() -> &'static str {
        "OneKeyPage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

impl SettingsPageMeta for OneKeyPageView {
    fn section() -> SettingsSection {
        SettingsSection::OneKey
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id);
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

fn load_credentials() -> Vec<OneKeyCredential> {
    warp_onekey::find_all().unwrap_or_default()
}

fn load_rules() -> Vec<PromptTriggerRule> {
    warp_onekey::list_rules().unwrap_or_default()
}

fn load_or_init_rules() -> Vec<PromptTriggerRule> {
    let rules = load_rules();
    if rules.is_empty() {
        let _ = warp_onekey::reset_rules_to_defaults();
        load_rules()
    } else {
        rules
    }
}

fn build_editor(
    ctx: &mut ViewContext<OneKeyPageView>,
    placeholder: String,
) -> ViewHandle<EditorView> {
    ctx.add_typed_action_view(move |ctx| {
        let appearance = Appearance::as_ref(ctx);
        let options = SingleLineEditorOptions {
            text: TextOptions {
                font_size_override: Some(appearance.ui_font_size()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut editor = EditorView::single_line(options, ctx);
        editor.set_placeholder_text(placeholder, ctx);
        editor
    })
}

fn build_password_editor(
    ctx: &mut ViewContext<OneKeyPageView>,
) -> ViewHandle<EditorView> {
    ctx.add_typed_action_view(move |ctx| {
        let appearance = Appearance::as_ref(ctx);
        let options = SingleLineEditorOptions {
            is_password: true,
            text: TextOptions {
                font_size_override: Some(appearance.ui_font_size()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut editor = EditorView::single_line(options, ctx);
        editor.set_placeholder_text("Password".to_string(), ctx);
        editor
    })
}

fn render_small_button(
    appearance: &Appearance,
    text: String,
    mouse_state: MouseStateHandle,
    action: OneKeyPageAction,
) -> Box<dyn Element> {
    appearance
        .ui_builder()
        .button(ButtonVariant::Text, mouse_state)
        .with_style(UiComponentStyles {
            font_size: Some(appearance.ui_font_size()),
            padding: Some(Coords::uniform(4.)),
            ..Default::default()
        })
        .with_text_label(text)
        .build()
        .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()))
        .finish()
}

fn render_field_with_editor(
    appearance: &Appearance,
    label: String,
    editor: &ViewHandle<EditorView>,
) -> Box<dyn Element> {
    Container::new(render_field_with_editor_inner(appearance, label, editor))
        .with_margin_bottom(FORM_HALF_GAP)
        .finish()
}

fn render_field_with_editor_inner(
    appearance: &Appearance,
    label: String,
    editor: &ViewHandle<EditorView>,
) -> Box<dyn Element> {
    Flex::column()
        .with_child(
            Text::new_inline(label, appearance.ui_font_family(), appearance.ui_font_size())
                .with_color(appearance.theme().active_ui_text_color().into())
                .finish(),
        )
        .with_child(
            Container::new(ChildView::new(editor).finish())
                .with_margin_top(4.)
                .finish(),
        )
        .finish()
}

#[derive(Default)]
struct OneKeyWidget;

impl SettingsWidget for OneKeyWidget {
    type View = OneKeyPageView;

    fn search_terms(&self) -> &str {
        "onekey credentials 快速凭证 password 密码 username 用户名 credential"
    }

    fn render(
        &self,
        view: &OneKeyPageView,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        match &view.mode {
            PageMode::List => view.render_list_mode(appearance),
            PageMode::AddForm | PageMode::EditForm(_) => {
                let is_edit = matches!(view.mode, PageMode::EditForm(_));
                view.render_form_mode(is_edit, appearance)
            }
        }
    }
}
