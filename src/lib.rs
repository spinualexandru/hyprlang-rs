//! # Hyprlang-rs
//!
//! A Rust implementation of the Hyprlang configuration language parser.
//!
//! Hyprlang is a highly efficient, user-friendly configuration language designed for Linux applications.
//! This crate provides a complete implementation of the Hyprlang specification with idiomatic Rust APIs.
//!
//! ## Features
//!
//! - **Multiple data types**: Int, Float, String, Vec2, Color, and custom types
//! - **Variables**: User-defined and environment variables with recursive expansion
//! - **Expressions**: Mathematical expressions with arithmetic operations
//! - **Nested categories**: Hierarchical configuration structure
//! - **Special categories**: Key-based, static, and anonymous categories
//! - **Custom handlers**: Extensible keyword handlers with flag support
//! - **Comment directives**: Conditional parsing and error control
//! - **Multiline values**: Line continuation support
//! - **Source directives**: Include external configuration files
//! - **Dynamic parsing**: Parse and update configuration at runtime
//! - **Mutation & Serialization** (optional): Modify config values and save back to files
//!
//! ## Optional Features
//!
//! ### `mutation` Feature
//!
//! Enable the `mutation` feature to unlock configuration modification and serialization capabilities:
//!
//! ```toml
//! [dependencies]
//! hyprlang = { version = "0.3.0", features = ["mutation"] }
//! ```
//!
//! This provides:
//! - **Value mutations**: [`Config::set_int`], [`Config::set_float`], [`Config::set_string`], [`Config::remove`]
//! - **Variable mutations**: [`Config::set_variable`], [`Config::get_variable_mut`], [`Config::remove_variable`]
//! - **Handler mutations**: [`Config::add_handler_call`], [`Config::remove_handler_call`]
//! - **Category mutations**: [`Config::get_special_category_mut`], [`Config::remove_special_category_instance`]
//! - **Serialization**: [`Config::serialize`], [`Config::save`], [`Config::save_as`]
//!
//! See the mutation API documentation on [`MutableVariable`] and [`MutableCategoryInstance`] for detailed examples.
//!
//! ### `hyprland` Feature
//!
//! The `hyprland` feature provides a high-level API with pre-configured Hyprland handlers and typed accessors.
//! See the [`Hyprland`] struct documentation for details.
//!
//! ## Example
//!
//! ```rust
//! use hyprlang::{Config, ConfigValue};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new configuration
//! let mut config = Config::new();
//!
//! // Parse a configuration string
//! config.parse(r#"
//! $SCALE = 2
//! $WIDTH = 800
//!
//! window_width = $WIDTH
//! window_height = 600
//! scale = $SCALE
//! total_width = {{WIDTH * SCALE}}
//!
//! appearance {
//!     color = rgb(255, 255, 255)
//!     position = (100, 200)
//! }
//! "#)?;
//!
//! // Retrieve values
//! assert_eq!(config.get_int("window_width")?, 800);
//! assert_eq!(config.get_int("scale")?, 2);
//! assert_eq!(config.get_int("total_width")?, 1600);
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Custom Handlers
//!
//! ```rust
//! use hyprlang::{Config, HandlerContext};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = Config::new();
//!
//! // Register a custom handler
//! config.register_handler_fn("exec", |ctx| {
//!     println!("Executing: {}", ctx.value);
//!     Ok(())
//! });
//!
//! config.parse("exec = /usr/bin/app")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Special Categories
//!
//! ```rust
//! use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = Config::new();
//!
//! // Register a special category
//! config.register_special_category(
//!     SpecialCategoryDescriptor::keyed("device", "name")
//! );
//! config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
//! config.register_special_category_value("device", "repeat_rate", ConfigValue::Int(0));
//!
//! config.parse(r#"
//! device[mouse] {
//!     sensitivity = 2.5
//! }
//!
//! device[keyboard] {
//!     repeat_rate = 50
//! }
//! "#)?;
//!
//! let keys = config.list_special_category_keys("device");
//! assert!(keys.contains(&"mouse".to_string()));
//! # Ok(())
//! # }
//! ```

