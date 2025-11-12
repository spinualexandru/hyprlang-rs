//! Hyprland-specific configuration wrapper
//!
//! This module provides a high-level interface for working with Hyprland configurations.
//! It automatically registers all Hyprland handlers, special categories, and provides
//! typed access to common configuration options.
//!
//! # Overview
//!
//! The [`Hyprland`] struct wraps the low-level [`Config`] API with Hyprland-specific
//! conveniences:
//!
//! - **Automatic Handler Registration**: All Hyprland handlers (bind, monitor, env, etc.)
//!   are pre-registered when you create a [`Hyprland`] instance
//! - **Typed Accessor Methods**: Instead of string-based key access, use typed methods
//!   like [`general_border_size()`](Hyprland::general_border_size) that return the correct type
//! - **Handler Arrays**: Access all binds, windowrules, etc. as arrays with methods like
//!   [`all_binds()`](Hyprland::all_binds)
//! - **Special Categories**: Device and monitor categories are pre-configured
//!
//! # When to Use This Module
//!
//! Use this module when:
//! - You're parsing Hyprland configuration files
//! - You want typed, convenient access to common config values
//! - You're building tools for Hyprland users (config editors, validators, etc.)
//!
//! Use the low-level [`Config`] API when:
//! - You're implementing a different config language
//! - You need full control over handler registration
//! - You want minimal dependencies
//!
//! # Quick Start
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! use hyprlang::Hyprland;
//!
//! // Create instance - handlers are automatically registered
//! let mut hypr = Hyprland::new();
//!
//! // Parse configuration
//! hypr.parse(r#"
//!     general {
//!         border_size = 2
//!         gaps_in = 5
//!         col.active_border = rgba(33ccffee)
//!     }
//!
//!     bind = SUPER, Q, exec, kitty
//!     bind = SUPER, C, killactive
//! "#).unwrap();
//!
//! // Access with typed methods
//! let border = hypr.general_border_size().unwrap();
//! let color = hypr.general_active_border_color().unwrap();
//!
//! // Get all bindings as an array
//! let binds = hypr.all_binds();
//! assert_eq!(binds.len(), 2);
//! # }
//! ```
//!
//! # Configuration Categories
//!
//! ## General Settings
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! # use hyprlang::Hyprland;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut hypr = Hyprland::new();
//! # hypr.parse(r#"
//! # general {
//! #     border_size = 2
//! #     gaps_in = 5
//! #     gaps_out = 20
//! #     layout = dwindle
//! #     allow_tearing = false
//! #     col.active_border = rgba(33ccffee)
//! #     col.inactive_border = rgba(595959aa)
//! # }
//! # "#)?;
//! // Access general settings
//! let border_size = hypr.general_border_size()?;
//! let gaps_in = hypr.general_gaps_in()?;
//! let layout = hypr.general_layout()?;
//! let tearing = hypr.general_allow_tearing()?;
//!
//! // Access colors
//! let active = hypr.general_active_border_color()?;
//! let inactive = hypr.general_inactive_border_color()?;
//! # Ok(())
//! # }
//! # example().unwrap();
//! # }
//! ```
//!
//! ## Decoration Settings
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! # use hyprlang::Hyprland;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut hypr = Hyprland::new();
//! # hypr.parse(r#"
//! # decoration {
//! #     rounding = 10
//! #     active_opacity = 1.0
//! #     inactive_opacity = 0.9
//! #     blur {
//! #         enabled = true
//! #         size = 3
//! #         passes = 1
//! #     }
//! # }
//! # "#)?;
//! let rounding = hypr.decoration_rounding()?;
//! let active_opacity = hypr.decoration_active_opacity()?;
//! let blur_enabled = hypr.decoration_blur_enabled()?;
//! let blur_size = hypr.decoration_blur_size()?;
//! # Ok(())
//! # }
//! # example().unwrap();
//! # }
//! ```
//!
//! ## Animation Settings
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! # use hyprlang::Hyprland;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let mut hypr = Hyprland::new();
//! # hypr.parse(r#"
//! # animations {
//! #     enabled = true
//! #     animation = windows, 1, 4, default
//! #     animation = fade, 1, 3, quick
//! #     bezier = easeOut, 0.23, 1, 0.32, 1
//! # }
//! # "#)?;
//! if hypr.animations_enabled()? {
//!     // Get all animation definitions
//!     for anim in hypr.all_animations() {
//!         println!("Animation: {}", anim);
//!     }
//!
//!     // Get all bezier curves
//!     for bezier in hypr.all_beziers() {
//!         println!("Bezier: {}", bezier);
//!     }
//! }
//! # Ok(())
//! # }
//! # example().unwrap();
//! # }
//! ```
//!
//! ## Handler Arrays
//!
//! All Hyprland handlers (bind, windowrule, etc.) are collected into arrays:
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! # use hyprlang::Hyprland;
//! # let mut hypr = Hyprland::new();
//! # hypr.parse(r#"
//! # bind = SUPER, Q, exec, kitty
//! # bind = SUPER, C, killactive
//! # windowrule = float, ^(kitty)$
//! # monitor = ,preferred,auto,1
//! # env = XCURSOR_SIZE,24
//! # exec-once = waybar
//! # "#).unwrap();
//! // Get all keybindings
//! let binds = hypr.all_binds();
//! for bind in binds {
//!     println!("Bind: {}", bind);
//! }
//!
//! // Get all window rules
//! let rules = hypr.all_windowrules();
//!
//! // Get all monitors
//! let monitors = hypr.all_monitors();
//!
//! // Get all environment variables
//! let envs = hypr.all_env();
//!
//! // Get all exec-once commands
//! let execs = hypr.all_exec_once();
//! # }
//! ```
//!
//! ## Variables
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! # use hyprlang::Hyprland;
//! # let mut hypr = Hyprland::new();
//! # hypr.parse(r#"
//! # $terminal = kitty
//! # $mod = SUPER
//! # "#).unwrap();
//! // Get all variables
//! let vars = hypr.variables();
//! for (name, value) in vars {
//!     println!("${} = {}", name, value);
//! }
//!
//! // Get specific variable
//! if let Some(terminal) = hypr.get_variable("terminal") {
//!     println!("Terminal: {}", terminal);
//! }
//! # }
//! ```
//!
//! # Pre-Registered Handlers
//!
//! The following handlers are automatically registered:
//!
//! **Root-level handlers:**
//! - `monitor` - Monitor configuration
//! - `env` - Environment variables
//! - `bind`, `bindm`, `bindel`, `bindl`, `bindr`, `binde`, `bindn` - Keybindings
//! - `windowrule`, `windowrulev2` - Window rules
//! - `layerrule` - Layer rules
//! - `workspace` - Workspace configuration
//! - `exec`, `exec-once` - Commands
//! - `source` - File inclusion
//! - `blurls` - Blur layer surface
//! - `plugin` - Plugin loading
//!
//! **Category-specific handlers:**
//! - `animations:animation` - Animation definitions
//! - `animations:bezier` - Bezier curve definitions
//!
//! **Special categories:**
//! - `device[name]` - Per-device input configuration (keyed)
//! - `monitor[name]` - Per-monitor configuration (keyed)
//!
//! # Direct Config Access
//!
//! If you need access to the underlying [`Config`] API:
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! # use hyprlang::Hyprland;
//! # let mut hypr = Hyprland::new();
//! // Immutable access
//! let config = hypr.config();
//! let value = config.get("custom:key");
//!
//! // Mutable access
//! let config = hypr.config_mut();
//! config.register_handler_fn("custom", |ctx| {
//!     println!("Custom: {}", ctx.value);
//!     Ok(())
//! });
//! # }
//! ```
//!
//! # Examples
//!
//! ## Parse a Hyprland Config File
//!
//! ```rust,no_run
//! # #[cfg(feature = "hyprland")]
//! # {
//! use hyprlang::Hyprland;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut hypr = Hyprland::new();
//! hypr.parse_file(Path::new("~/.config/hypr/hyprland.conf"))?;
//!
//! // Access any setting
//! println!("Border size: {}", hypr.general_border_size()?);
//! println!("Layout: {}", hypr.general_layout()?);
//!
//! // List all keybindings
//! for (i, bind) in hypr.all_binds().iter().enumerate() {
//!     println!("[{}] {}", i + 1, bind);
//! }
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Validate a Hyprland Config
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! use hyprlang::Hyprland;
//!
//! fn validate_config(content: &str) -> Result<(), String> {
//!     let mut hypr = Hyprland::new();
//!
//!     hypr.parse(content).map_err(|e| format!("Parse error: {}", e))?;
//!
//!     // Validate required settings
//!     if hypr.general_layout().is_err() {
//!         return Err("Missing general:layout setting".to_string());
//!     }
//!
//!     // Check for recommended settings
//!     if hypr.all_binds().is_empty() {
//!         eprintln!("Warning: No keybindings defined");
//!     }
//!
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ## Extract Config Values
//!
//! ```rust
//! # #[cfg(feature = "hyprland")]
//! # {
//! use hyprlang::Hyprland;
//!
//! # let mut hypr = Hyprland::new();
//! # hypr.parse(r#"
//! # general {
//! #     border_size = 2
//! #     gaps_in = 5
//! #     gaps_out = 20
//! # }
//! # decoration {
//! #     rounding = 10
//! # }
//! # "#).unwrap();
//! // Extract settings into a struct
//! struct Settings {
//!     border_size: i64,
//!     gaps_in: String,
//!     rounding: i64,
//! }
//!
//! let settings = Settings {
//!     border_size: hypr.general_border_size().unwrap_or(2),
//!     gaps_in: hypr.general_gaps_in().unwrap_or("5".to_string()),
//!     rounding: hypr.decoration_rounding().unwrap_or(0),
//! };
//! # }
//! ```
//!
//! [`Config`]: crate::Config

