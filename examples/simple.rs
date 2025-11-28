use hyprlang::Config;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("examples/example.conf");

    let mut config = Config::new();

    // Parse the configuration
    config.parse_file(&config_path)?;
    println!("✅ Successfully parsed configuration!\n");

    // Access variables
    let file_manager = config.get_variable("fileManager");
    println!(
        "File Manager: {}",
        file_manager.unwrap_or(&"Not set".to_string())
    );

    // Access configuration values
    let shadow_enabled = config.get_int("decoration:shadow:enabled")?;
    println!("Shadow Enabled: {}", shadow_enabled);

    // Access other values
    if let Ok(border_size) = config.get_int("general:border_size") {
        println!("Border Size: {}", border_size);
    }

    Ok(())
}
