#![cfg(feature = "mutation")]

use hyprlang::Config;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Helper to create a temporary directory for test files
fn create_test_dir() -> PathBuf {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprlang_multi_file_test_{}_{}",
        timestamp,
        counter
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper to clean up test directory
fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_multi_file_source_tracking() {
    let test_dir = create_test_dir();

    // Create subconfig1.conf with variables
    let subconfig1_path = test_dir.join("subconfig1.conf");
    fs::write(
        &subconfig1_path,
        r#"$GAPS = 10
$BORDER = 2
"#,
    )
    .unwrap();

    // Create subconfig2.conf with decoration settings
    let subconfig2_path = test_dir.join("subconfig2.conf");
    fs::write(
        &subconfig2_path,
        r#"decoration {
    rounding = 5
    active_opacity = 0.95
}
"#,
    )
    .unwrap();

    // Create master.conf that sources both subconfigs
    let master_path = test_dir.join("master.conf");
    fs::write(
        &master_path,
        format!(
            r#"# Master configuration
source = {}
source = {}

general {{
    border_size = $BORDER
}}
"#,
            subconfig1_path.display(),
            subconfig2_path.display()
        ),
    )
    .unwrap();

    // Parse the master config
    let mut config = Config::new();
    config.parse_file(&master_path).unwrap();

    // Verify values from all files were loaded
    assert_eq!(config.get_variable("GAPS"), Some("10"));
    assert_eq!(config.get_variable("BORDER"), Some("2"));
    assert_eq!(config.get_int("decoration:rounding").unwrap(), 5);
    assert_eq!(config.get_float("decoration:active_opacity").unwrap(), 0.95);

    // Verify source files are tracked
    let source_files = config.get_source_files();
    assert!(source_files.len() >= 3, "Expected at least 3 source files, got {}", source_files.len());

    // Verify key source tracking
    let var_source = config.get_key_source_file("$GAPS");
    assert!(var_source.is_some(), "Expected to find source file for $GAPS");

    let rounding_source = config.get_key_source_file("decoration:rounding");
    assert!(rounding_source.is_some(), "Expected to find source file for decoration:rounding");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_multi_file_mutation_updates_correct_file() {
    let test_dir = create_test_dir();

    // Create subconfig1.conf with a variable
    let subconfig1_path = test_dir.join("subconfig1.conf");
    fs::write(&subconfig1_path, "$MY_VAR = original\n").unwrap();

    // Create subconfig2.conf with a value we'll modify
    let subconfig2_path = test_dir.join("subconfig2.conf");
    fs::write(
        &subconfig2_path,
        r#"decoration {
    rounding = 5
}
"#,
    )
    .unwrap();

    // Create master.conf
    let master_path = test_dir.join("master.conf");
    fs::write(
        &master_path,
        format!(
            r#"source = {}
source = {}

border_size = 3
"#,
            subconfig1_path.display(),
            subconfig2_path.display()
        ),
    )
    .unwrap();

    // Parse the master config
    let mut config = Config::new();
    config.parse_file(&master_path).unwrap();

    // Verify initial values
    assert_eq!(config.get_int("decoration:rounding").unwrap(), 5);
    assert_eq!(config.get_int("border_size").unwrap(), 3);

    // Mutate the value from subconfig2
    config.set_int("decoration:rounding", 15);

    // Verify the mutation was tracked
    let modified = config.get_modified_files();
    assert!(!modified.is_empty(), "Expected at least one modified file");

    // Save all modified files
    let saved = config.save_all().unwrap();
    assert!(!saved.is_empty(), "Expected at least one file to be saved");

    // Read back subconfig2 to verify the change
    let subconfig2_content = fs::read_to_string(&subconfig2_path).unwrap();
    assert!(
        subconfig2_content.contains("rounding = 15"),
        "Expected subconfig2 to contain 'rounding = 15', got:\n{}",
        subconfig2_content
    );

    // Read back subconfig1 to verify it was NOT changed
    let subconfig1_content = fs::read_to_string(&subconfig1_path).unwrap();
    assert!(
        subconfig1_content.contains("$MY_VAR = original"),
        "Expected subconfig1 to still contain original value, got:\n{}",
        subconfig1_content
    );

    // Read back master to verify source directives are preserved
    let master_content = fs::read_to_string(&master_path).unwrap();
    assert!(
        master_content.contains("source ="),
        "Expected master to still contain source directive, got:\n{}",
        master_content
    );

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_multi_file_variable_mutation() {
    let test_dir = create_test_dir();

    // Create vars.conf with variables
    let vars_path = test_dir.join("vars.conf");
    fs::write(
        &vars_path,
        r#"$GAPS = 10
$OPACITY = 0.9
"#,
    )
    .unwrap();

    // Create master.conf
    let master_path = test_dir.join("master.conf");
    fs::write(
        &master_path,
        format!(
            r#"source = {}

border_size = $GAPS
"#,
            vars_path.display()
        ),
    )
    .unwrap();

    // Parse
    let mut config = Config::new();
    config.parse_file(&master_path).unwrap();

    // Verify variable is loaded
    assert_eq!(config.get_variable("GAPS"), Some("10"));

    // Mutate the variable
    config.set_variable("GAPS".to_string(), "20".to_string());

    // Save all
    let saved = config.save_all().unwrap();
    assert!(!saved.is_empty(), "Expected at least one file to be saved");

    // Read back vars.conf to verify the change
    let vars_content = fs::read_to_string(&vars_path).unwrap();
    assert!(
        vars_content.contains("$GAPS = 20"),
        "Expected vars.conf to contain '$GAPS = 20', got:\n{}",
        vars_content
    );

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_new_key_goes_to_primary_file() {
    let test_dir = create_test_dir();

    // Create subconfig.conf
    let subconfig_path = test_dir.join("subconfig.conf");
    fs::write(&subconfig_path, "existing_key = 123\n").unwrap();

    // Create master.conf
    let master_path = test_dir.join("master.conf");
    fs::write(
        &master_path,
        format!(
            r#"source = {}

master_key = 456
"#,
            subconfig_path.display()
        ),
    )
    .unwrap();

    // Parse
    let mut config = Config::new();
    config.parse_file(&master_path).unwrap();

    // Add a completely new key
    config.set_int("brand_new_key", 789);

    // Save all
    config.save_all().unwrap();

    // The new key should be in the master file (primary)
    let master_content = fs::read_to_string(&master_path).unwrap();
    assert!(
        master_content.contains("brand_new_key = 789"),
        "Expected new key to be added to master.conf, got:\n{}",
        master_content
    );

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_serialize_specific_file() {
    let test_dir = create_test_dir();

    // Create subconfig.conf
    let subconfig_path = test_dir.join("subconfig.conf");
    fs::write(&subconfig_path, "sub_key = 100\n").unwrap();

    // Create master.conf
    let master_path = test_dir.join("master.conf");
    fs::write(
        &master_path,
        format!(
            r#"source = {}

master_key = 200
"#,
            subconfig_path.display()
        ),
    )
    .unwrap();

    // Parse
    let config = Config::new();
    let mut config = config;
    config.parse_file(&master_path).unwrap();

    // Serialize the subconfig file
    let canonical_subconfig = subconfig_path.canonicalize().unwrap();
    let serialized = config.serialize_file(&canonical_subconfig).unwrap();
    assert!(
        serialized.contains("sub_key = 100"),
        "Expected serialized subconfig to contain 'sub_key = 100', got:\n{}",
        serialized
    );

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_round_trip_with_multi_file_mutation() {
    let test_dir = create_test_dir();

    // Create appearance.conf
    let appearance_path = test_dir.join("appearance.conf");
    fs::write(
        &appearance_path,
        r#"decoration {
    rounding = 10
    blur = true
}
"#,
    )
    .unwrap();

    // Create master.conf
    let master_path = test_dir.join("master.conf");
    fs::write(
        &master_path,
        format!(
            r#"source = {}

border_size = 2
"#,
            appearance_path.display()
        ),
    )
    .unwrap();

    // First parse
    let mut config1 = Config::new();
    config1.parse_file(&master_path).unwrap();

    // Mutate
    config1.set_int("decoration:rounding", 25);

    // Save
    config1.save_all().unwrap();

    // Parse again with fresh config
    let mut config2 = Config::new();
    config2.parse_file(&master_path).unwrap();

    // Verify the mutation persisted
    assert_eq!(
        config2.get_int("decoration:rounding").unwrap(),
        25,
        "Expected rounding to be 25 after round-trip"
    );

    // Verify other values weren't affected
    assert_eq!(config2.get_int("border_size").unwrap(), 2);

    cleanup_test_dir(&test_dir);
}
