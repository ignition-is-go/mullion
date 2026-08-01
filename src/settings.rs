use std::sync::Arc;

use leptos::prelude::*;

use crate::focus::PaneFocusBehavior;

/// One allowed value in a typed Mullion setting.
///
/// Settings integrations can use this metadata to populate their own select,
/// radio-group, or settings-search UI without depending on a Mullion-rendered
/// settings page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MullionSettingOption<T: 'static> {
    value: T,
    label: &'static str,
    description: &'static str,
}

impl<T: 'static> MullionSettingOption<T> {
    const fn new(value: T, label: &'static str, description: &'static str) -> Self {
        Self {
            value,
            label,
            description,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub const fn description(&self) -> &'static str {
        self.description
    }
}

/// A live, typed setting that a host application can present in its own UI.
///
/// The value signal and setter remain connected to the host's source of truth.
/// Mullion supplies stable metadata and options, but deliberately does not own
/// a settings screen or persistence format.
pub struct MullionSetting<T: Send + Sync + 'static> {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    options: &'static [MullionSettingOption<T>],
    value: ArcSignal<T>,
    set_value: Arc<dyn Fn(T) + Send + Sync>,
}

impl<T: Send + Sync + 'static> Clone for MullionSetting<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            label: self.label,
            description: self.description,
            options: self.options,
            value: self.value.clone(),
            set_value: self.set_value.clone(),
        }
    }
}

impl<T: Send + Sync + 'static> MullionSetting<T> {
    fn new(
        id: &'static str,
        label: &'static str,
        description: &'static str,
        options: &'static [MullionSettingOption<T>],
        value: ArcSignal<T>,
        set_value: Arc<dyn Fn(T) + Send + Sync>,
    ) -> Self {
        Self {
            id,
            label,
            description,
            options,
            value,
            set_value,
        }
    }

    /// Stable id suitable for a host settings registry.
    pub const fn id(&self) -> &'static str {
        self.id
    }

    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub const fn description(&self) -> &'static str {
        self.description
    }

    pub const fn options(&self) -> &'static [MullionSettingOption<T>] {
        self.options
    }

    /// Clone the live value signal for a host-controlled settings UI.
    pub fn value_signal(&self) -> ArcSignal<T> {
        self.value.clone()
    }

    /// Send a new value to the host-provided setter.
    pub fn set(&self, value: T) {
        (self.set_value)(value);
    }
}

impl<T> MullionSetting<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Read the current value and track it in the current reactive observer.
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Read the current value without creating a reactive dependency.
    pub fn get_untracked(&self) -> T {
        self.value.get_untracked()
    }
}

const FOCUS_BEHAVIOR_OPTIONS: [MullionSettingOption<PaneFocusBehavior>; 2] = [
    MullionSettingOption::new(
        PaneFocusBehavior::Click,
        "Click",
        "Focus a pane when it is clicked and keep focus there.",
    ),
    MullionSettingOption::new(
        PaneFocusBehavior::Hover,
        "Hover",
        "Move focus whenever the pointer enters another pane.",
    ),
];

/// Reactive Mullion preferences shared by the pane system and the host app.
///
/// Use [`Self::controlled`] when the application already owns settings state.
/// Use [`Self::local`] for a self-contained instance. Cloning this handle keeps
/// every clone connected to the same value and setter.
#[derive(Clone)]
pub struct MullionSettings {
    focus_behavior: MullionSetting<PaneFocusBehavior>,
}

impl MullionSettings {
    /// Bind Mullion to an application-owned focus preference.
    ///
    /// `focus_behavior` may be any Leptos signal convertible to `ArcSignal`.
    /// The setter is where the host can update its store, persist the value, or
    /// dispatch its own settings action.
    pub fn controlled(
        focus_behavior: impl Into<ArcSignal<PaneFocusBehavior>>,
        set_focus_behavior: impl Fn(PaneFocusBehavior) + Send + Sync + 'static,
    ) -> Self {
        Self {
            focus_behavior: MullionSetting::new(
                "mullion.focus_behavior",
                "Pane focus behavior",
                "Choose whether pointer hover or click changes the focused pane.",
                &FOCUS_BEHAVIOR_OPTIONS,
                focus_behavior.into(),
                Arc::new(set_focus_behavior),
            ),
        }
    }

    /// Create settings owned by this handle, initialized to `focus_behavior`.
    pub fn local(focus_behavior: PaneFocusBehavior) -> Self {
        let value = ArcRwSignal::new(focus_behavior);
        let set_value = value.clone();
        Self::controlled(value, move |next| set_value.set(next))
    }

    /// Descriptor, live value, and setter for a host settings registry or UI.
    pub fn focus_behavior_setting(&self) -> MullionSetting<PaneFocusBehavior> {
        self.focus_behavior.clone()
    }

    /// Read the focus preference and track it in the current reactive observer.
    pub fn focus_behavior(&self) -> PaneFocusBehavior {
        self.focus_behavior.get()
    }

    /// Read the focus preference without creating a reactive dependency.
    pub fn focus_behavior_untracked(&self) -> PaneFocusBehavior {
        self.focus_behavior.get_untracked()
    }

    /// Update the preference through the host-provided setter.
    pub fn set_focus_behavior(&self, focus_behavior: PaneFocusBehavior) {
        self.focus_behavior.set(focus_behavior);
    }
}

impl Default for MullionSettings {
    fn default() -> Self {
        Self::local(PaneFocusBehavior::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_is_the_default_focus_behavior() {
        let settings = MullionSettings::default();
        assert_eq!(
            settings.focus_behavior_untracked(),
            PaneFocusBehavior::Click
        );
    }

    #[test]
    fn local_setting_updates_every_clone() {
        let settings = MullionSettings::local(PaneFocusBehavior::Click);
        let settings_page = settings.clone();

        settings_page.set_focus_behavior(PaneFocusBehavior::Hover);

        assert_eq!(
            settings.focus_behavior_untracked(),
            PaneFocusBehavior::Hover
        );
    }

    #[test]
    fn controlled_setting_writes_to_the_host_store() {
        let host_value = ArcRwSignal::new(PaneFocusBehavior::Click);
        let host_writer = host_value.clone();
        let settings = MullionSettings::controlled(host_value.clone(), move |next| {
            host_writer.set(next);
        });

        settings.set_focus_behavior(PaneFocusBehavior::Hover);

        assert_eq!(host_value.get_untracked(), PaneFocusBehavior::Hover);
        assert_eq!(
            settings.focus_behavior_untracked(),
            PaneFocusBehavior::Hover
        );
    }

    #[test]
    fn descriptor_has_stable_host_integration_metadata() {
        let setting = MullionSettings::default().focus_behavior_setting();

        assert_eq!(setting.id(), "mullion.focus_behavior");
        assert_eq!(setting.label(), "Pane focus behavior");
        assert_eq!(setting.options().len(), 2);
        assert_eq!(setting.options()[0].value(), &PaneFocusBehavior::Click);
    }
}
