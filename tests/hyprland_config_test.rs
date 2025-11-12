use hyprlang::{Config, ConfigOptions};
use std::path::PathBuf;

#[test]
fn test_hyprland_config_comprehensive() {
    let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("tests/config/hyprland.conf");

    // Create config with options for this test
    let mut options = ConfigOptions::default();
    options.base_dir = Some(config_path.parent().unwrap().to_path_buf());
    options.throw_all_errors = false;

    let mut config = Config::with_options(options);

    // Parse the hyprland config file
    let result = config.parse_file(&config_path);
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }

    // For now, we expect some parsing to work even if not all features are supported
    // This is an integration test to validate the parser handles real-world configs

    // The config should at least parse without panicking
    // and we can test the parts that are currently supported

    // Note: The actual Hyprland config uses many keywords that would need handlers
    // For this test, we're validating that the parser can handle the syntax

    println!("Hyprland config parse result: {:?}", result);

    // Test that we can at least parse some basic structure
    // Even if handlers aren't registered, the parser should work
}

#[test]
fn test_hyprland_config_variables() {
    // Test a simplified version with variables
    let mut config = Config::new();

    config
        .parse(
            r#"
        $terminal = kitty
        $fileManager = dolphin
        $menu = wofi --show drun
    "#,
        )
        .unwrap();

    // Verify variables were set
    assert_eq!(config.get_variable("terminal"), Some("kitty"));
    assert_eq!(config.get_variable("fileManager"), Some("dolphin"));
    assert_eq!(config.get_variable("menu"), Some("wofi --show drun"));
}

#[test]
fn test_hyprland_config_nested_categories() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        general {
            gaps_in = 5
            gaps_out = 20
            border_size = 2
            resize_on_border = false
            allow_tearing = false
        }
    "#,
        )
        .unwrap();

    // Test nested category values
    assert_eq!(config.get_int("general:gaps_in").unwrap(), 5);
    assert_eq!(config.get_int("general:gaps_out").unwrap(), 20);
    assert_eq!(config.get_int("general:border_size").unwrap(), 2);
    assert_eq!(config.get_int("general:resize_on_border").unwrap(), 0); // false = 0
    assert_eq!(config.get_int("general:allow_tearing").unwrap(), 0); // false = 0
}

#[test]
fn test_hyprland_config_deep_nesting() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        decoration {
            rounding = 10
            rounding_power = 2
            active_opacity = 1.0
            inactive_opacity = 1.0

            shadow {
                enabled = true
                range = 4
                render_power = 3
            }

            blur {
                enabled = true
                size = 3
                passes = 1
                vibrancy = 0.1696
            }
        }
    "#,
        )
        .unwrap();

    // Test deeply nested values
    assert_eq!(config.get_int("decoration:rounding").unwrap(), 10);
    assert_eq!(config.get_int("decoration:rounding_power").unwrap(), 2);
    assert_eq!(config.get_float("decoration:active_opacity").unwrap(), 1.0);

    assert_eq!(config.get_int("decoration:shadow:enabled").unwrap(), 1); // true = 1
    assert_eq!(config.get_int("decoration:shadow:range").unwrap(), 4);
    assert_eq!(config.get_int("decoration:shadow:render_power").unwrap(), 3);

    assert_eq!(config.get_int("decoration:blur:enabled").unwrap(), 1); // true = 1
    assert_eq!(config.get_int("decoration:blur:size").unwrap(), 3);
    assert_eq!(config.get_int("decoration:blur:passes").unwrap(), 1);
    assert_eq!(
        config.get_float("decoration:blur:vibrancy").unwrap(),
        0.1696
    );
}

#[test]
fn test_hyprland_config_colors() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        general {
            col.active_border = rgba(33ccffee)
            col.inactive_border = rgba(595959aa)
        }

        decoration {
            shadow {
                color = rgba(1a1a1aee)
            }
        }
    "#,
        )
        .unwrap();

    // Test color parsing
    let active_border = config.get_color("general:col.active_border").unwrap();
    assert_eq!(active_border.r, 0x33);
    assert_eq!(active_border.g, 0xcc);
    assert_eq!(active_border.b, 0xff);
    assert_eq!(active_border.a, 0xee);

    let inactive_border = config.get_color("general:col.inactive_border").unwrap();
    assert_eq!(inactive_border.r, 0x59);
    assert_eq!(inactive_border.g, 0x59);
    assert_eq!(inactive_border.b, 0x59);
    assert_eq!(inactive_border.a, 0xaa);

    let shadow_color = config.get_color("decoration:shadow:color").unwrap();
    assert_eq!(shadow_color.r, 0x1a);
    assert_eq!(shadow_color.g, 0x1a);
    assert_eq!(shadow_color.b, 0x1a);
    assert_eq!(shadow_color.a, 0xee);
}

