use hyprlang::{Color, Config, ConfigValue};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build path to the example config
    let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("examples/example.conf");

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║          Hyprlang Parser - Pretty Print Example              ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    println!("📄 Parsing: {}\n", config_path.display());

    let mut config = Config::new();

    // Parse the configuration
    config.parse_file(&config_path)?;

    println!("✅ Successfully parsed configuration!\n");

    // Pretty print the configuration
    pretty_print_config(&config);

    Ok(())
}

fn pretty_print_config(config: &Config) {
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                         VARIABLES                           │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Print variables
    print_variables(config);

    println!("\n╭─────────────────────────────────────────────────────────────╮");
    println!("│                    CONFIGURATION VALUES                     │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Group and print values by category
    print_categorized_values(config);

    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                          SUMMARY                            │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    print_summary(config);
}

fn print_variables(config: &Config) {
    let all_keys = config.keys();

    // Find all variables (keys starting with $)
    let mut var_count = 0;
    for key in &all_keys {
        // Variables are stored with their $ prefix
        if !key.contains(':') && !key.starts_with('$') {
            // Check if it's using a variable
            if let Ok(value) = config.get(key) {
                if let ConfigValue::String(s) = value {
                    if !s.is_empty() {
                        continue;
                    }
                }
            }
        }
    }

    // Print all root-level values that look like variables
    println!("  Variables defined:");
    for key in &all_keys {
        if !key.contains(':') {
            if let Ok(value) = config.get(key) {
                match value {
                    ConfigValue::String(s) if !s.is_empty() && !key.starts_with('$') => {
                        println!("  ${:<20} = \"{}\"", key, s);
                        var_count += 1;
                    }
                    ConfigValue::Int(i) if !key.starts_with('$') => {
                        println!("  ${:<20} = {}", key, i);
                        var_count += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    if var_count == 0 {
        println!("  ℹ️  No variables found");
    }
}

fn print_categorized_values(config: &Config) {
    let all_keys = config.keys();
    let mut by_category: HashMap<String, Vec<(String, ConfigValue)>> = HashMap::new();

    // Group by category
    for key in &all_keys {
        if let Ok(value) = config.get(key) {
            if key.contains(':') {
                let parts: Vec<&str> = key.split(':').collect();
                let category = parts[0].to_string();
                let subkey = parts[1..].join(":");

                by_category
                    .entry(category)
                    .or_insert_with(Vec::new)
                    .push((subkey, value.clone()));
            } else if !is_variable_like(key, value) {
                by_category
                    .entry("".to_string())
                    .or_insert_with(Vec::new)
                    .push((key.to_string(), value.clone()));
            }
        }
    }

    // Print root level values
    if let Some(root_values) = by_category.get("") {
        println!("┌─ Root Level ─────────────────────────────────────────────┐\n");
        for (key, value) in root_values {
            println!("  {:<30} = {}", key, format_value(value));
        }
        println!();
    }

    // Print categories
    let mut categories: Vec<_> = by_category
        .keys()
        .filter(|k| !k.is_empty())
        .cloned()
        .collect();
    categories.sort();

    for category in categories {
        let values = by_category.get(&category).unwrap();
        print_category(&category, values);
    }
}

fn print_category(name: &str, values: &[(String, ConfigValue)]) {
    let header = format!("┌─ {} ", name);
    let padding = "─".repeat(62 - header.len());
    println!("{}{}", header, padding);
    println!("│");

    let mut sorted_values = values.to_vec();
    sorted_values.sort_by_key(|(k, _)| k.clone());

    for (key, value) in sorted_values {
        let indent = get_indent_level(&key);
        let display_key = get_display_key(&key);
        let formatted_value = format_value(&value);

        println!(
            "│ {}{:<28} = {}",
            "  ".repeat(indent),
            display_key,
            formatted_value
        );
    }

    println!("│");
    println!("└{}", "─".repeat(62));
    println!();
}

fn print_summary(config: &Config) {
    let all_keys = config.keys();
    let total_values = all_keys.len();

    // Count categories
    let mut categories = std::collections::HashSet::new();
    for key in &all_keys {
        if let Some(category) = key.split(':').next() {
            if key.contains(':') {
                categories.insert(category);
            }
        }
    }

    println!("  📊 Total configuration values: {}", total_values);
    println!("  📁 Total categories: {}", categories.len());

    // Count value types
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for key in &all_keys {
        if let Ok(value) = config.get(key) {
            *type_counts
                .entry(value.type_name().to_string())
                .or_insert(0) += 1;
        }
    }

    println!("  📝 Value types:");
    let mut types: Vec<_> = type_counts.iter().collect();
    types.sort_by_key(|(name, _)| *name);
    for (type_name, count) in types {
        println!("     • {}: {}", type_name, count);
    }

    println!();
}

fn format_value(value: &ConfigValue) -> String {
    match value {
        ConfigValue::Int(i) => {
            if *i >= 0 && *i <= 1 {
                format!("{} ({})", i, if *i == 1 { "true" } else { "false" })
            } else if *i > 1000000 {
                format!("0x{:X}", i)
            } else {
                format!("{}", i)
            }
        }
        ConfigValue::Float(f) => format!("{}", f),
        ConfigValue::String(s) => {
            if s.is_empty() {
                "\"\"".to_string()
            } else if s.len() > 50 {
                format!("\"{}...\"", &s[..47])
            } else {
                format!("\"{}\"", s)
            }
        }
        ConfigValue::Vec2(v) => format!("({}, {})", v.x, v.y),
        ConfigValue::Color(c) => format_color(c),
        ConfigValue::Custom { type_name, .. } => format!("<{}>", type_name),
    }
}

fn format_color(color: &Color) -> String {
    format!(
        "rgba({}, {}, {}, {}) │ #{:02x}{:02x}{:02x}{:02x}",
        color.r, color.g, color.b, color.a, color.r, color.g, color.b, color.a
    )
}

fn get_indent_level(key: &str) -> usize {
    key.matches(':').count()
}

fn get_display_key(key: &str) -> String {
    if let Some(last_part) = key.split(':').last() {
        last_part.to_string()
    } else {
        key.to_string()
    }
}

fn is_variable_like(key: &str, value: &ConfigValue) -> bool {
    !key.contains(':') && matches!(value, ConfigValue::String(_) | ConfigValue::Int(_))
}
