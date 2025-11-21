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
//! hyprlang = { version = "0.2.0", features = ["mutation"] }
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
//! use hyprlang::{Config, SpecialCategoryDescriptor};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut config = Config::new();
//!
//! // Register a special category
//! config.register_special_category(
//!     SpecialCategoryDescriptor::keyed("device", "name")
//! );
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
pub use hyprland::Hyprland;

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
}