use crate::config::{Config, ConfigOptions};
use crate::error::ParseResult;
use crate::special_categories::SpecialCategoryDescriptor;
use crate::types::{Color, ConfigValue};
use std::path::Path;

/// High-level wrapper for Hyprland configuration
///
/// This struct automatically registers all Hyprland-specific handlers and provides
/// convenient methods for accessing Hyprland configuration values.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "hyprland")]
/// # {
/// use hyprlang::Hyprland;
/// use std::path::Path;
///
/// let mut hypr = Hyprland::new();
/// hypr.parse_file(Path::new("~/.config/hypr/hyprland.conf")).unwrap();
///
/// // Access config values with typed methods
/// let border_size = hypr.general_border_size().unwrap_or(2);
/// let gaps_in = hypr.general_gaps_in().unwrap_or("5".to_string());
///
/// // Access all binds
/// let binds = hypr.all_binds();
/// for bind in binds {
///     println!("Bind: {}", bind);
/// }
/// # }
/// ```
pub struct Hyprland {
    config: Config,
}

impl Hyprland {
    /// Create a new Hyprland configuration with default options
    pub fn new() -> Self {
        let mut config = Config::new();
        Self::register_all_handlers(&mut config);
        Self::register_all_special_categories(&mut config);
        Self { config }
    }

