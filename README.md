# hyprlang-rs

A Rust reimplementation of [Hyprlang](https://github.com/hyprwm/hyprlang), the configuration language used by [Hyprland](https://hyprland.org/).

Hyprlang is a powerful configuration language featuring variables, nested categories, expressions, custom handlers, and more. This library provides a complete parser and configuration manager with a clean, idiomatic Rust API.

This project is not endorsed by or affiliated with the Hyprland project/HyprWM Organization.

[![Rust Build](https://github.com/spinualexandru/hyprlang-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/spinualexandru/hyprlang-rs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/hyprlang.svg)](https://crates.io/crates/hyprlang)
[![Docs.rs](https://docs.rs/hyprlang/badge.svg)](https://docs.rs/hyprlang)
[![License](https://img.shields.io/crates/l/hyprlang.svg)](https://img.shields.io/crates/l/hyprlang.svg)
![Crates.io Total Downloads](https://img.shields.io/crates/d/hyprlang)
![Crates.io Size](https://img.shields.io/crates/size/hyprlang)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/spinualexandru)](https://github.com/sponsors/spinualexandru)

## Features

- 🎯 **Complete Hyprlang Implementation** - Full compatibility with the original C++ version
- 🚀 **Fast PEG Parser** - Built with [pest](https://pest.rs/) for efficient parsing
- 🔧 **Type-Safe API** - Strongly-typed configuration values (Int, Float, String, Vec2, Color)
- 📦 **Variable System** - Support for user-defined and environment variables with cycle detection
- 🧮 **Expression Evaluation** - Arithmetic expressions with `{{expr}}` syntax
- 🎨 **Color Support** - Multiple color formats: `rgba()`, `rgb()`, and hex colors
- 📐 **Vec2 Coordinates** - Built-in support for 2D coordinate pairs
- 🔌 **Handler System** - Extensible keyword handlers for custom syntax
- 🏷️ **Special Categories** - Keyed, static, and anonymous category types
- 📄 **Source Directives** - Include external configuration files
- 💬 **Conditional Directives** - `# hyprlang if/endif/noerror` support
- ✅ **Fully Tested** - 44 tests covering all features

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hyprlang = "0.1.4"
```

### Optional Features

#### `hyprland` Feature

Enable the `hyprland` feature to get a high-level `Hyprland` struct with pre-configured handlers and typed access to Hyprland configuration options:

```toml
[dependencies]
hyprlang = { version = "0.1.4", features = ["hyprland"] }
```

This feature provides:
- Automatic registration of all Hyprland handlers (bind, monitor, env, etc.)
- Typed accessor methods for common Hyprland config values
- Convenient methods to access all binds, windowrules, animations, etc.

## Quick Start

```rust
use hyprlang::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();

    // Parse a configuration string
    config.parse(r#"
        general {
            gaps_in = 5
            gaps_out = 20
            border_size = 2
        }
    "#)?;

    // Access values
    let gaps_in = config.get_int("general:gaps_in")?;
    let gaps_out = config.get_int("general:gaps_out")?;

    println!("gaps_in: {}, gaps_out: {}", gaps_in, gaps_out);

    Ok(())
}
```

## Hyprland API (Optional Feature)

The `hyprland` feature provides a high-level, type-safe API specifically designed for working with Hyprland configurations. Instead of manually registering handlers and using string-based key access, you get a convenient `Hyprland` struct with pre-configured handlers and typed accessor methods.

### Why Use the Hyprland Feature?

**Without the Hyprland feature** (using low-level Config API):
```rust
use hyprlang::Config;

let mut config = Config::new();

// Manually register all handlers
config.register_handler_fn("bind", |_| Ok(()));
config.register_handler_fn("monitor", |_| Ok(()));
config.register_handler_fn("windowrule", |_| Ok(()));
// ... register 20+ more handlers

config.register_category_handler_fn("animations", "animation", |_| Ok(()));
config.register_category_handler_fn("animations", "bezier", |_| Ok(()));

// Access values with string keys and manual type conversion
let border_size = config.get_int("general:border_size")?;
let gaps_in = config.get_string("general:gaps_in")?; // Could be int or string
let binds = config.get_handler_calls("bind").unwrap_or(&vec![]);
```

**With the Hyprland feature** (using high-level Hyprland API):
```rust
use hyprlang::Hyprland;

let mut hypr = Hyprland::new(); // All handlers pre-registered!

// Typed accessor methods
let border_size = hypr.general_border_size()?;         // Returns i64
let gaps_in = hypr.general_gaps_in()?;                 // Returns String (CSS-style)
let active_border = hypr.general_active_border_color()?; // Returns Color

// Convenient array access
let binds = hypr.all_binds();  // Returns Vec<&String>
```

### Complete Example

```rust
use hyprlang::Hyprland;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Hyprland config (handlers auto-registered)
    let mut hypr = Hyprland::new();

    // Parse your Hyprland config
    hypr.parse_file(Path::new("~/.config/hypr/hyprland.conf"))?;

    // === Access General Settings ===
    println!("Border: {}", hypr.general_border_size()?);
    println!("Layout: {}", hypr.general_layout()?);
    println!("Gaps: {} / {}", hypr.general_gaps_in()?, hypr.general_gaps_out()?);

    let active = hypr.general_active_border_color()?;
    println!("Active border: rgba({}, {}, {}, {})",
        active.r, active.g, active.b, active.a);

    // === Access Decoration Settings ===
    println!("\nRounding: {}", hypr.decoration_rounding()?);
    println!("Active opacity: {}", hypr.decoration_active_opacity()?);
    println!("Blur enabled: {}", hypr.decoration_blur_enabled()?);
    println!("Blur size: {}", hypr.decoration_blur_size()?);

    // === Access Animation Settings ===
    if hypr.animations_enabled()? {
        println!("\nAnimations:");
        for anim in hypr.all_animations() {
            println!("  - {}", anim);
        }

        println!("\nBezier curves:");
        for bezier in hypr.all_beziers() {
            println!("  - {}", bezier);
        }
    }

    // === Access Input Settings ===
    println!("\nKeyboard layout: {}", hypr.input_kb_layout()?);
    println!("Mouse sensitivity: {}", hypr.input_sensitivity()?);
    println!("Natural scroll: {}", hypr.input_touchpad_natural_scroll()?);

    // === Access Keybindings ===
    println!("\nKeybindings ({}):", hypr.all_binds().len());
    for (i, bind) in hypr.all_binds().iter().enumerate() {
        println!("  [{}] {}", i + 1, bind);
    }

    // === Access Window Rules ===
    println!("\nWindow rules ({}):", hypr.all_windowrules().len());
    for rule in hypr.all_windowrules() {
        println!("  - {}", rule);
    }

    // === Access Variables ===
    println!("\nVariables:");
    for (name, value) in hypr.variables() {
        println!("  ${} = {}", name, value);
    }

    // === Access Monitors ===
    for monitor in hypr.all_monitors() {
        println!("Monitor: {}", monitor);
    }

    // === Access Environment Variables ===
    for env in hypr.all_env() {
        println!("Env: {}", env);
    }

    // === Access Autostart ===
    for exec in hypr.all_exec_once() {
        println!("Exec-once: {}", exec);
    }

    Ok(())
}
```

### Available Methods

#### General Settings
```rust
hypr.general_border_size() -> Result<i64>
hypr.general_gaps_in() -> Result<String>           // CSS-style: "5" or "5 10 15 20"
hypr.general_gaps_out() -> Result<String>          // CSS-style: "20" or "5 10 15 20"
hypr.general_layout() -> Result<&str>              // "dwindle" or "master"
hypr.general_allow_tearing() -> Result<bool>
hypr.general_active_border_color() -> Result<Color>
hypr.general_inactive_border_color() -> Result<Color>
```

#### Decoration Settings
```rust
hypr.decoration_rounding() -> Result<i64>
hypr.decoration_active_opacity() -> Result<f64>
hypr.decoration_inactive_opacity() -> Result<f64>
hypr.decoration_blur_enabled() -> Result<bool>
hypr.decoration_blur_size() -> Result<i64>
hypr.decoration_blur_passes() -> Result<i64>
```

#### Animation Settings
```rust
hypr.animations_enabled() -> Result<bool>
hypr.all_animations() -> Vec<&String>       // All animation definitions
hypr.all_beziers() -> Vec<&String>          // All bezier curve definitions
```

#### Input Settings
```rust
hypr.input_kb_layout() -> Result<&str>
hypr.input_follow_mouse() -> Result<i64>
hypr.input_sensitivity() -> Result<f64>
hypr.input_touchpad_natural_scroll() -> Result<bool>
```

#### Layout Settings
```rust
hypr.dwindle_pseudotile() -> Result<bool>
hypr.dwindle_preserve_split() -> Result<bool>
hypr.master_new_status() -> Result<&str>
```

#### Misc Settings
```rust
hypr.misc_disable_hyprland_logo() -> Result<bool>
hypr.misc_force_default_wallpaper() -> Result<i64>
```

#### Handler Calls (Arrays)
```rust
hypr.all_binds() -> Vec<&String>            // All bind definitions
hypr.all_bindm() -> Vec<&String>            // All mouse bindings
hypr.all_bindel() -> Vec<&String>           // All bindel definitions
hypr.all_bindl() -> Vec<&String>            // All bindl definitions
hypr.all_windowrules() -> Vec<&String>      // All windowrule definitions
hypr.all_windowrulesv2() -> Vec<&String>    // All windowrulev2 definitions
hypr.all_layerrules() -> Vec<&String>       // All layerrule definitions
hypr.all_workspaces() -> Vec<&String>       // All workspace definitions
hypr.all_monitors() -> Vec<&String>         // All monitor definitions
hypr.all_env() -> Vec<&String>              // All env definitions
hypr.all_exec() -> Vec<&String>             // All exec definitions
hypr.all_exec_once() -> Vec<&String>        // All exec-once definitions
```

#### Variables
```rust
hypr.variables() -> &HashMap<String, String>  // All variables
hypr.get_variable(name: &str) -> Option<&String>  // Get specific variable
```

#### Direct Config Access

If you need access to the underlying low-level `Config` API:

```rust
let config: &Config = hypr.config();        // Immutable access
let config: &mut Config = hypr.config_mut(); // Mutable access

// Use all Config methods
let custom_value = config.get("custom:key")?;
config.register_handler_fn("custom", |ctx| { /* ... */ Ok(()) });
```

### What Handlers Are Pre-Registered?

The `Hyprland` struct automatically registers these handlers:

**Root-level handlers:**
- `monitor` - Monitor configuration
- `env` - Environment variables
- `bind`, `bindm`, `bindel`, `bindl`, `bindr`, `binde`, `bindn` - Keybindings
- `windowrule`, `windowrulev2` - Window rules
- `layerrule` - Layer rules
- `workspace` - Workspace configuration
- `exec`, `exec-once` - Commands
- `source` - File inclusion
- `blurls` - Blur layer surface
- `plugin` - Plugin loading

**Category-specific handlers:**
- `animations:animation` - Animation definitions
- `animations:bezier` - Bezier curve definitions

**Special categories:**
- `device[name]` - Per-device input configuration
- `monitor[name]` - Per-monitor configuration (keyed category)

### When to Use Each API

**Use the `Hyprland` API when:**
- ✅ You're working specifically with Hyprland configurations
- ✅ You want typed, convenient access to common config values
- ✅ You want all Hyprland handlers pre-registered automatically
- ✅ You're building tools for Hyprland users (config editors, validators, etc.)

**Use the low-level `Config` API when:**
- ✅ You're implementing a different config language
- ✅ You need full control over handler registration
- ✅ You're working with a custom config format
- ✅ You want minimal dependencies (no Hyprland-specific code)

## Usage Examples

### Basic Values

```rust
use hyprlang::Config;

let mut config = Config::new();
config.parse(r#"
    # Integers and floats
    count = 42
    opacity = 0.95

    # Strings
    terminal = kitty
    shell = "zsh"

    # Booleans
    enabled = true
    disabled = false
"#)?;

assert_eq!(config.get_int("count")?, 42);
assert_eq!(config.get_float("opacity")?, 0.95);
assert_eq!(config.get_string("terminal")?, "kitty");
```

### Variables

```rust
use hyprlang::Config;

let mut config = Config::new();
config.parse(r#"
    $terminal = kitty
    $mod = SUPER

    # Variables are expanded when used
    my_term = $terminal
    modifier = $mod
"#)?;

// Access variables directly
let vars = config.variables();
assert_eq!(vars.get("terminal"), Some(&"kitty".to_string()));

// Or access expanded values
assert_eq!(config.get_string("my_term")?, "kitty");
```

### Colors

```rust
use hyprlang::Config;

let mut config = Config::new();
config.parse(r#"
    color1 = rgba(33ccffee)
    color2 = rgb(255, 128, 64)
    color3 = 0xff8040ff
"#)?;

let color = config.get_color("color1")?;
println!("R: {}, G: {}, B: {}, A: {}", color.r, color.g, color.b, color.a);
```

### Vec2 (2D Coordinates)

```rust
use hyprlang::Config;

let mut config = Config::new();
config.parse(r#"
    position1 = 100, 200
    position2 = (50, 75)
"#)?;

let pos = config.get_vec2("position1")?;
assert_eq!(pos.x, 100.0);
assert_eq!(pos.y, 200.0);
```

### Expressions

```rust
use hyprlang::Config;

let mut config = Config::new();
config.parse(r#"
    $base = 10

    # Arithmetic expressions with {{}}
    double = {{$base * 2}}
    sum = {{5 + 3}}
    complex = {{($base + 5) * 2}}
"#)?;

assert_eq!(config.get_int("double")?, 20);
assert_eq!(config.get_int("sum")?, 8);
assert_eq!(config.get_int("complex")?, 30);
```

### Nested Categories

```rust
use hyprlang::Config;

let mut config = Config::new();
config.parse(r#"
    general {
        border_size = 2

        gaps {
            inner = 5
            outer = 10
        }
    }
"#)?;

// Access with colon-separated paths
assert_eq!(config.get_int("general:border_size")?, 2);
assert_eq!(config.get_int("general:gaps:inner")?, 5);
assert_eq!(config.get_int("general:gaps:outer")?, 10);
```

### Custom Handlers

```rust
use hyprlang::Config;

let mut config = Config::new();

// Register a handler for custom keywords
config.register_handler_fn("bind", |ctx| {
    println!("Bind: {}", ctx.value);
    Ok(())
});

config.parse(r#"
    bind = SUPER, Q, exec, kitty
    bind = SUPER, C, killactive
"#)?;

// Access handler calls as arrays
let binds = config.get_handler_calls("bind").unwrap();
assert_eq!(binds.len(), 2);
```

### Category-Specific Handlers

```rust
use hyprlang::Config;

let mut config = Config::new();

// Register handlers that only work in specific categories
config.register_category_handler_fn("animations", "animation", |ctx| {
    println!("Animation: {}", ctx.value);
    Ok(())
});

config.parse(r#"
    animations {
        animation = windows, 1, 4, default
        animation = fade, 1, 3, quick
    }
"#)?;

// Handler calls are namespaced by category
let anims = config.get_handler_calls("animations:animation").unwrap();
assert_eq!(anims.len(), 2);
```

### Special Categories

```rust
use hyprlang::{Config, SpecialCategoryDescriptor, SpecialCategoryType};

let mut config = Config::new();

// Register a special category
config.register_special_category(
    SpecialCategoryDescriptor::new("device", SpecialCategoryType::Keyed)
);

config.parse(r#"
    device[mouse] {
        sensitivity = 0.5
        accel_profile = flat
    }

    device[keyboard] {
        repeat_rate = 50
        repeat_delay = 300
    }
"#)?;

// Access keyed category instances
let mouse = config.get_special_category("device", "mouse")?;
println!("Mouse sensitivity: {:?}", mouse.get("sensitivity"));
```

### Source Directive

```hyprlang
 # .colors.conf
 
 $borderSize = 3
```

```rust
use hyprlang::{Config, ConfigOptions};
use std::path::PathBuf;

let mut options = ConfigOptions::default();
options.base_dir = Some(PathBuf::from("/path/to/config"));

let mut config = Config::with_options(options);

// This will include another config file
config.parse(r#"
    source = ./colors.conf

    general {
        border_size = $borderSize
    }
"#)?;
```

### Parse from File

```rust
use hyprlang::Config;
use std::path::Path;

let mut config = Config::new();
config.parse_file(Path::new("config.conf"))?;

// Access all keys
for key in config.keys() {
    println!("Key: {}", key);
}
```

### Hyprland Feature (Optional)

When the `hyprland` feature is enabled, you can use the high-level `Hyprland` struct:

```rust
use hyprlang::Hyprland;
use std::path::Path;

// Create a new Hyprland config (automatically registers all handlers)
let mut hypr = Hyprland::new();

// Parse your Hyprland config
hypr.parse_file(Path::new("~/.config/hypr/hyprland.conf"))?;

// Access config with typed methods
let border_size = hypr.general_border_size()?;
let gaps_in = hypr.general_gaps_in()?;
let active_border = hypr.general_active_border_color()?;

// Get all binds as an array
let binds = hypr.all_binds();
for bind in binds {
    println!("Bind: {}", bind);
}

// Get all animations
let animations = hypr.all_animations();
println!("Found {} animations", animations.len());

// Get all window rules
let rules = hypr.all_windowrules();
for rule in rules {
    println!("Rule: {}", rule);
}

// Access variables
let terminal = hypr.get_variable("terminal");
```

The `Hyprland` struct provides convenient typed access to:
- **General settings**: border_size, gaps, colors, layout, etc.
- **Decoration**: rounding, opacity, blur settings
- **Animations**: enabled status, all animations, all beziers
- **Input**: keyboard layout, mouse settings, touchpad
- **Layout**: dwindle and master layout settings
- **Handlers**: all binds, windowrules, monitors, env vars, exec-once, etc.
- **Variables**: all user-defined variables

## Configuration Options

```rust
use hyprlang::{Config, ConfigOptions};
use std::path::PathBuf;

let mut options = ConfigOptions::default();

// Collect all errors instead of stopping at the first one
options.throw_all_errors = false;

// Allow parsing after initial parse
options.allow_dynamic_parsing = true;

// Base directory for resolving source directives
options.base_dir = Some(PathBuf::from("/path/to/config"));

let config = Config::with_options(options);
```

## API Overview

### Main Types

- `Config` - Main configuration manager
- `ConfigValue` - Enum representing all value types
  - `Int(i64)` - Integer value
  - `Float(f64)` - Float value
  - `String(String)` - String value
  - `Vec2(Vec2)` - 2D coordinate
  - `Color(Color)` - RGBA color
  - `Custom { type_name, value }` - Custom value type
- `Color` - RGBA color (r, g, b, a)
- `Vec2` - 2D coordinate (x, y)

### Key Methods

```rust
// Parsing
config.parse(content: &str) -> Result<()>
config.parse_file(path: &Path) -> Result<()>

// Getting values
config.get(key: &str) -> Result<&ConfigValue>
config.get_int(key: &str) -> Result<i64>
config.get_float(key: &str) -> Result<f64>
config.get_string(key: &str) -> Result<&str>
config.get_vec2(key: &str) -> Result<Vec2>
config.get_color(key: &str) -> Result<Color>

// Setting values
config.set(key: impl Into<String>, value: ConfigValue)
config.set_variable(name: String, value: String)

// Querying
config.keys() -> Vec<&str>
config.variables() -> &HashMap<String, String>
config.has(key: &str) -> bool

// Handlers
config.register_handler_fn(keyword, handler_fn)
config.register_category_handler_fn(category, keyword, handler_fn)
config.get_handler_calls(handler: &str) -> Option<&Vec<String>>
config.all_handler_calls() -> &HashMap<String, Vec<String>>

// Special categories
config.register_special_category(descriptor)
config.get_special_category(category: &str, key: &str) -> Result<HashMap<String, &ConfigValue>>
```

## Examples

The repository includes several examples demonstrating different features:

### `examples/pretty_print.rs`

A comprehensive example showing a complete configuration with all features:

```bash
cargo run --example pretty_print
```

### `examples/parse_hyprland.rs`

Parse and pretty-print a real Hyprland configuration file:

```bash
cargo run --example parse_hyprland
```

This example demonstrates:
- Parsing complex real-world configs
- Variable handling
- Nested categories
- Handler calls (binds, windowrules, etc.)
- Beautiful formatted output

### `examples/hyprland_api.rs`

Demonstrate the high-level Hyprland API (requires `hyprland` feature):

```bash
cargo run --example hyprland_api --features hyprland
```

This example demonstrates:
- Using the `Hyprland` struct for typed config access
- Accessing general, decoration, animation, and input settings
- Getting all binds, windowrules, and other handler calls
- Working with variables
- Comprehensive display of all Hyprland configuration options

## Testing

Run the full test suite:

```bash
cargo test
```

The project includes:
- 29 unit tests covering core functionality
- 12 integration tests for Hyprland-specific scenarios
- 3 documentation tests

All tests from the original Hyprlang C++ implementation have been ported and pass successfully.

## Grammar

The parser is implemented using [pest](https://pest.rs/) with a PEG grammar. The grammar file is located at `src/hyprlang.pest`.

Key syntax features:
- Comments: `#` for single-line, `##` for documentation
- Variables: `$VAR = value`
- Expressions: `{{expr}}`
- Categories: `category { ... }`
- Special categories: `category[key] { ... }`
- Assignments: `key = value`
- Handlers: `keyword = value`
- Directives: `source = path`, `# hyprlang if/endif/noerror`

## License

This project is a reimplementation of Hyprlang in Rust, based on the original C++ implementation by the Hyprland team.

## Contributing

Contributions are welcome! Please ensure all tests pass before submitting a PR:

```bash
cargo test
cargo clippy
cargo fmt --check
```

## Acknowledgments

- [Hyprland](https://hyprland.org/) - The original project
- [Hyprlang](https://github.com/hyprwm/hyprlang) - The C++ reference implementation
- [pest](https://pest.rs/) - The parser generator used in this project