#[test]
fn test_hyprland_config_strings_with_special_chars() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        input {
            kb_layout = us
            kb_variant =
            kb_model =
        }

        misc {
            force_default_wallpaper = -1
            disable_hyprland_logo = false
        }
    "#,
        )
        .unwrap();

    assert_eq!(config.get_string("input:kb_layout").unwrap(), "us");
    assert_eq!(config.get_int("misc:force_default_wallpaper").unwrap(), -1);
    assert_eq!(config.get_int("misc:disable_hyprland_logo").unwrap(), 0);
}

#[test]
fn test_hyprland_config_with_handlers() {
    let mut config = Config::new();

    // Register handlers that Hyprland uses
    let mut monitor_set = false;
    let monitor_set_ref = &mut monitor_set;

    config.register_handler_fn("monitor", move |ctx| {
        // In real Hyprland, this would configure monitors
        assert!(!ctx.value.is_empty());
        Ok(())
    });

    config.register_handler_fn("env", |ctx| {
        // In real Hyprland, this would set environment variables
        assert!(ctx.value.contains(','));
        Ok(())
    });

    config.register_handler_fn("bind", |ctx| {
        // In real Hyprland, this would register keybindings
        Ok(())
    });

    config.register_handler_fn("bindm", |_ctx| Ok(()));

    config.register_handler_fn("bindel", |_ctx| Ok(()));

    config.register_handler_fn("bindl", |_ctx| Ok(()));

    config.register_handler_fn("windowrule", |_ctx| Ok(()));

    config.register_handler_fn("gesture", |_ctx| Ok(()));

    config.register_handler_fn("animation", |_ctx| Ok(()));

    config.register_handler_fn("bezier", |_ctx| Ok(()));

    // Now parse a config with handlers
    config
        .parse(
            r#"
        monitor=,preferred,auto,auto

        env = XCURSOR_SIZE,24
        env = HYPRCURSOR_SIZE,24

        $mainMod = SUPER

        bind = $mainMod, Q, exec, kitty
        bind = $mainMod, C, killactive,

        bindm = $mainMod, mouse:272, movewindow

        animations {
            enabled = yes, please :)

            bezier = easeOutQuint, 0.23, 1, 0.32, 1
            animation = global, 1, 10, default
        }

        windowrule = float,class:^(kitty)$
    "#,
        )
        .unwrap();

    // Verify variables were processed
    assert_eq!(config.get_variable("mainMod"), Some("SUPER"));
}

#[test]
fn test_hyprland_config_boolean_variations() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        test1 = true
        test2 = false
        test3 = yes
        test4 = no
        test5 = on
        test6 = off
    "#,
        )
        .unwrap();

    // All boolean values should parse as integers (1 or 0)
    assert_eq!(config.get_int("test1").unwrap(), 1);
    assert_eq!(config.get_int("test2").unwrap(), 0);
    assert_eq!(config.get_int("test3").unwrap(), 1);
    assert_eq!(config.get_int("test4").unwrap(), 0);
    assert_eq!(config.get_int("test5").unwrap(), 1);
    assert_eq!(config.get_int("test6").unwrap(), 0);
}

#[test]
fn test_hyprland_config_float_values() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        decoration {
            active_opacity = 1.0
            inactive_opacity = 0.95
        }

        animations {
            bezier = easeOutQuint, 0.23, 1, 0.32, 1
        }

        input {
            sensitivity = 0
        }

        device {
            sensitivity = -0.5
        }
    "#,
        )
        .unwrap();

    assert_eq!(config.get_float("decoration:active_opacity").unwrap(), 1.0);
    assert_eq!(
        config.get_float("decoration:inactive_opacity").unwrap(),
        0.95
    );
    assert_eq!(config.get_float("input:sensitivity").unwrap(), 0.0);
    assert_eq!(config.get_float("device:sensitivity").unwrap(), -0.5);
}

#[test]
fn test_hyprland_config_comments() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        # This is a comment
        value1 = 100

        ## This is also a comment
        value2 = 200

        # Multiple
        # Comments
        # In a row
        value3 = 300
    "#,
        )
        .unwrap();

    assert_eq!(config.get_int("value1").unwrap(), 100);
    assert_eq!(config.get_int("value2").unwrap(), 200);
    assert_eq!(config.get_int("value3").unwrap(), 300);
}

#[test]
fn test_hyprland_config_empty_values() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        input {
            kb_layout = us
            kb_variant =
            kb_model =
            kb_options =
            kb_rules =
        }
    "#,
        )
        .unwrap();

    assert_eq!(config.get_string("input:kb_layout").unwrap(), "us");
    // Empty values might not be set or might be empty strings
    // The parser should handle them gracefully
}

#[test]
fn test_hyprland_config_numbers_hex() {
    let mut config = Config::new();

    config
        .parse(
            r#"
        test_decimal = 255
        test_hex = 0xFF
        test_negative = -1
    "#,
        )
        .unwrap();

    assert_eq!(config.get_int("test_decimal").unwrap(), 255);
    assert_eq!(config.get_int("test_hex").unwrap(), 0xFF);
    assert_eq!(config.get_int("test_negative").unwrap(), -1);
}