    /// Create a new Hyprland configuration with custom options
    pub fn with_options(options: ConfigOptions) -> Self {
        let mut config = Config::with_options(options);
        Self::register_all_handlers(&mut config);
        Self::register_all_special_categories(&mut config);
        Self { config }
    }

    /// Get a reference to the underlying Config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a mutable reference to the underlying Config
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Parse a configuration string
    pub fn parse(&mut self, content: &str) -> ParseResult<()> {
        self.config.parse(content)
    }

    /// Parse a configuration file
    pub fn parse_file(&mut self, path: &Path) -> ParseResult<()> {
        self.config.parse_file(path)
    }

    /// Register all Hyprland-specific handlers
    fn register_all_handlers(config: &mut Config) {
        // Root-level handlers
        let root_handlers = [
            "monitor",
            "env",
            "bind",
            "bindm",
            "bindel",
            "bindl",
            "bindr",
            "binde",
            "bindn",
            "windowrule",
            "windowrulev2",
            "layerrule",
            "workspace",
            "exec",
            "exec-once",
            "source",
            "blurls",
            "plugin",
        ];

        for handler in root_handlers {
            config.register_handler_fn(handler, |_ctx| Ok(()));
        }

        // Category-specific handlers
        config.register_category_handler_fn("animations", "animation", |_ctx| Ok(()));
        config.register_category_handler_fn("animations", "bezier", |_ctx| Ok(()));
    }

    /// Register all Hyprland-specific special categories
    fn register_all_special_categories(config: &mut Config) {
        // Device is a keyed category: device[name] { ... }
        config.register_special_category(
            SpecialCategoryDescriptor::keyed("device", "name")
        );

        // Monitor is a keyed category: monitor[name] { ... } (for per-monitor settings)
        config.register_special_category(
            SpecialCategoryDescriptor::keyed("monitor", "name")
        );
    }

    // ==================== General Config ====================

    /// Get general:border_size
    pub fn general_border_size(&self) -> ParseResult<i64> {
        self.config.get_int("general:border_size")
    }

