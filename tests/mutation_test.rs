#![cfg(feature = "mutation")]

use hyprlang::{Config, ConfigValue};

#[test]
fn test_serialize_synthetic() {
    let mut config = Config::new();

    config.parse(r#"
$GAPS = 10
$SCALE = 2

border_size = 3
active_opacity = 0.9

decoration {
    rounding = 10
}
"#).unwrap();

    let serialized = config.serialize();

    // Verify variables are serialized
    assert!(serialized.contains("$GAPS = 10"));
    assert!(serialized.contains("$SCALE = 2"));

    // Verify values are serialized
    assert!(serialized.contains("border_size = 3"));
    assert!(serialized.contains("active_opacity = 0.9"));
    // Category values are serialized in block form, not as flattened keys
    assert!(serialized.contains("decoration"));
}

#[test]
fn test_save_as() {
    let mut config = Config::new();

    config.parse(r#"
test_key = 123
another_key = hello
"#).unwrap();

    // Save to a temporary file
    let temp_file = "/tmp/hyprlang_test_save.conf";
    config.save_as(temp_file).unwrap();

    // Read it back
    let contents = std::fs::read_to_string(temp_file).unwrap();
    assert!(contents.contains("test_key = 123"));
    assert!(contents.contains("another_key = hello"));

    // Clean up
    std::fs::remove_file(temp_file).ok();
}

#[test]
fn test_serialize_with_handlers() {
    let mut config = Config::new();

    // Register a handler
    config.register_handler_fn("bind", |_ctx| Ok(()));

    config.parse(r#"
bind = SUPER, A, exec, terminal
bind = SUPER, B, exec, browser
"#).unwrap();

    let serialized = config.serialize();

    // Verify handler calls are serialized
    assert!(serialized.contains("bind = SUPER, A, exec, terminal"));
    assert!(serialized.contains("bind = SUPER, B, exec, browser"));
}

#[test]
fn test_full_fidelity_serialization() {
    let mut config = Config::new();

    // Parse config with comments and specific formatting
    config.parse(r#"
# Configuration file
$GAPS = 10
$SCALE = 2

# Window settings
border_size = 3
active_opacity = 0.9
"#).unwrap();

    // Get initial serialization
    let initial = config.serialize();
    println!("Initial serialization:\n{}", initial);

    // Mutate some values
    config.set_variable("GAPS".to_string(), "15".to_string());
    config.set_int("border_size", 5);

    // Serialize after mutation
    let mutated = config.serialize();
    println!("\nAfter mutation:\n{}", mutated);

    // Verify mutations are present
    assert!(mutated.contains("$GAPS = 15"));
    assert!(mutated.contains("border_size = 5"));
    assert!(mutated.contains("$SCALE = 2")); // Unchanged variable
    assert!(mutated.contains("active_opacity = 0.9")); // Unchanged value
}

#[test]
fn test_document_preserves_structure() {
    let mut config = Config::new();

    config.parse(r#"
$PRIMARY = red
$SECONDARY = green

general {
border_size = 2
gaps = 10
}
"#).unwrap();

    // Mutate a value
    config.set_int("general:border_size", 5);

    let serialized = config.serialize();
    println!("Serialized with structure:\n{}", serialized);

    // Verify the structure is maintained
    assert!(serialized.contains("$PRIMARY = red"));
    assert!(serialized.contains("general"));
    assert!(serialized.contains("border_size = 5"));
}

#[test]
fn test_round_trip_with_mutation() {
    let mut config1 = Config::new();

    // Parse and mutate
    config1.parse(r#"
$VAR = original
key = value1
"#).unwrap();

    config1.set_variable("VAR".to_string(), "modified".to_string());
    config1.set_string("key", "value2");

    // Serialize
    let serialized = config1.serialize();

    // Parse serialized output into a new config
    let mut config2 = Config::new();
    config2.parse(&serialized).unwrap();

    // Verify values match
    assert_eq!(config2.get_variable("VAR"), Some("modified"));
    assert_eq!(config2.get_string("key").unwrap(), "value2");
}
