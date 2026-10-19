use super::*;
use crate::settings_panel::{self, Category};
use ferrostage_render::{CacheBudgetAck, CacheBudgets, Reply};
use ferrostage_settings::Settings;
use std::sync::mpsc::TryRecvError;

pub(super) struct PendingPreferences {
    previous: Settings,
    next: Settings,
    reply: Option<Reply<CacheBudgetAck>>,
    rollback: Option<String>,
    return_settings: bool,
}
fn budgets(settings: &Settings) -> CacheBudgets {
    CacheBudgets {
        l2_bytes: settings.cache.l2_cpu_mib as usize * 1024 * 1024,
        l3_bytes: settings.cache.l3_thumb_mib as usize * 1024 * 1024,
    }
}
impl Shell {
    pub(crate) fn captures_preferences(&self, window: gpui::WindowId) -> bool {
        self.modal == Some(Modal::Settings)
            && self
                .presenter
                .as_ref()
                .is_some_and(|record| record.handle.window_id() == window)
            && (self.preferences.edit.is_some()
                || self.preferences.recording.is_some()
                || self.preferences.font_picker.is_some())
    }

    pub(super) fn dismiss_preferences(&mut self) {
        self.preferences.edit = None;
        self.preferences.recording = None;
        self.preferences.font_picker = None;
        if let Some(pending) = &mut self.preference_change {
            pending.return_settings = false;
        }
    }
    pub(crate) fn config_path(&self) -> &std::path::Path {
        self.store.path()
    }
    pub(super) fn dispatch_preference(&mut self, intent: &Intent, cx: &mut Context<Self>) -> bool {
        match intent {
            Intent::SettingsCategory(category) => {
                if self.busy() {
                    return true;
                }
                self.preferences.confirm_reset = false;
                self.preferences.category = *category;
                self.preferences.font_picker = None;
                self.preferences.edit = None;
                self.preferences.recording = None;
                if *category == Category::Displays {
                    self.detect_displays(cx);
                    self.preferences.display = Some(DisplayDraft::new(
                        self.topology_revision,
                        &self.monitors,
                        &self.settings.settings.display,
                    ));
                }
            }
            Intent::ToggleFontPicker => {
                if self.preferences_editable() {
                    if !self.ui_fonts_loaded {
                        self.ui_fonts = std::sync::Arc::new(cx.text_system().all_font_names());
                        self.ui_fonts_loaded = true;
                    }
                    self.preferences.font_picker = if self.preferences.font_picker.is_some() {
                        None
                    } else {
                        Some(Default::default())
                    };
                    self.preferences.edit = None;
                    self.preferences.recording = None;
                }
            }
            Intent::FontPickerMove(delta) => {
                if let Some(picker) = &mut self.preferences.font_picker {
                    let count = picker.matches(&self.ui_fonts).len();
                    picker.selected = picker.selected.saturating_add_signed(*delta).min(count - 1);
                }
            }
            Intent::EditSetting(id, input) => {
                if !self.preferences_editable() {
                    return true;
                }
                match settings_panel::edit(&self.settings.settings, id, input) {
                    Ok(next) => {
                        if *id == "general.ui_font_family" {
                            self.preferences.font_picker = None;
                        }
                        self.apply_preferences(next, cx)
                    }
                    Err(error) => self.preferences.error = Some(error),
                }
            }
            Intent::BeginSettingEdit(id) => {
                if self.preferences_editable()
                    && let Some(field) = settings_panel::FIELDS.iter().find(|field| field.id == *id)
                {
                    self.preferences.edit =
                        Some((id, settings_panel::value(&self.settings.settings, *field)));
                    self.preferences.recording = None;
                    self.preferences.font_picker = None;
                }
            }
            Intent::SettingsKey { key, text } => self.preference_key(key, text.as_deref(), cx),
            Intent::RecordBinding(action) => {
                if self.preferences_editable() {
                    self.preferences.recording = Some(action);
                    self.preferences.edit = None;
                    self.preferences.error = None;
                }
            }
            Intent::RemoveBinding(action, index) => {
                if self.preferences_editable() {
                    let mut next = self.settings.settings.clone();
                    if let Some(keys) = settings_panel::bindings_mut(&mut next.keybindings, action)
                        && *index < keys.len()
                    {
                        keys.remove(*index);
                        self.apply_preferences(next, cx);
                    }
                }
            }
            Intent::ResetBindings => {
                if self.preferences_editable() {
                    let mut next = self.settings.settings.clone();
                    next.keybindings = Default::default();
                    self.apply_preferences(next, cx);
                }
            }
            Intent::ClearRecent => {
                if self.preferences_editable() {
                    let mut next = self.settings.settings.clone();
                    next.general.recent_files.clear();
                    next.general.recent_opened_at.clear();
                    self.apply_preferences(next, cx);
                }
            }
            Intent::OpenConfigDirectory => {
                if self.preferences_editable() {
                    cx.reveal_path(self.store.path());
                }
            }
            Intent::ResetPreferences(confirm) => {
                if !self.preferences_editable() {
                    return true;
                }
                self.preferences.confirm_reset = !confirm;
                if *confirm {
                    let next = Settings {
                        hot_reload: self.settings.settings.hot_reload.clone(),
                        ..Default::default()
                    };
                    self.apply_preferences(next, cx);
                }
            }
            _ => return false,
        }
        true
    }
    fn preferences_editable(&self) -> bool {
        !self.busy() && self.settings.write_policy.is_writable()
    }
    fn preference_key(&mut self, key: &str, text: Option<&str>, cx: &mut Context<Self>) {
        if !self.preferences_editable() {
            return;
        }
        if key == "escape" {
            self.preferences.recording = None;
            self.preferences.edit = None;
            self.preferences.font_picker = None;
            return;
        }
        if let Some(picker) = &mut self.preferences.font_picker {
            let matches = picker.matches(&self.ui_fonts);
            match key {
                "enter" => {
                    let family = matches[picker.selected.min(matches.len() - 1)]
                        .unwrap_or("")
                        .to_string();
                    self.dispatch_preference(
                        &Intent::EditSetting("general.ui_font_family", family),
                        cx,
                    );
                }
                "up" | "down" | "pageup" | "pagedown" => {
                    let delta = match key {
                        "up" => -1,
                        "down" => 1,
                        "pageup" => -6,
                        _ => 6,
                    };
                    picker.selected = picker
                        .selected
                        .saturating_add_signed(delta)
                        .min(matches.len() - 1);
                }
                "tab" | "shift-tab" => self.preferences.font_picker = None,
                "backspace" => {
                    picker.filter.pop();
                    picker.selected = 0;
                }
                "ctrl-a" | "cmd-a" => {
                    picker.filter.clear();
                    picker.selected = 0;
                }
                "ctrl-v" | "cmd-v" | "shift-insert" => {
                    if let Some(text) = text
                        && !text.chars().any(char::is_control)
                        && picker.filter.len().saturating_add(text.len()) <= 128
                    {
                        picker.filter.push_str(text);
                        picker.selected = 0;
                    }
                }
                _ if key.split('-').any(|part| {
                    matches!(part, "ctrl" | "cmd" | "alt" | "super" | "win" | "fn")
                }) => {}
                _ => {
                    if let Some(text) = text
                        && !text.chars().any(char::is_control)
                        && picker.filter.len().saturating_add(text.len()) <= 128
                    {
                        picker.filter.push_str(text);
                        picker.selected = 0;
                    }
                }
            }
            return;
        }
        if let Some(action) = self.preferences.recording {
            if let Some(key) = policy::canonical_key(key) {
                let mut next = self.settings.settings.clone();
                if let Some(keys) = settings_panel::bindings_mut(&mut next.keybindings, action) {
                    if !keys
                        .iter()
                        .any(|old| policy::canonical_key(old).as_ref() == Some(&key))
                    {
                        keys.push(key);
                    }
                    self.preferences.recording = None;
                    self.apply_preferences(next, cx);
                }
            } else {
                self.preferences.error = Some(self.locale.text("settings-invalid-key").into());
            }
            return;
        }
        if let Some((id, input)) = &mut self.preferences.edit {
            match key {
                "enter" => {
                    let id = *id;
                    let input = input.clone();
                    self.dispatch_preference(&Intent::EditSetting(id, input), cx);
                }
                "backspace" => {
                    input.pop();
                }
                "ctrl-a" | "cmd-a" => input.clear(),
                _ => {
                    if let Some(text) = text
                        && text.is_ascii()
                        && input.len() < 32
                    {
                        input.push_str(text);
                    }
                }
            }
        }
    }
    fn preflight_gpu(&self, proposed: &Settings, cx: &App) -> Result<(), String> {
        if proposed.cache.l1_gpu_mib == self.settings.settings.cache.l1_gpu_mib {
            return Ok(());
        }
        let bytes = proposed.cache.l1_gpu_mib as usize * 1024 * 1024 / 2;
        if let Some(record) = &self.presenter {
            let handle = record
                .handle
                .downcast::<Presenter>()
                .ok_or("Presenter unavailable")?;
            if !handle
                .read(cx)
                .map_err(|e| e.to_string())?
                .accepts_texture_budget(bytes)
            {
                return Err(self.locale.text("settings-gpu-budget").into());
            }
        }
        if let Some(record) = &self.audience {
            let handle = record
                .handle
                .downcast::<Audience>()
                .ok_or("Audience unavailable")?;
            if !handle
                .read(cx)
                .map_err(|e| e.to_string())?
                .accepts_texture_budget(bytes, cx)
            {
                return Err(self.locale.text("settings-gpu-budget").into());
            }
        }
        Ok(())
    }
    fn apply_gpu(&self, proposed: &Settings, cx: &mut Context<Self>) -> Result<(), String> {
        let bytes = proposed.cache.l1_gpu_mib as usize * 1024 * 1024 / 2;
        if let Some(record) = &self.presenter {
            record
                .handle
                .downcast::<Presenter>()
                .ok_or("Presenter unavailable")?
                .update(cx, |view, _, _| view.apply_texture_budget(bytes))
                .map_err(|e| e.to_string())??;
        }
        self.snapshot.update(cx, |snapshot, cx| {
            if let Some(visual) = &mut snapshot.visual {
                visual.texture_budget_bytes = bytes;
            }
            cx.notify();
        });
        if let Some(record) = &self.audience {
            record
                .handle
                .downcast::<Audience>()
                .ok_or("Audience unavailable")?
                .update(cx, |view, _, cx| view.commit_snapshot_budget(cx))
                .map_err(|e| e.to_string())??;
        }
        Ok(())
    }
    pub(super) fn apply_preferences(&mut self, next: Settings, cx: &mut Context<Self>) {
        if !self.preferences_editable() {
            return;
        }
        if let Err(error) = self.preflight_gpu(&next, cx) {
            self.preferences.error = Some(error);
            return;
        }
        // Validate the checked core consumer before any external application.
        let mut probe = if self.state.stage() == SessionStage::Idle {
            ferrostage_core::PresentationState::new(1).expect("one validation page")
        } else {
            self.state.clone()
        };
        if let Err(error) = probe.apply_at(
            Command::SetLaserStyle(ferrostage_core::LaserStyle {
                color_bgra: next.pointer.laser_color_bgra,
                radius_norm: next.pointer.laser_radius_norm as f32,
            }),
            Instant::now(),
        ) {
            self.preferences.error = Some(error.to_string());
            return;
        }
        let reply = if budgets(&next) != budgets(&self.settings.settings) {
            match self
                .renderer
                .as_ref()
                .map_err(ToString::to_string)
                .and_then(|worker| {
                    worker
                        .set_cache_budgets(budgets(&next))
                        .map_err(|e| e.to_string())
                }) {
                Ok(reply) => Some(reply),
                Err(error) => {
                    self.preferences.error = Some(error);
                    return;
                }
            }
        } else {
            None
        };
        self.preferences.error = None;
        self.preference_change = Some(PendingPreferences {
            previous: self.settings.settings.clone(),
            next,
            reply,
            rollback: None,
            return_settings: self.modal == Some(Modal::Settings),
        });
        if self
            .preference_change
            .as_ref()
            .is_some_and(|pending| pending.reply.is_none())
        {
            self.continue_preferences(cx);
        }
    }
    pub(super) fn poll_preferences(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.preference_change.as_mut() else {
            return;
        };
        let Some(reply) = &pending.reply else {
            return;
        };
        let result = match reply.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(RenderError::WorkerUnavailable),
        };
        pending.reply = None;
        if let Some(error) = pending.rollback.take() {
            let message = match result {
                Ok(_) => error,
                Err(rollback) => format!("{error}; rollback failed: {rollback}"),
            };
            if pending.return_settings {
                self.modal = Some(Modal::Settings);
            }
            self.preferences.error = Some(message);
            self.preference_change = None;
            cx.notify();
            return;
        }
        match result {
            Ok(_) => self.continue_preferences(cx),
            Err(error) => {
                self.preferences.error = Some(error.to_string());
                self.preference_change = None;
            }
        }
        cx.notify();
    }
    fn continue_preferences(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = &self.preference_change else {
            return;
        };
        let next = pending.next.clone();
        let mut old_display = self.settings.settings.display.clone();
        old_display.presenter_start_fullscreen = next.display.presenter_start_fullscreen;
        old_display.audience_click_to_advance = next.display.audience_click_to_advance;
        if next.display != old_display {
            self.assign_displays(next.display.clone(), cx);
            if self.pending.is_none() {
                self.rollback_preferences(
                    self.error
                        .clone()
                        .unwrap_or_else(|| "Display application failed".into()),
                    cx,
                );
            }
        } else {
            self.commit_preferences(cx);
        }
    }
    pub(super) fn commit_preferences(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(pending) = &self.preference_change else {
            return false;
        };
        let next = pending.next.clone();
        let mut state = self.state.clone();
        if state.stage() != SessionStage::Idle
            && let Err(error) = state.apply_at(
                Command::SetLaserStyle(ferrostage_core::LaserStyle {
                    color_bgra: next.pointer.laser_color_bgra,
                    radius_norm: next.pointer.laser_radius_norm as f32,
                }),
                Instant::now(),
            )
        {
            self.rollback_preferences(error.to_string(), cx);
            return false;
        }
        let watch_update = if next.hot_reload != pending.previous.hot_reload {
            match self.prepare_pdf_watch(&next.hot_reload) {
                Ok(watch) => Some(watch),
                Err(error) => {
                    self.rollback_preferences(error, cx);
                    return false;
                }
            }
        } else {
            None
        };
        if next.cache.l1_gpu_mib != pending.previous.cache.l1_gpu_mib
            && let Err(error) = self
                .preflight_gpu(&next, cx)
                .and_then(|()| self.apply_gpu(&next, cx))
        {
            self.rollback_preferences(error, cx);
            return false;
        }
        if let Some(watch) = watch_update {
            self.install_pdf_watch(watch);
        }
        let Some(pending) = self.preference_change.take() else {
            return false;
        };
        let changed_keys = pending.next.keybindings != self.settings.settings.keybindings;
        self.state = state;
        self.settings.settings = pending.next;
        self.preferences.edit = None;
        self.preferences.error = None;
        self.preferences.confirm_reset = false;
        if pending.return_settings {
            self.modal = Some(Modal::Settings);
        }
        if changed_keys {
            actions::replace_bindings(&self.settings.settings.keybindings, self.locale, cx);
            self.escape_stops = policy::bindings(&self.settings.settings.keybindings, self.locale)
                .0
                .iter()
                .any(|(key, action)| key == "escape" && *action == "stop_presentation");
        }
        self.settings.note_edit();
        self.publish(self.pending.is_some(), cx);
        self.save(cx);
        true
    }
    pub(super) fn rollback_preferences(&mut self, error: String, cx: &mut Context<Self>) {
        let Some(mut pending) = self.preference_change.take() else {
            return;
        };
        let mut error = error;
        if pending.previous.cache.l1_gpu_mib != pending.next.cache.l1_gpu_mib
            && let Err(rollback) = self.apply_gpu(&pending.previous, cx)
        {
            error = format!("{error}; GPU rollback failed: {rollback}");
        }
        if budgets(&pending.previous) != budgets(&pending.next) {
            match self
                .renderer
                .as_ref()
                .map_err(ToString::to_string)
                .and_then(|worker| {
                    worker
                        .set_cache_budgets(budgets(&pending.previous))
                        .map_err(|e| e.to_string())
                }) {
                Ok(reply) => {
                    pending.reply = Some(reply);
                    pending.rollback = Some(error);
                    self.preference_change = Some(pending);
                    return;
                }
                Err(rollback) => error = format!("{error}; worker rollback failed: {rollback}"),
            }
        }
        if pending.return_settings {
            self.modal = Some(Modal::Settings);
        }
        self.preferences.error = Some(error);
    }
}