// Module declarations
mod config;
mod error;
mod escaping;
mod expressions;
mod features;
mod handlers;
mod parser;
mod special_categories;
mod types;
mod variables;

// Feature-gated modules
#[cfg(feature = "hyprland")]
mod hyprland;

#[cfg(feature = "mutation")]
mod document;

#[cfg(feature = "mutation")]
mod mutation;

// Public API exports
pub use config::{Config, ConfigOptions};
pub use error::{ConfigError, ParseResult};
pub use types::{Color, ConfigValue, ConfigValueEntry, CustomValueType, Vec2};

// Re-export submodules for advanced usage
pub use escaping::{process_escapes, restore_escaped_braces};
pub use expressions::ExpressionEvaluator;
pub use handlers::{FunctionHandler, Handler, HandlerContext, HandlerManager, HandlerScope};
pub use special_categories::{
    SpecialCategoryDescriptor, SpecialCategoryInstance, SpecialCategoryManager, SpecialCategoryType,
};
pub use variables::VariableManager;

// Feature-gated exports
#[cfg(feature = "hyprland")]
pub use hyprland::{Hyprland, RuleInstance};

#[cfg(feature = "mutation")]
pub use document::{ConfigDocument, DocumentNode, NodeLocation, NodeType};

#[cfg(feature = "mutation")]
pub use mutation::{MutableCategoryInstance, MutableVariable};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut config = Config::new();
        config.parse("test = 123").unwrap();
        assert_eq!(config.get_int("test").unwrap(), 123);
    }

    #[test]
    fn test_variables() {
        let mut config = Config::new();
        config
            .parse(
                r#"
            $VAR = 42
            value = $VAR
        "#,
            )
            .unwrap();
        assert_eq!(config.get_int("value").unwrap(), 42);
    }

    #[test]
    fn test_expressions() {
        let mut config = Config::new();
        config
            .parse(
                r#"
            $A = 10
            $B = 5
            result = {{A + B}}
        "#,
            )
            .unwrap();
        assert_eq!(config.get_int("result").unwrap(), 15);
    }

    #[test]
    fn test_nested_categories() {
        let mut config = Config::new();
        config
            .parse(
                r#"
            category {
                value = 100
            }
        "#,
            )
            .unwrap();
        assert_eq!(config.get_int("category:value").unwrap(), 100);
    }

    #[test]
    fn test_colors() {
        let mut config = Config::new();
        config.parse("color = rgb(255, 128, 64)").unwrap();
        let color = config.get_color("color").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_vec2() {
        let mut config = Config::new();
        config.parse("pos = (100, 200)").unwrap();
        let pos = config.get_vec2("pos").unwrap();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    // ========== ConfigOptions ==========

    #[test]
    fn test_verify_only_no_values_stored() {
        let mut config = Config::with_options(ConfigOptions {
            verify_only: true,
            ..Default::default()
        });
        config.parse("key = 42").unwrap();
        assert!(!config.contains("key"));
    }

    #[test]
    fn test_verify_only_detects_errors() {
        let mut config = Config::with_options(ConfigOptions {
            verify_only: true,
            ..Default::default()
        });
        config.parse("key = value").unwrap();
        assert!(config.parse("= value").is_err());
    }

    #[test]
    fn test_verify_only_variables_not_stored() {
        let mut config = Config::with_options(ConfigOptions {
            verify_only: true,
            ..Default::default()
        });
        config.parse("$VAR = hello\nkey = $VAR").unwrap();
        assert!(config.get_variable("VAR").is_none());
        assert!(!config.contains("key"));
    }

    #[test]
    fn test_allow_missing_config() {
        let mut config = Config::with_options(ConfigOptions {
            allow_missing_config: true,
            ..Default::default()
        });
        let result = config.parse_file("/nonexistent/path/to/config.conf");
        assert!(result.is_ok());
    }

    #[test]
    fn test_allow_missing_config_false() {
        let mut config = Config::with_options(ConfigOptions {
            allow_missing_config: false,
            ..Default::default()
        });
        let result = config.parse_file("/nonexistent/path/to/config.conf");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_is_stream() {
        let mut config = Config::with_options(ConfigOptions {
            path_is_stream: true,
            ..Default::default()
        });
        config.parse_file("key = 42").unwrap();
        assert_eq!(config.get_int("key").unwrap(), 42);
    }

    #[test]
    fn test_path_is_stream_with_categories() {
        let mut config = Config::with_options(ConfigOptions {
            path_is_stream: true,
            ..Default::default()
        });
        config
            .parse_file("general {\n  border_size = 3\n}")
            .unwrap();
        assert_eq!(config.get_int("general:border_size").unwrap(), 3);
    }

    // ========== change_root_path ==========

    #[test]
    fn test_change_root_path() {
        let mut config = Config::new();
        config.change_root_path(std::path::Path::new("/tmp/new/path"));
        assert!(config.parse("key = value").is_ok());
    }

    // ========== special_category_exists_for_key ==========

    #[test]
    fn test_special_category_exists_for_key() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
        config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
        config
            .parse("device[mouse] {\n  sensitivity = 2.5\n}")
            .unwrap();

        assert!(config.special_category_exists_for_key("device", "mouse"));
        assert!(!config.special_category_exists_for_key("device", "keyboard"));
        assert!(!config.special_category_exists_for_key("nonexistent", "key"));
    }

    #[test]
    fn test_special_category_exists_multiple_instances() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
        config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
        config.register_special_category_value("device", "repeat_rate", ConfigValue::Int(0));
        config
            .parse(
                r#"
        device[mouse] {
            sensitivity = 1.0
        }
        device[keyboard] {
            repeat_rate = 50
        }
    "#,
            )
            .unwrap();

        assert!(config.special_category_exists_for_key("device", "mouse"));
        assert!(config.special_category_exists_for_key("device", "keyboard"));
        assert!(!config.special_category_exists_for_key("device", "touchpad"));
    }

    #[test]
    fn test_special_category_bracketless_key() {
        // Key resolved from inner `name = ...` assignment instead of brackets
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
        config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
        config
            .parse(
                r#"
        device {
            name = mouse
            sensitivity = 2.5
        }
    "#,
            )
            .unwrap();

        assert!(config.special_category_exists_for_key("device", "mouse"));
        let instance = config.get_special_category("device", "mouse").unwrap();
        assert_eq!(instance.get("sensitivity").unwrap().as_float().unwrap(), 2.5);
    }

    #[test]
    fn test_special_category_bracketless_requires_key_first() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
        config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));

        let result = config.parse(
            r#"
        device {
            sensitivity = 2.5
            name = mouse
        }
    "#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_special_category_bracketless_multiple() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
        config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
        config.register_special_category_value("device", "repeat_rate", ConfigValue::Int(0));
        config
            .parse(
                r#"
        device {
            name = mouse
            sensitivity = 1.0
        }
        device {
            name = keyboard
            repeat_rate = 50
        }
    "#,
            )
            .unwrap();

        assert!(config.special_category_exists_for_key("device", "mouse"));
        assert!(config.special_category_exists_for_key("device", "keyboard"));
    }

    #[test]
    fn test_special_category_state_cleanup() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
        config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
        config
            .parse(
                r#"
        device[mouse] {
            sensitivity = 2.0
        }

        regular_key = after_special_category
    "#,
            )
            .unwrap();

        assert_eq!(
            config.get_string("regular_key").unwrap(),
            "after_special_category"
        );
    }

    #[test]
    fn test_special_category_ignore_missing() {
        let mut config = Config::new();
        config.register_special_category(
            SpecialCategoryDescriptor::keyed("device", "name").with_ignore_missing(),
        );

        let keys = config.list_special_category_keys("device");
        assert!(keys.is_empty());
    }

    // ========== unregister_handler ==========

    #[test]
    fn test_unregister_handler() {
        let mut config = Config::new();
        config.register_handler_fn("exec", |_ctx| Ok(()));

        config.parse("exec = test").unwrap();

        assert!(config.unregister_handler("exec"));

        config.parse("exec = test2").unwrap();
        assert_eq!(config.get_string("exec").unwrap(), "test2");
    }

    #[test]
    fn test_unregister_handler_not_found() {
        let mut config = Config::new();
        assert!(!config.unregister_handler("nonexistent"));
    }

    #[test]
    fn test_unregister_category_handler() {
        let mut config = Config::new();
        config.register_category_handler_fn("animations", "bezier", |_ctx| Ok(()));

        assert!(config.unregister_category_handler("animations", "bezier"));
        assert!(!config.unregister_category_handler("animations", "bezier"));
    }

    // ========== Handler Scoping ==========

    #[test]
    fn test_global_handler_in_category() {
        let mut config = Config::new();
        config.register_handler_fn("bind", |_ctx| Ok(()));

        config
            .parse(
                r#"
        bind = SUPER, Q, exec, terminal
        general {
            bind = SUPER, C, killactive
        }
    "#,
            )
            .unwrap();

        let calls = config.get_handler_calls("bind").unwrap();
        assert_eq!(calls.len(), 1);

        let cat_calls = config.get_handler_calls("general:bind").unwrap();
        assert_eq!(cat_calls.len(), 1);
    }

    #[test]
    fn test_category_handler_only_fires_in_scope() {
        let mut config = Config::new();
        config.register_category_handler_fn("animations", "bezier", |_ctx| Ok(()));

        config
            .parse(
                r#"
        animations {
            bezier = myBezier, 0.05, 0.9, 0.1, 1.05
        }
    "#,
            )
            .unwrap();

        let calls = config.get_handler_calls("animations:bezier").unwrap();
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn test_flagged_handler_prefix_invocation() {
        let mut config = Config::new();
        let seen_flags = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let seen_flags_ref = std::rc::Rc::clone(&seen_flags);

        config.register_handler(
            "flags",
            FunctionHandler::with_flags("flags", move |ctx| {
                seen_flags_ref.borrow_mut().push((
                    ctx.keyword.clone(),
                    ctx.flags.clone(),
                    ctx.value.clone(),
                ));
                Ok(())
            }),
        );

        config.parse("flagsabc = test").unwrap();

        let seen = seen_flags.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, "flagsabc");
        assert_eq!(seen[0].1.as_deref(), Some("abc"));
        assert_eq!(seen[0].2, "test");
        assert!(config.get("flagsabc").is_err());
        assert_eq!(config.get_handler_calls("flags").unwrap(), &vec!["test".to_string()]);
    }

    // ========== Dynamic Variable Propagation ==========

    #[test]
    fn test_dynamic_variable_propagation() {
        let mut config = Config::new();
        config
            .parse(
                r#"
        $GAPS = 10
        gaps_in = $GAPS
        gaps_out = $GAPS
    "#,
            )
            .unwrap();

        assert_eq!(config.get_int("gaps_in").unwrap(), 10);
        assert_eq!(config.get_int("gaps_out").unwrap(), 10);

        config.parse_dynamic("$GAPS = 20").unwrap();

        assert_eq!(config.get_int("gaps_in").unwrap(), 20);
        assert_eq!(config.get_int("gaps_out").unwrap(), 20);
    }

    #[test]
    fn test_dynamic_variable_propagation_with_expression() {
        let mut config = Config::new();
        config
            .parse(
                r#"
        $BASE = 10
        doubled = {{BASE * 2}}
    "#,
            )
            .unwrap();

        assert_eq!(config.get_int("doubled").unwrap(), 20);

        config.parse_dynamic("$BASE = 15").unwrap();
        assert_eq!(config.get_variable("BASE").unwrap(), "15");
    }

    #[test]
    fn test_dynamic_variable_chain_propagation() {
        let mut config = Config::new();
        config
            .parse(
                r#"
        $A = hello
        $B = $A world
        greeting = $B
    "#,
            )
            .unwrap();

        assert_eq!(config.get_string("greeting").unwrap(), "hello world");

        config.parse_dynamic("$A = hi").unwrap();

        assert_eq!(config.get_variable("B").unwrap(), "hi world");
    }

    #[test]
    fn test_dynamic_variable_replay_does_not_duplicate_handler_side_effects() {
        let mut config = Config::new();
        let seen_values = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let seen_values_ref = std::rc::Rc::clone(&seen_values);

        config.register_handler_fn("exec", move |ctx| {
            seen_values_ref.borrow_mut().push(ctx.value.clone());
            Ok(())
        });

        config
            .parse(
                r#"
        $A = 1
        exec = $A
    "#,
            )
            .unwrap();
        config.parse_dynamic("$A = 2").unwrap();
        config.parse_dynamic("$A = 3").unwrap();

        assert_eq!(
            seen_values.borrow().as_slice(),
            &["1".to_string(), "2".to_string(), "3".to_string()]
        );
        assert_eq!(config.get_handler_calls("exec").unwrap().len(), 3);
    }

    #[test]
    fn test_dynamic_no_propagation_for_non_variable_lines() {
        let mut config = Config::new();
        config
            .parse(
                r#"
        $VAR = 10
        static_key = 42
        dynamic_key = $VAR
    "#,
            )
            .unwrap();

        assert_eq!(config.get_int("static_key").unwrap(), 42);
        assert_eq!(config.get_int("dynamic_key").unwrap(), 10);

        config.parse_dynamic("$VAR = 20").unwrap();

        assert_eq!(config.get_int("static_key").unwrap(), 42);
        assert_eq!(config.get_int("dynamic_key").unwrap(), 20);
    }

    // ========== parse_dynamic_kv ==========

    #[test]
    fn test_parse_dynamic_kv() {
        let mut config = Config::new();
        config.parse("key = 10").unwrap();
        assert_eq!(config.get_int("key").unwrap(), 10);

        config.parse_dynamic_kv("key", "42").unwrap();
        assert_eq!(config.get_int("key").unwrap(), 42);
    }

    #[test]
    fn test_parse_dynamic_kv_category_path() {
        let mut config = Config::new();
        config.parse("cat {\n  val = 1\n}").unwrap();
        assert_eq!(config.get_int("cat:val").unwrap(), 1);

        config.parse_dynamic_kv("cat:val", "99").unwrap();
        assert_eq!(config.get_int("cat:val").unwrap(), 99);
    }

    // ========== Dynamic special category bracket syntax ==========

    #[test]
    fn test_dynamic_special_category_bracket() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("special", "name"));
        config.register_special_category_value("special", "value", ConfigValue::Int(0));
        config
            .parse("special[a] {\n  value = 1\n}")
            .unwrap();

        assert_eq!(
            config
                .get_special_category("special", "a")
                .unwrap()
                .get("value")
                .unwrap()
                .as_int()
                .unwrap(),
            1
        );

        // Update via dynamic bracket syntax
        config.parse_dynamic("special[a]:value = 69").unwrap();
        assert_eq!(
            config
                .get_special_category("special", "a")
                .unwrap()
                .get("value")
                .unwrap()
                .as_int()
                .unwrap(),
            69
        );
    }

    #[test]
    fn test_dynamic_special_category_bracket_creates_instance() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("special", "name"));
        config.register_special_category_value("special", "value", ConfigValue::Int(0));

        // Dynamic bracket syntax creates instance if it doesn't exist
        config.parse_dynamic("special[new]:value = 42").unwrap();
        assert!(config.special_category_exists_for_key("special", "new"));
        assert_eq!(
            config
                .get_special_category("special", "new")
                .unwrap()
                .get("value")
                .unwrap()
                .as_int()
                .unwrap(),
            42
        );
    }

    #[test]
    fn test_dynamic_special_category_rejects_unknown_property() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::keyed("special", "name"));
        config.register_special_category_value("special", "value", ConfigValue::Int(0));

        assert!(config.parse_dynamic("special[a]:typo = 1").is_err());
        assert!(!config.special_category_exists_for_key("special", "a"));

        config.register_special_category(SpecialCategoryDescriptor::static_category(
            "specialGeneric:two",
        ));
        config.register_special_category_value(
            "specialGeneric:two",
            "value",
            ConfigValue::Int(0),
        );

        assert!(config.parse_dynamic("specialGeneric:two:typo = 1").is_err());
        assert!(
            config
                .get_special_category("specialGeneric:two", "static")
                .unwrap()
                .get("typo")
                .is_none()
        );
    }

    // ========== Colon-scoped special categories ==========

    #[test]
    fn test_colon_scoped_special_category() {
        let mut config = Config::new();
        config.register_special_category(
            SpecialCategoryDescriptor::static_category("specialGeneric:one"),
        );
        config.register_special_category_value(
            "specialGeneric:one",
            "value",
            ConfigValue::Int(0),
        );

        config
            .parse(
                r#"
        specialGeneric {
            one {
                value = 1
            }
        }
    "#,
            )
            .unwrap();

        let instance = config.get_special_category("specialGeneric:one", "static").unwrap();
        assert_eq!(instance.get("value").unwrap().as_int().unwrap(), 1);
    }

    #[test]
    fn test_colon_scoped_special_category_flat_syntax() {
        let mut config = Config::new();
        config.register_special_category(
            SpecialCategoryDescriptor::static_category("specialGeneric:two"),
        );
        config.register_special_category_value(
            "specialGeneric:two",
            "hola",
            ConfigValue::String(String::new()),
        );

        // Direct colon-separated key assignment at root level
        config.parse("specialGeneric:two:hola = rose").unwrap();

        let instance = config.get_special_category("specialGeneric:two", "static").unwrap();
        assert_eq!(
            instance.get("hola").unwrap().as_string().unwrap(),
            "rose"
        );
    }

    #[test]
    fn test_colon_scoped_multiple_categories() {
        let mut config = Config::new();
        config.register_special_category(
            SpecialCategoryDescriptor::static_category("generic:one"),
        );
        config.register_special_category_value("generic:one", "val", ConfigValue::Int(0));
        config.register_special_category(
            SpecialCategoryDescriptor::static_category("generic:two"),
        );
        config.register_special_category_value("generic:two", "val", ConfigValue::Int(0));

        config
            .parse(
                r#"
        generic {
            one {
                val = 10
            }
            two {
                val = 20
            }
        }
    "#,
            )
            .unwrap();

        let one = config.get_special_category("generic:one", "static").unwrap();
        assert_eq!(one.get("val").unwrap().as_int().unwrap(), 10);

        let two = config.get_special_category("generic:two", "static").unwrap();
        assert_eq!(two.get("val").unwrap().as_int().unwrap(), 20);
    }

    #[test]
    fn test_same_keyword_special_category_prefers_nested_category_match() {
        let mut config = Config::new();
        let handler_values = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let handler_values_ref = std::rc::Rc::clone(&handler_values);

        config.register_handler_fn("sameKeywordSpecialCat", move |ctx| {
            handler_values_ref.borrow_mut().push(ctx.value.clone());
            Ok(())
        });

        config.register_special_category(
            SpecialCategoryDescriptor::static_category("sameKeywordSpecialCat")
                .with_ignore_missing(),
        );
        config.register_special_category(
            SpecialCategoryDescriptor::static_category("sameKeywordSpecialCat:one")
                .with_ignore_missing(),
        );
        config.register_special_category_value(
            "sameKeywordSpecialCat:one",
            "some_size",
            ConfigValue::Int(10),
        );
        config.register_special_category_value(
            "sameKeywordSpecialCat:one",
            "some_radius",
            ConfigValue::Float(0.0),
        );
        config.register_special_category(
            SpecialCategoryDescriptor::static_category("sameKeywordSpecialCat:two")
                .with_ignore_missing(),
        );
        config.register_special_category_value(
            "sameKeywordSpecialCat:two",
            "hola",
            ConfigValue::String(String::new()),
        );

        config
            .parse(
                r#"
        sameKeywordSpecialCat = pablo
        sameKeywordSpecialCat:two:hola = rose
        sameKeywordSpecialCat {
            one {
                some_size = 44
                some_radius = 7.6
            }
        }
    "#,
            )
            .unwrap();

        assert_eq!(handler_values.borrow().as_slice(), &["pablo".to_string()]);
        assert_eq!(
            config
                .get_special_category("sameKeywordSpecialCat:one", "static")
                .unwrap()
                .get("some_size")
                .unwrap()
                .as_int()
                .unwrap(),
            44
        );
        assert_eq!(
            config
                .get_special_category("sameKeywordSpecialCat:one", "static")
                .unwrap()
                .get("some_radius")
                .unwrap()
                .as_float()
                .unwrap(),
            7.6
        );
        assert_eq!(
            config
                .get_special_category("sameKeywordSpecialCat:two", "static")
                .unwrap()
                .get("hola")
                .unwrap()
                .as_string()
                .unwrap(),
            "rose"
        );
    }

    // ========== Anonymous nested special categories ==========

    #[test]
    fn test_anonymous_nested_flat_keys() {
        let mut config = Config::new();
        config
            .register_special_category(SpecialCategoryDescriptor::anonymous("anon_nested"));
        config.register_special_category_value(
            "anon_nested",
            "nested:value1",
            ConfigValue::Int(0),
        );
        config.register_special_category_value(
            "anon_nested",
            "nested:value2",
            ConfigValue::Int(0),
        );

        config
            .parse(
                r#"
        anon_nested {
            nested:value1 = 1
            nested:value2 = 2
        }
    "#,
            )
            .unwrap();

        let keys = config.list_special_category_keys("anon_nested");
        assert_eq!(keys.len(), 1);
        let instance = config
            .get_special_category("anon_nested", &keys[0])
            .unwrap();
        assert_eq!(instance.get("nested:value1").unwrap().as_int().unwrap(), 1);
        assert_eq!(instance.get("nested:value2").unwrap().as_int().unwrap(), 2);
    }

    #[test]
    fn test_anonymous_nested_block_syntax() {
        let mut config = Config::new();
        config
            .register_special_category(SpecialCategoryDescriptor::anonymous("anon_nested"));
        config.register_special_category_value(
            "anon_nested",
            "nested:value1",
            ConfigValue::Int(0),
        );
        config.register_special_category_value(
            "anon_nested",
            "nested:value2",
            ConfigValue::Int(0),
        );

        config
            .parse(
                r#"
        anon_nested {
            nested {
                value1 = 3
                value2 = 4
            }
        }
    "#,
            )
            .unwrap();

        let keys = config.list_special_category_keys("anon_nested");
        assert_eq!(keys.len(), 1);
        let instance = config
            .get_special_category("anon_nested", &keys[0])
            .unwrap();
        assert_eq!(instance.get("nested:value1").unwrap().as_int().unwrap(), 3);
        assert_eq!(instance.get("nested:value2").unwrap().as_int().unwrap(), 4);
    }

    #[test]
    fn test_anonymous_special_category_accepts_explicit_dynamic_keys() {
        let mut config = Config::new();
        config.register_special_category(SpecialCategoryDescriptor::anonymous(
            "specialAnonymousNested",
        ));
        config.register_special_category_value(
            "specialAnonymousNested",
            "nested:value1",
            ConfigValue::Int(0),
        );
        config.register_special_category_value(
            "specialAnonymousNested",
            "nested:value2",
            ConfigValue::Int(0),
        );

        config
            .parse(
                r#"
        specialAnonymousNested {
            nested:value1 = 1
            nested:value2 = 2
        }
        specialAnonymousNested {
            nested:value1 = 3
            nested:value2 = 4
        }
    "#,
            )
            .unwrap();

        let keys = config.list_special_category_keys("specialAnonymousNested");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"1".to_string()));
        assert!(keys.contains(&"2".to_string()));

        config
            .parse_dynamic("specialAnonymousNested[c]:nested:value1 = 5")
            .unwrap();
        config
            .parse_dynamic("specialAnonymousNested[c]:nested:value2 = 6")
            .unwrap();

        let instance = config
            .get_special_category("specialAnonymousNested", "c")
            .unwrap();
        assert_eq!(instance.get("nested:value1").unwrap().as_int().unwrap(), 5);
        assert_eq!(instance.get("nested:value2").unwrap().as_int().unwrap(), 6);
    }

    #[test]
    fn test_change_root_path_uses_parent_of_config_file() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir =
            std::env::temp_dir().join(format!("hyprlang-change-root-{unique}-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let root_config = temp_dir.join("main.conf");
        let included = temp_dir.join("included.conf");
        std::fs::write(&included, "included = 42\n").unwrap();

        let mut config = Config::new();
        config.change_root_path(&root_config);
        config.parse("source = included.conf").unwrap();

        assert_eq!(config.get_int("included").unwrap(), 42);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
