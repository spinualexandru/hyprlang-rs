//! Comprehensive example demonstrating the mutation and serialization features.
//!
//! This example requires the `mutation` feature to be enabled:
//! ```bash
//! cargo run --example mutation_example --features mutation
//! ```

#[cfg(feature = "mutation")]
use hyprlang::{Config, ConfigValue};

#[cfg(feature = "mutation")]
fn run_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hyprlang Mutation & Serialization Example ===\n");

    // Create a new configuration
    let mut config = Config::new();

    // Parse some initial configuration
    config.parse(
        r#"
# Configuration for my app
$GAPS = 10
$SCALE = 2
$FILE_MANAGER = nautilus

general {
    border_size = 3
    gaps_in = $GAPS
    gaps_out = {{GAPS * 2}}
    active_opacity = 0.9
}

decoration {
    rounding = 10
    shadow {
        enabled = true
        range = 20
    }
}
"#,
    )?;

    println!("📖 Initial configuration loaded\n");

    // ========== VALUE MUTATIONS ==========
    println!("🔧 Mutating configuration values...\n");

    // Set a new value using the typed setter
    config.set_int("general:border_size", 5);
    println!("✓ Changed border_size: {} -> 5", 3);

    // Set a float value
    config.set_float("general:active_opacity", 1.0);
    println!("✓ Changed active_opacity: {} -> 1.0", 0.9);

    // Add a new value that doesn't exist
    config.set("decoration:blur", ConfigValue::Int(1));
    println!("✓ Added new value: decoration:blur = 1");

    // Remove a value
    let old_value = config.remove("decoration:rounding")?;
    println!("✓ Removed decoration:rounding (was: {})", old_value);

    // ========== VARIABLE MUTATIONS ==========
    println!("\n📝 Mutating variables...\n");

    // Method 1: Direct variable mutation
    config.set_variable("GAPS".to_string(), "15".to_string());
    println!("✓ Changed $GAPS: 10 -> 15");

    // Method 2: Using mutable variable reference
    if let Some(mut file_manager) = config.get_variable_mut("FILE_MANAGER") {
        let old_value = file_manager.get().to_string();
        file_manager.set("thunar")?;
        println!(
            "✓ Changed ${}: {} -> thunar",
            file_manager.name(),
            old_value
        );
    }

    // Add a new variable
    config.set_variable("COLOR_THEME".to_string(), "dark".to_string());
    println!("✓ Added new variable: $COLOR_THEME = dark");

    // Remove a variable
    if let Some(old_value) = config.remove_variable("SCALE") {
        println!("✓ Removed $SCALE (was: {})", old_value);
    }

    // ========== HANDLER MUTATIONS ==========
    println!("\n⚡ Mutating handler calls...\n");

    // Register a handler (required before using it)
    config.register_handler_fn("bind", |_ctx| Ok(()));
    config.register_handler_fn("exec-once", |_ctx| Ok(()));

    // Add handler calls
    config.add_handler_call("bind", "SUPER, A, exec, terminal".to_string())?;
    config.add_handler_call("bind", "SUPER, B, exec, browser".to_string())?;
    config.add_handler_call("exec-once", "waybar".to_string())?;

    println!("✓ Added 2 bind handlers");
    println!("✓ Added 1 exec-once handler");

    // Remove a specific handler call by index
    let removed = config.remove_handler_call("bind", 0)?;
    println!("✓ Removed bind[0]: {}", removed);

    // ========== VERIFICATION ==========
    println!("\n✅ Verifying mutations...\n");

    // Verify the changes
    assert_eq!(config.get_int("general:border_size")?, 5);
    println!("  border_size = {}", config.get_int("general:border_size")?);

    assert_eq!(config.get_float("general:active_opacity")?, 1.0);
    println!(
        "  active_opacity = {}",
        config.get_float("general:active_opacity")?
    );

    assert_eq!(config.get_variable("GAPS"), Some("15"));
    println!("  $GAPS = {}", config.get_variable("GAPS").unwrap());

    assert_eq!(config.get_variable("FILE_MANAGER"), Some("thunar"));
    println!(
        "  $FILE_MANAGER = {}",
        config.get_variable("FILE_MANAGER").unwrap()
    );

    assert!(config.get_handler_calls("bind").is_some());
    println!(
        "  bind handlers: {:?}",
        config.get_handler_calls("bind").unwrap()
    );

    // ========== SERIALIZATION ==========
    println!("\n💾 Serializing configuration...\n");

    let serialized = config.serialize();
    println!("Serialized config (synthetic format, first 400 chars):");
    println!("{}", &serialized[..serialized.len().min(400)]);
    if serialized.len() > 400 {
        println!("... (truncated)\n");
    } else {
        println!();
    }

    // Save to a temporary file
    let temp_file = "/tmp/hyprlang_mutation_example.conf";
    config.save_as(temp_file)?;
    println!("✓ Saved to: {}", temp_file);

    // Verify we can read it back
    let mut config2 = Config::new();
    config2.parse_file(std::path::Path::new(temp_file))?;
    assert_eq!(config2.get_int("general:border_size")?, 5);
    assert_eq!(config2.get_variable("GAPS"), Some("15"));
    println!("✓ Verified round-trip: parse -> mutate -> save -> parse");

    // Clean up
    std::fs::remove_file(temp_file).ok();

    println!("\n🎉 All mutations and serialization working correctly!");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "mutation")]
    {
        run_example()
    }

    #[cfg(not(feature = "mutation"))]
    {
        eprintln!("❌ This example requires the 'mutation' feature to be enabled.");
        eprintln!("\nPlease run with:");
        eprintln!("  cargo run --example mutation_example --features mutation");
        std::process::exit(1);
    }
}
