use hyprlang::Config;
use std::path::PathBuf;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("examples/example.conf");

    let mut config = Config::new();
    println!("✅ Successfully parsed configuration!\n");

    // Parse the configuration
    config.parse_file(&config_path)?;

    let file_manager = config.get_variable("fileManager");
    let shadow_enabled = config.get_int("decoration:shadow:enabled")?;

    println!("File Manager: {}", file_manager.unwrap());
    println!("Shadow Enabled?: {}", shadow_enabled);
    
    config.set_int("decoration:shadow:enabled", 0);
    let shadow_enabled = config.get_int("decoration:shadow:enabled")?;
    println!("Shadow Enabled after mutation?: {}", shadow_enabled);
    let mut new_config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    new_config_path.push("examples/example_modified.conf");
    config.save_as(new_config_path);

    Ok(())
}
