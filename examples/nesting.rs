use hyprlang::{Color, Config, ConfigOptions, ConfigValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build path to the test config
    let mut config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("examples/nested/parent.conf");

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                         Nested Config                         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let mut options = ConfigOptions::default();
    options.base_dir = Some(config_path.parent().unwrap().to_path_buf());
    options.throw_all_errors = false;

    let mut config = Config::with_options(options);

    match config.parse_file(&config_path) {
        Ok(_) => println!("✅ Successfully parsed configuration!\n"),
        Err(e) => {
            println!("❌ Parse error: {}\n", e);
            println!("Note: Some Hyprland-specific syntax may not be fully supported yet.\n");
        }
    }

    // Pretty print the configuration
    pretty_print_config(&config);
    Ok(())
}

fn pretty_print_config(config: &Config) {
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                         VARIABLES                           │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Group and print variables
    let all_variables = config.variables();
    let mut variables: Vec<_> = all_variables.iter().collect();
    variables.sort_by_key(|(name, _)| *name);

    if variables.is_empty() {
        println!("  ℹ️  No variables found in configuration\n");
    } else {
        for (var_name, value) in &variables {
            println!("  ${:<20} = \"{}\"", var_name, value);
        }
        println!();
    }

    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                    CONFIGURATION VALUES                     │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Group values by category
    let all_keys = config.keys();
    let mut by_category: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for key in &all_keys {
        if let Ok(value) = config.get(key) {
            let value_str = format_value(value);
            if key.contains(':') {
                let parts: Vec<&str> = key.split(':').collect();
                let category = parts[0].to_string();
                let subkey = parts[1..].join(":");

                by_category
                    .entry(category)
                    .or_insert_with(Vec::new)
                    .push((subkey, value_str));
            } else if !key.starts_with('$') {
                by_category
                    .entry("".to_string())
                    .or_insert_with(Vec::new)
                    .push((key.to_string(), value_str));
            }
        }
    }

    // Print root level values first
    if let Some(root_values) = by_category.get("") {
        println!("┌─ Root Level ─────────────────────────────────────────────┐\n");
        for (key, value_str) in root_values {
            println!("  {:<30} = {}", key, value_str);
        }
        println!();
    }

    // Print categorized values
    let mut categories: Vec<_> = by_category
        .keys()
        .filter(|k| !k.is_empty())
        .cloned()
        .collect();
    categories.sort();

    for category in categories {
        println!("┌─ {} {}", category, "─".repeat(60 - category.len()));
        println!("│");

        let mut values = by_category.get(&category).unwrap().clone();
        values.sort_by_key(|(k, _)| k.clone());

        for (key, value_str) in values {
            let indent = get_indent_level(&key);
            let display_key = get_display_key(&key);

            println!(
                "│ {}{:<28} = {}",
                "  ".repeat(indent),
                display_key,
                value_str
            );
        }

        println!("│");
        println!("└{}", "─".repeat(62));
        println!();
    }

    // Print handler calls (binds, windowrules, etc.)
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                       HANDLER CALLS                         │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    let handler_calls = config.all_handler_calls();
    if handler_calls.is_empty() {
        println!("  ℹ️  No handler calls found\n");
    } else {
        let mut handler_names: Vec<_> = handler_calls.keys().collect();
        handler_names.sort();

        for handler_name in handler_names {
            let calls = &handler_calls[handler_name];
            println!(
                "┌─ {} ({} entries) {}",
                handler_name,
                calls.len(),
                "─".repeat(
                    60_usize
                        .saturating_sub(handler_name.len() + calls.len().to_string().len() + 13)
                )
            );
            println!("│");

            for (i, value) in calls.iter().enumerate() {
                let display_value = if value.len() > 55 {
                    format!("{}...", &value[..52])
                } else {
                    value.clone()
                };
                println!("│ [{:3}] {}", i + 1, display_value);
            }

            println!("│");
            println!("└{}", "─".repeat(62));
            println!();
        }
    }

    // Print summary
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                          SUMMARY                            │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    let total_values = all_keys.len();
    let total_categories = by_category.keys().filter(|k| !k.is_empty()).count();
    let total_handlers: usize = handler_calls.values().map(|v| v.len()).sum();

    println!("  📊 Total configuration values: {}", total_values);
    println!("  📁 Total categories: {}", total_categories);
    println!("  🔧 Total handler calls: {}", total_handlers);

    // Count different value types
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
        ConfigValue::Int(i) => format!("{}", i),
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
        "rgba({}, {}, {}, {}) [#{:02x}{:02x}{:02x}{:02x}]",
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
