/// Global color theme for mullion components.
///
/// Defines CSS custom properties on `:root` that all component styles
/// can reference via `var(--ml-*)`.
#[derive(css_styled::Theme, Clone, Debug)]
pub struct MullionTheme {
    /// Root/page background (darkest).
    #[var("--ml-bg")]
    pub bg: String,
    /// Surface background (panels, panes, activity bar).
    #[var("--ml-surface")]
    pub surface: String,
    /// Border/separator color.
    #[var("--ml-border")]
    pub border: String,
    /// Subtle accent (active tabs, hover backgrounds).
    #[var("--ml-accent")]
    pub accent: String,
    /// Primary text color.
    #[var("--ml-text")]
    pub text: String,
    /// Muted/secondary text color.
    #[var("--ml-text-muted")]
    pub text_muted: String,
    /// Highlight color for interactive elements (hover, focus).
    #[var("--ml-highlight")]
    pub highlight: String,
    /// Drop overlay indicator color.
    #[var("--ml-drop-indicator")]
    pub drop_indicator: String,
}

impl Default for MullionTheme {
    fn default() -> Self {
        MullionTheme {
            bg: "#0e0e0e".into(),
            surface: "#111111".into(),
            border: "#1a1a1a".into(),
            accent: "#222222".into(),
            highlight: "#333333".into(),
            text: "#eeeeee".into(),
            text_muted: "#888888".into(),
            drop_indicator: "rgba(255,255,255,0.06)".into(),
        }
    }
}