    /// Get general:gaps_in (supports CSS-style: "5" or "5 10 15 20")
    pub fn general_gaps_in(&self) -> ParseResult<String> {
        match self.config.get("general:gaps_in")? {
            ConfigValue::Int(i) => Ok(i.to_string()),
            ConfigValue::String(s) => Ok(s.clone()),
            _ => Ok("5".to_string()),
        }
    }

    /// Get general:gaps_out (supports CSS-style: "20" or "5 10 15 20")
    pub fn general_gaps_out(&self) -> ParseResult<String> {
        match self.config.get("general:gaps_out")? {
            ConfigValue::Int(i) => Ok(i.to_string()),
            ConfigValue::String(s) => Ok(s.clone()),
            _ => Ok("20".to_string()),
        }
    }

    /// Get general:col.active_border
    pub fn general_active_border_color(&self) -> ParseResult<Color> {
        self.config.get_color("general:col.active_border")
    }

    /// Get general:col.inactive_border
    pub fn general_inactive_border_color(&self) -> ParseResult<Color> {
        self.config.get_color("general:col.inactive_border")
    }

    /// Get general:layout
    pub fn general_layout(&self) -> ParseResult<&str> {
        self.config.get_string("general:layout")
    }

    /// Get general:allow_tearing
    pub fn general_allow_tearing(&self) -> ParseResult<bool> {
        match self.config.get("general:allow_tearing")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    // ==================== Decoration Config ====================

    /// Get decoration:rounding
    pub fn decoration_rounding(&self) -> ParseResult<i64> {
        self.config.get_int("decoration:rounding")
    }

    /// Get decoration:active_opacity
    pub fn decoration_active_opacity(&self) -> ParseResult<f64> {
        self.config.get_float("decoration:active_opacity")
    }

    /// Get decoration:inactive_opacity
    pub fn decoration_inactive_opacity(&self) -> ParseResult<f64> {
        self.config.get_float("decoration:inactive_opacity")
    }

    /// Get decoration:blur:enabled
    pub fn decoration_blur_enabled(&self) -> ParseResult<bool> {
        match self.config.get("decoration:blur:enabled")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    /// Get decoration:blur:size
    pub fn decoration_blur_size(&self) -> ParseResult<i64> {
        self.config.get_int("decoration:blur:size")
    }

    /// Get decoration:blur:passes
    pub fn decoration_blur_passes(&self) -> ParseResult<i64> {
        self.config.get_int("decoration:blur:passes")
    }

    // ==================== Animations Config ====================

    /// Get animations:enabled
    pub fn animations_enabled(&self) -> ParseResult<bool> {
        match self.config.get("animations:enabled")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    /// Get all animation definitions
    pub fn all_animations(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("animations:animation")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all bezier curve definitions
    pub fn all_beziers(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("animations:bezier")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    // ==================== Input Config ====================

    /// Get input:kb_layout
    pub fn input_kb_layout(&self) -> ParseResult<&str> {
        self.config.get_string("input:kb_layout")
    }

    /// Get input:follow_mouse
    pub fn input_follow_mouse(&self) -> ParseResult<i64> {
        self.config.get_int("input:follow_mouse")
    }

    /// Get input:sensitivity
    pub fn input_sensitivity(&self) -> ParseResult<f64> {
        self.config.get_float("input:sensitivity")
    }

    /// Get input:touchpad:natural_scroll
    pub fn input_touchpad_natural_scroll(&self) -> ParseResult<bool> {
        match self.config.get("input:touchpad:natural_scroll")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    // ==================== Misc Config ====================

    /// Get misc:disable_hyprland_logo
    pub fn misc_disable_hyprland_logo(&self) -> ParseResult<bool> {
        match self.config.get("misc:disable_hyprland_logo")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    /// Get misc:force_default_wallpaper
    pub fn misc_force_default_wallpaper(&self) -> ParseResult<i64> {
        self.config.get_int("misc:force_default_wallpaper")
    }

    // ==================== Dwindle Layout ====================

    /// Get dwindle:pseudotile
    pub fn dwindle_pseudotile(&self) -> ParseResult<bool> {
        match self.config.get("dwindle:pseudotile")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    /// Get dwindle:preserve_split
    pub fn dwindle_preserve_split(&self) -> ParseResult<bool> {
        match self.config.get("dwindle:preserve_split")? {
            ConfigValue::Int(i) => Ok(*i != 0),
            ConfigValue::String(s) => Ok(s == "true" || s == "yes" || s == "on" || s == "1"),
            _ => Ok(false),
        }
    }

    // ==================== Master Layout ====================

    /// Get master:new_status
    pub fn master_new_status(&self) -> ParseResult<&str> {
        self.config.get_string("master:new_status")
    }

    // ==================== Handler Calls ====================

    /// Get all bind definitions
    pub fn all_binds(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("bind")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all bindm definitions
    pub fn all_bindm(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("bindm")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all bindel definitions
    pub fn all_bindel(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("bindel")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all bindl definitions
    pub fn all_bindl(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("bindl")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all windowrule definitions
    pub fn all_windowrules(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("windowrule")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all windowrulev2 definitions
    pub fn all_windowrulesv2(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("windowrulev2")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all layerrule definitions
    pub fn all_layerrules(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("layerrule")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all workspace definitions
    pub fn all_workspaces(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("workspace")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all monitor definitions
    pub fn all_monitors(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("monitor")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all env definitions
    pub fn all_env(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("env")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all exec-once definitions
    pub fn all_exec_once(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("exec-once")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    /// Get all exec definitions
    pub fn all_exec(&self) -> Vec<&String> {
        self.config
            .get_handler_calls("exec")
            .map(|calls| calls.iter().collect())
            .unwrap_or_default()
    }

    // ==================== Variables ====================

    /// Get all variables defined in the config
    pub fn variables(&self) -> &std::collections::HashMap<String, String> {
        self.config.variables()
    }

    /// Get a specific variable value
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables().get(name)
    }
}

impl Default for Hyprland {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyprland_basic_config() {
        let mut hypr = Hyprland::new();

        hypr.parse(r#"
            general {
                border_size = 2
                gaps_in = 5
                gaps_out = 20
                layout = dwindle
            }
        "#).unwrap();

        assert_eq!(hypr.general_border_size().unwrap(), 2);
        assert_eq!(hypr.general_gaps_in().unwrap(), "5".to_string());
        assert_eq!(hypr.general_gaps_out().unwrap(), "20".to_string());
        assert_eq!(hypr.general_layout().unwrap(), "dwindle");
    }

    #[test]
    fn test_hyprland_binds() {
        let mut hypr = Hyprland::new();

        hypr.parse(r#"
            bind = SUPER, Q, exec, kitty
            bind = SUPER, C, killactive
        "#).unwrap();

        let binds = hypr.all_binds();
        assert_eq!(binds.len(), 2);
        assert_eq!(binds[0], "SUPER, Q, exec, kitty");
        assert_eq!(binds[1], "SUPER, C, killactive");
    }

    #[test]
    fn test_hyprland_animations() {
        let mut hypr = Hyprland::new();

        hypr.parse(r#"
            animations {
                enabled = true
                animation = windows, 1, 4, default
                animation = fade, 1, 3, quick
                bezier = easeOut, 0.23, 1, 0.32, 1
            }
        "#).unwrap();

        assert!(hypr.animations_enabled().unwrap());

        let animations = hypr.all_animations();
        assert_eq!(animations.len(), 2);

        let beziers = hypr.all_beziers();
        assert_eq!(beziers.len(), 1);
    }

    #[test]
    fn test_hyprland_variables() {
        let mut hypr = Hyprland::new();

        hypr.parse(r#"
            $terminal = kitty
            $mod = SUPER
        "#).unwrap();

        let vars = hypr.variables();
        assert_eq!(vars.get("terminal"), Some(&"kitty".to_string()));
        assert_eq!(vars.get("mod"), Some(&"SUPER".to_string()));
    }

    #[test]
    fn test_hyprland_decoration() {
        let mut hypr = Hyprland::new();

        hypr.parse(r#"
            decoration {
                rounding = 10
                active_opacity = 1.0
                inactive_opacity = 0.8

                blur {
                    enabled = true
                    size = 3
                    passes = 1
                }
            }
        "#).unwrap();

        assert_eq!(hypr.decoration_rounding().unwrap(), 10);
        assert_eq!(hypr.decoration_active_opacity().unwrap(), 1.0);
        assert_eq!(hypr.decoration_inactive_opacity().unwrap(), 0.8);
        assert!(hypr.decoration_blur_enabled().unwrap());
        assert_eq!(hypr.decoration_blur_size().unwrap(), 3);
        assert_eq!(hypr.decoration_blur_passes().unwrap(), 1);
    }
}
