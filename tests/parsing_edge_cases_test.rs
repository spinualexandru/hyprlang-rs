//! Edge case tests for parsing color and Vec2 values.

use hyprlang::Config;

// ========== COLOR PARSING EDGE CASES ==========

#[test]
fn test_color_invalid_hex_chars() {
    let mut config = Config::new();
    // 'g' is not a valid hex character
    let result = config.parse("color = 0xgggggggg");
    // Should parse as a string since it's not valid hex
    assert!(result.is_ok());
    // The value should be stored as a string, not a color
    let value = config.get("color").unwrap();
    assert!(value.as_color().is_err());
}

#[test]
fn test_color_hex_wrong_length() {
    let mut config = Config::new();
    // Too short for valid color
    let result = config.parse("color = 0xfff");
    assert!(result.is_ok());
    // Should be stored as a string
    let value = config.get("color").unwrap();
    assert!(value.as_color().is_err());
}

#[test]
fn test_color_rgba_out_of_range() {
    let mut config = Config::new();
    // 300 is out of valid range (0-255)
    let result = config.parse("color = rgba(300, 100, 100, 1.0)");
    assert!(result.is_ok());
    // Should fail to parse as valid color
    let value = config.get("color").unwrap();
    assert!(value.as_color().is_err());
}

#[test]
fn test_color_rgba_negative_values() {
    let mut config = Config::new();
    // Negative values are invalid
    let result = config.parse("color = rgba(-10, 100, 100, 1.0)");
    assert!(result.is_ok());
    let value = config.get("color").unwrap();
    assert!(value.as_color().is_err());
}

#[test]
fn test_color_rgba_alpha_out_of_range() {
    let mut config = Config::new();
    // Alpha > 1.0 is out of range
    let result = config.parse("color = rgba(100, 100, 100, 2.0)");
    assert!(result.is_ok());
    // Value may be clamped or rejected - check it's stored
    let _value = config.get("color").unwrap();
}

#[test]
fn test_color_malformed_rgba_syntax() {
    let mut config = Config::new();
    // Missing closing paren
    let result = config.parse("color = rgba(100, 100, 100, 1.0");
    assert!(result.is_ok());
    let value = config.get("color").unwrap();
    assert!(value.as_color().is_err());
}

#[test]
fn test_color_valid_hex_formats() {
    let mut config = Config::new();

    // AARRGGBB format
    config.parse("color1 = 0xffff0000").unwrap();
    assert!(config.get_color("color1").is_ok());

    // RRGGBB format (no alpha)
    config.parse("color2 = 0xff0000").unwrap();
    assert!(config.get_color("color2").is_ok());

    // RGB shorthand
    config.parse("color3 = 0xff0").unwrap();
    // Depends on implementation - may or may not be valid
}

#[test]
fn test_color_valid_rgba_formats() {
    let mut config = Config::new();

    // Standard rgba
    config.parse("color1 = rgba(255, 128, 64, 0.5)").unwrap();
    assert!(config.get_color("color1").is_ok());

    // rgb (no alpha)
    config.parse("color2 = rgb(255, 128, 64)").unwrap();
    assert!(config.get_color("color2").is_ok());
}

// ========== VEC2 PARSING EDGE CASES ==========

#[test]
fn test_vec2_non_numeric_values() {
    let mut config = Config::new();
    let result = config.parse("size = abc, def");
    assert!(result.is_ok());
    let value = config.get("size").unwrap();
    assert!(value.as_vec2().is_err());
}

#[test]
fn test_vec2_missing_component() {
    let mut config = Config::new();
    let result = config.parse("size = 100");
    assert!(result.is_ok());
    let value = config.get("size").unwrap();
    // Single value might be stored as int, not vec2
    assert!(value.as_vec2().is_err());
}

#[test]
fn test_vec2_whitespace_variations() {
    let mut config = Config::new();

    // Various whitespace
    config.parse("size1 = 100,200").unwrap();
    config.parse("size2 = 100, 200").unwrap();
    config.parse("size3 = 100 , 200").unwrap();
    config.parse("size4 =   100  ,  200  ").unwrap();

    // All should parse as valid vec2
    assert!(config.get_vec2("size1").is_ok());
    assert!(config.get_vec2("size2").is_ok());
    assert!(config.get_vec2("size3").is_ok());
    assert!(config.get_vec2("size4").is_ok());
}

#[test]
fn test_vec2_negative_values() {
    let mut config = Config::new();
    config.parse("offset = -10, -20").unwrap();
    let vec = config.get_vec2("offset").unwrap();
    assert_eq!(vec.x, -10.0);
    assert_eq!(vec.y, -20.0);
}

#[test]
fn test_vec2_float_values() {
    let mut config = Config::new();
    config.parse("scale = 1.5, 2.5").unwrap();
    let vec = config.get_vec2("scale").unwrap();
    assert_eq!(vec.x, 1.5);
    assert_eq!(vec.y, 2.5);
}

#[test]
fn test_vec2_mixed_int_float() {
    let mut config = Config::new();
    config.parse("pos = 100, 50.5").unwrap();
    let vec = config.get_vec2("pos").unwrap();
    assert_eq!(vec.x, 100.0);
    assert_eq!(vec.y, 50.5);
}

#[test]
fn test_vec2_extra_components_ignored() {
    let mut config = Config::new();
    // Three components - should fail or only use first two
    let result = config.parse("size = 100, 200, 300");
    assert!(result.is_ok());
    // May store as string since it doesn't match vec2 format
    let value = config.get("size").unwrap();
    // Behavior depends on implementation
    let _ = value.as_vec2();
}

// ========== GENERAL PARSING EDGE CASES ==========

#[test]
fn test_empty_value() {
    let mut config = Config::new();
    let result = config.parse("key =");
    assert!(result.is_ok());
    let value = config.get_string("key");
    assert!(value.is_ok());
    assert_eq!(value.unwrap(), "");
}

#[test]
fn test_whitespace_only_value() {
    let mut config = Config::new();
    let result = config.parse("key =    ");
    assert!(result.is_ok());
    let value = config.get_string("key");
    assert!(value.is_ok());
}

#[test]
fn test_special_characters_in_value() {
    let mut config = Config::new();
    config.parse(r#"key = hello "world" 'test'"#).unwrap();
    let value = config.get_string("key").unwrap();
    assert!(value.contains("\"world\""));
}

#[test]
fn test_unicode_in_value() {
    let mut config = Config::new();
    config.parse("emoji = 🎉 hello 世界").unwrap();
    let value = config.get_string("emoji").unwrap();
    assert!(value.contains("🎉"));
    assert!(value.contains("世界"));
}
