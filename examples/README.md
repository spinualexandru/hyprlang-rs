# Hyprlang-rs Examples

This directory contains example programs demonstrating how to use the Hyprlang parser.

## Examples

### 1. Pretty Print (`pretty_print.rs`)

A comprehensive example that demonstrates parsing and pretty-printing a Hyprlang configuration file.

**Run it:**
```bash
cargo run --example pretty_print
```

**Features demonstrated:**
- Parsing configuration files
- Variable definitions and expansion
- Nested categories
- Expression evaluation
- Color parsing (rgba, hex)
- Vec2 values
- Type detection and formatting
- Configuration statistics

**Output:**
The example produces a beautifully formatted output showing:
- All defined variables
- Configuration values grouped by category
- Proper indentation for nested values
- Color values with both decimal and hex representation
- Summary statistics (total values, categories, type counts)

### 2. Parse Hyprland Config (`parse_hyprland.rs`)

An example that attempts to parse a real Hyprland configuration file.

**Run it:**
```bash
cargo run --example parse_hyprland
```

**Features demonstrated:**
- Handler registration for Hyprland-specific keywords
- Graceful error handling
- Parsing complex real-world configurations
- Pretty-printing parsed results

**Note:** Some advanced Hyprland syntax (like gradient borders with multiple colors) may not be fully supported yet.

## Example Configuration

The `example.conf` file demonstrates all supported Hyprlang features:

```hyprlang
# Variables
$SCALE = 2
$WIDTH = 1920

# Simple values
monitor = preferred
refresh_rate = 144

# Nested categories
general {
    gaps_in = 5
    col.active_border = rgba(33ccffee)
}

# Deep nesting
decoration {
    shadow {
        enabled = true
        color = rgba(1a1a1aee)
    }
}

# Expressions
animations {
    default_speed = {{10 * SCALE}}
}

# Vec2
cursor {
    position = (100, 200)
}

# Colors in different formats
colors {
    test_hex = 0xFFAABBCC
    test_rgba = rgba(255, 128, 64, 255)
}
```

## Creating Your Own Examples

To create a new example:

1. Create a new `.rs` file in the `examples/` directory
2. Add a `main()` function that returns `Result<(), Box<dyn std::error::Error>>`
3. Use the `hyprlang_rs` crate to parse configurations:

```rust
use hyprlang_rs::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.parse_file("path/to/config.conf")?;

    // Access values
    let value = config.get_int("some_key")?;
    println!("Value: {}", value);

    Ok(())
}
```

4. Run your example:
```bash
cargo run --example your_example_name
```

## API Features Used

These examples demonstrate:

- **`Config::new()`** - Create a new configuration
- **`Config::parse_file()`** - Parse a configuration file
- **`Config::get()`** - Get a raw configuration value
- **`Config::get_int()`** - Get an integer value
- **`Config::get_float()`** - Get a float value
- **`Config::get_string()`** - Get a string value
- **`Config::get_color()`** - Get a color value
- **`Config::get_vec2()`** - Get a Vec2 value
- **`Config::keys()`** - Get all configuration keys
- **`Config::register_handler_fn()`** - Register custom handlers
- **`ConfigValue::type_name()`** - Get the type of a value

## Tips

1. **Error Handling**: Always handle `Result` types properly in production code
2. **Handlers**: Register handlers for keywords that aren't assignments (like `bind`, `exec`, etc.)
3. **Variables**: Variables are automatically expanded in values
4. **Expressions**: Use `{{}}` syntax for arithmetic expressions
5. **Type Safety**: Use typed getters (`get_int()`, `get_float()`, etc.) for type safety

## Learn More

- See the main README.md for full API documentation
- Check the tests in `tests/` for more usage examples
- Read the inline documentation in `src/lib.rs`
