#![cfg(feature = "hyprland")]

use hyprlang::Hyprland;

#[test]
fn test_basic_windowrule_v3() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[float-terminals] {
            match:class = ^(kitty|alacritty)$
            match:floating = false

            float = true
            size = 800 600
            center = true
        }
    "#,
    )
    .unwrap();

    // Check that the windowrule was registered
    let names = hypr.windowrule_names();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "float-terminals");

    // Get the specific rule
    let rule = hypr.get_windowrule("float-terminals").unwrap();

    // Check match properties
    assert_eq!(
        rule.get_string("match:class").unwrap(),
        "^(kitty|alacritty)$"
    );
    assert_eq!(rule.get_int("match:floating").unwrap(), 0); // false = 0

    // Check effect properties
    assert_eq!(rule.get_int("float").unwrap(), 1); // true = 1
    assert_eq!(rule.get_string("size").unwrap(), "800 600");
    assert_eq!(rule.get_int("center").unwrap(), 1); // true = 1
}

#[test]
fn test_multiple_windowrules() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[float-rule] {
            match:class = ^(kitty)$
            float = true
        }

        windowrule[opacity-rule] {
            match:class = .*
            opacity = 0.95
            rounding = 10
        }
    "#,
    )
    .unwrap();

    let names = hypr.windowrule_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"float-rule".to_string()));
    assert!(names.contains(&"opacity-rule".to_string()));

    let float_rule = hypr.get_windowrule("float-rule").unwrap();
    assert_eq!(float_rule.get_string("match:class").unwrap(), "^(kitty)$");
    assert_eq!(float_rule.get_int("float").unwrap(), 1); // true = 1

    let opacity_rule = hypr.get_windowrule("opacity-rule").unwrap();
    assert_eq!(opacity_rule.get_string("match:class").unwrap(), ".*");
    assert_eq!(opacity_rule.get_float("opacity").unwrap(), 0.95);
    assert_eq!(opacity_rule.get_int("rounding").unwrap(), 10);
}

#[test]
fn test_layerrule_v2() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        layerrule[blur-waybar] {
            match:namespace = waybar
            blur = true
            ignorealpha = 0.5
        }
    "#,
    )
    .unwrap();

    let names = hypr.layerrule_names();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "blur-waybar");

    let rule = hypr.get_layerrule("blur-waybar").unwrap();
    assert_eq!(rule.get_string("match:namespace").unwrap(), "waybar");
    assert_eq!(rule.get_int("blur").unwrap(), 1); // true = 1
    assert_eq!(rule.get_float("ignorealpha").unwrap(), 0.5);
}

#[test]
fn test_windowrule_with_complex_properties() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[complex-rule] {
            match:title = ^(.*Firefox.*)$
            match:xwayland = true

            move = 100 200
            size = 1920 1080
            border_color = rgba(33ccffee)
            border_size = 3
            rounding = 15
            opacity = 0.9
            no_blur = false
            no_shadow = false
            suppressevent = maximize fullscreen
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("complex-rule").unwrap();

    // Match properties
    assert_eq!(rule.get_string("match:title").unwrap(), "^(.*Firefox.*)$");
    assert_eq!(rule.get_int("match:xwayland").unwrap(), 1); // true = 1

    // Effect properties
    assert_eq!(rule.get_string("move").unwrap(), "100 200");
    assert_eq!(rule.get_string("size").unwrap(), "1920 1080");
    let color = rule.get_color("border_color").unwrap();
    assert_eq!(color.r, 0x33);
    assert_eq!(color.g, 0xcc);
    assert_eq!(color.b, 0xff);
    assert_eq!(color.a, 0xee);
    assert_eq!(rule.get_int("border_size").unwrap(), 3);
    assert_eq!(rule.get_int("rounding").unwrap(), 15);
    assert_eq!(rule.get_float("opacity").unwrap(), 0.9);
    assert_eq!(rule.get_int("no_blur").unwrap(), 0); // false = 0
    assert_eq!(rule.get_int("no_shadow").unwrap(), 0); // false = 0
    assert_eq!(
        rule.get_string("suppressevent").unwrap(),
        "maximize fullscreen"
    );
}

#[test]
fn test_backward_compat_handler_syntax() {
    let mut hypr = Hyprland::new();

    // Old v2 handler syntax should still be accepted
    hypr.parse(
        r#"
        windowrulev2 = float, class:^(kitty)$
        windowrulev2 = size 800 600, class:^(kitty)$
    "#,
    )
    .unwrap();

    let v2_rules = hypr.all_windowrulesv2();
    assert_eq!(v2_rules.len(), 2);
}

#[test]
fn test_mixed_v2_and_v3_syntax() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        # Old v2 syntax (deprecated)
        windowrulev2 = float, class:^(old)$

        # New v3 syntax
        windowrule[new-rule] {
            match:class = ^(new)$
            float = test_yes
        }
    "#,
    )
    .unwrap();

    // Both should work
    let v2_rules = hypr.all_windowrulesv2();
    assert_eq!(v2_rules.len(), 1);

    let v3_names = hypr.windowrule_names();
    assert_eq!(v3_names.len(), 1);
    assert_eq!(v3_names[0], "new-rule");
}

#[test]
fn test_windowrule_not_found() {
    let hypr = Hyprland::new();

    let result = hypr.get_windowrule("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_minimal_windowrule() {
    let mut hypr = Hyprland::new();

    // Special categories need at least one property
    hypr.parse(
        r#"
        windowrule[minimal-rule] {
            float = enabled
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("minimal-rule").unwrap();
    // Should have the one property
    assert_eq!(rule.get_string("float").unwrap(), "enabled");
}

#[test]
fn test_windowrule_default_values() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[test-defaults] {
            match:class = test
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("test-defaults").unwrap();

    // The 'enable' property should default to 1
    assert_eq!(rule.get_int("enable").unwrap(), 1);

    // All registered match properties should exist with empty string defaults
    assert_eq!(rule.get_string("match:title").unwrap(), "");
    assert_eq!(rule.get_string("match:initial_class").unwrap(), "");
    assert_eq!(rule.get_string("match:tag").unwrap(), "");

    // All registered effect properties should exist with empty string defaults
    assert_eq!(rule.get_string("opacity").unwrap(), "");
    assert_eq!(rule.get_string("rounding").unwrap(), "");
    assert_eq!(rule.get_string("border_color").unwrap(), "");
}

#[test]
fn test_all_windowrule_match_properties() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[all-match-props] {
            match:class = test-class
            match:title = test-title
            match:initial_class = test-init-class
            match:initial_title = test-init-title
            match:floating = true
            match:tag = test-tag
            match:xwayland = true
            match:fullscreen = false
            match:pinned = true
            match:focus = false
            match:group = true
            match:modal = false
            match:fullscreenstate_internal = 1
            match:fullscreenstate_client = 2
            match:on_workspace = 5
            match:content = normal
            match:xdg_tag = test-xdg
            match:namespace = test-namespace
            match:exec_token = test-token
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("all-match-props").unwrap();

    // Verify all 19 match properties can be accessed
    assert_eq!(rule.get_string("match:class").unwrap(), "test-class");
    assert_eq!(rule.get_string("match:title").unwrap(), "test-title");
    assert_eq!(
        rule.get_string("match:initial_class").unwrap(),
        "test-init-class"
    );
    assert_eq!(
        rule.get_string("match:initial_title").unwrap(),
        "test-init-title"
    );
    assert_eq!(rule.get_int("match:floating").unwrap(), 1);
    assert_eq!(rule.get_string("match:tag").unwrap(), "test-tag");
    assert_eq!(rule.get_int("match:xwayland").unwrap(), 1);
    assert_eq!(rule.get_int("match:fullscreen").unwrap(), 0);
    assert_eq!(rule.get_int("match:pinned").unwrap(), 1);
    assert_eq!(rule.get_int("match:focus").unwrap(), 0);
    assert_eq!(rule.get_int("match:group").unwrap(), 1);
    assert_eq!(rule.get_int("match:modal").unwrap(), 0);
    assert_eq!(rule.get_int("match:fullscreenstate_internal").unwrap(), 1);
    assert_eq!(rule.get_int("match:fullscreenstate_client").unwrap(), 2);
    assert_eq!(rule.get_int("match:on_workspace").unwrap(), 5);
    assert_eq!(rule.get_string("match:content").unwrap(), "normal");
    assert_eq!(rule.get_string("match:xdg_tag").unwrap(), "test-xdg");
    assert_eq!(
        rule.get_string("match:namespace").unwrap(),
        "test-namespace"
    );
    assert_eq!(rule.get_string("match:exec_token").unwrap(), "test-token");
}

#[test]
fn test_windowrule_effect_properties_static() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[static-effects] {
            match:class = test

            # Static effects (applied once)
            float = true
            tile = false
            fullscreen = true
            maximize = false
            fullscreenstate = 1
            move = 100 200
            size = 1920 1080
            center = true
            pseudo = false
            monitor = HDMI_A_1
            workspace = 5
            noinitialfocus = true
            pin = false
            group = lock
            suppressevent = maximize
            content = browser
            noclosefor = 5
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("static-effects").unwrap();

    assert_eq!(rule.get_int("float").unwrap(), 1);
    assert_eq!(rule.get_int("tile").unwrap(), 0);
    assert_eq!(rule.get_int("fullscreen").unwrap(), 1);
    assert_eq!(rule.get_int("maximize").unwrap(), 0);
    assert_eq!(rule.get_int("fullscreenstate").unwrap(), 1);
    assert_eq!(rule.get_string("move").unwrap(), "100 200");
    assert_eq!(rule.get_string("size").unwrap(), "1920 1080");
    assert_eq!(rule.get_int("center").unwrap(), 1);
    assert_eq!(rule.get_int("pseudo").unwrap(), 0);
    assert_eq!(rule.get_string("monitor").unwrap(), "HDMI_A_1");
    assert_eq!(rule.get_int("workspace").unwrap(), 5);
    assert_eq!(rule.get_int("noinitialfocus").unwrap(), 1);
    assert_eq!(rule.get_int("pin").unwrap(), 0);
    assert_eq!(rule.get_string("group").unwrap(), "lock");
    assert_eq!(rule.get_string("suppressevent").unwrap(), "maximize");
    assert_eq!(rule.get_string("content").unwrap(), "browser");
    assert_eq!(rule.get_int("noclosefor").unwrap(), 5);
}

#[test]
fn test_windowrule_effect_properties_dynamic() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[dynamic-effects] {
            match:class = test

            # Dynamic effects (continuously applied)
            rounding = 15
            rounding_power = 2
            persistent_size = true
            animation = slide
            border_color = rgba(ff0000ff)
            idle_inhibit = focus
            opacity = 0.95
            tag = important
            max_size = 1920 1080
            min_size = 800 600
            border_size = 3
            allows_input = true
            dim_around = false
            decorate = true
            focus_on_activate = false
            keep_aspect_ratio = true
            nearest_neighbor = false
            no_anim = false
            no_blur = false
            no_dim = true
            no_focus = false
            no_follow_mouse = false
            no_max_size = true
            no_shadow = false
            no_shortcuts_inhibit = false
            opaque = true
            force_rgbx = false
            sync_fullscreen = true
            immediate = false
            xray = 1
            render_unfocused = true
            no_screen_share = false
            no_vrr = false
            scroll_mouse = 0.5
            scroll_touchpad = 1.0
            stay_focused = true
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("dynamic-effects").unwrap();

    assert_eq!(rule.get_int("rounding").unwrap(), 15);
    assert_eq!(rule.get_int("rounding_power").unwrap(), 2);
    assert_eq!(rule.get_int("persistent_size").unwrap(), 1);
    assert_eq!(rule.get_string("animation").unwrap(), "slide");
    let color = rule.get_color("border_color").unwrap();
    assert_eq!(color.r, 0xff);
    assert_eq!(rule.get_string("idle_inhibit").unwrap(), "focus");
    assert_eq!(rule.get_float("opacity").unwrap(), 0.95);
    assert_eq!(rule.get_string("tag").unwrap(), "important");
    assert_eq!(rule.get_string("max_size").unwrap(), "1920 1080");
    assert_eq!(rule.get_string("min_size").unwrap(), "800 600");
    assert_eq!(rule.get_int("border_size").unwrap(), 3);
    assert_eq!(rule.get_int("allows_input").unwrap(), 1);
    assert_eq!(rule.get_int("dim_around").unwrap(), 0);
    assert_eq!(rule.get_int("decorate").unwrap(), 1);
}

#[test]
fn test_windowrule_property_aliases() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[aliases-test] {
            match:class = test

            # Test that both forms of aliases work
            border_color = rgba(ff0000ff)
            bordercolor = rgba(00ff00ff)

            idle_inhibit = focus
            idleinhibit = always

            max_size = 1920 1080
            maxsize = 1280 720
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("aliases-test").unwrap();

    // Both aliases should be accessible
    // The last one set should win
    assert!(rule.get("border_color").is_ok());
    assert!(rule.get("bordercolor").is_ok());
    assert!(rule.get("idle_inhibit").is_ok());
    assert!(rule.get("idleinhibit").is_ok());
    assert!(rule.get("max_size").is_ok());
    assert!(rule.get("maxsize").is_ok());
}

#[test]
fn test_all_layerrule_properties() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        layerrule[all-props] {
            # Match properties
            match:namespace = waybar
            match:address = address123
            match:class = test-class
            match:title = test-title
            match:monitor = HDMI_A_1
            match:layer = top

            # Effect properties
            blur = true
            ignorealpha = 0.5
            ignorezero = false
            animation = slide
            noanim = false
            xray = 1
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_layerrule("all-props").unwrap();

    // Match properties
    assert_eq!(rule.get_string("match:namespace").unwrap(), "waybar");
    assert_eq!(rule.get_string("match:address").unwrap(), "address123");
    assert_eq!(rule.get_string("match:class").unwrap(), "test-class");
    assert_eq!(rule.get_string("match:title").unwrap(), "test-title");
    assert_eq!(rule.get_string("match:monitor").unwrap(), "HDMI_A_1");
    assert_eq!(rule.get_string("match:layer").unwrap(), "top");

    // Effect properties
    assert_eq!(rule.get_int("blur").unwrap(), 1);
    assert_eq!(rule.get_float("ignorealpha").unwrap(), 0.5);
    assert_eq!(rule.get_int("ignorezero").unwrap(), 0);
    assert_eq!(rule.get_string("animation").unwrap(), "slide");
    assert_eq!(rule.get_int("noanim").unwrap(), 0);
    assert_eq!(rule.get_int("xray").unwrap(), 1);
}

#[test]
fn test_layerrule_default_values() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        layerrule[defaults-test] {
            match:namespace = test
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_layerrule("defaults-test").unwrap();

    // The 'enable' property should default to 1
    assert_eq!(rule.get_int("enable").unwrap(), 1);

    // All other registered properties should have empty string defaults
    assert_eq!(rule.get_string("match:class").unwrap(), "");
    assert_eq!(rule.get_string("blur").unwrap(), "");
    assert_eq!(rule.get_string("animation").unwrap(), "");
}

// ==================== Hyprland 0.53.0 Tests ====================

#[test]
fn test_bindu_handler() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        bindu = SUPER, Y, exec, kitty
        bindu = SUPER SHIFT, Y, killactive
    "#,
    )
    .unwrap();

    let binds = hypr.all_bindu();
    assert_eq!(binds.len(), 2);
    assert_eq!(binds[0], "SUPER, Y, exec, kitty");
    assert_eq!(binds[1], "SUPER SHIFT, Y, killactive");
}

#[test]
fn test_new_config_accessors() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        general {
            locale = en_US
        }

        quirks {
            prefer_hdr = 2
        }

        cursor {
            hide_on_tablet = true
        }

        group {
            groupbar {
                blur = true
            }
        }
    "#,
    )
    .unwrap();

    assert_eq!(hypr.general_locale().unwrap(), "en_US");
    assert_eq!(hypr.quirks_prefer_hdr().unwrap(), 2);
    assert!(hypr.cursor_hide_on_tablet().unwrap());
    assert!(hypr.group_groupbar_blur().unwrap());
}

#[test]
fn test_windowrule_match_property_aliases() {
    let mut hypr = Hyprland::new();

    // Test Hyprland v3 naming with aliases
    hypr.parse(
        r#"
        windowrule[v3-aliases] {
            match:float = true
            match:pin = false
            match:workspace = 5
            match:fullscreen_state_internal = 1
            match:fullscreen_state_client = 2
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("v3-aliases").unwrap();

    // Test the Hyprland v3 naming aliases work
    assert_eq!(rule.get_int("match:float").unwrap(), 1);
    assert_eq!(rule.get_int("match:pin").unwrap(), 0);
    assert_eq!(rule.get_int("match:workspace").unwrap(), 5);
    assert_eq!(rule.get_int("match:fullscreen_state_internal").unwrap(), 1);
    assert_eq!(rule.get_int("match:fullscreen_state_client").unwrap(), 2);
}

#[test]
fn test_windowrule_effect_property_aliases() {
    let mut hypr = Hyprland::new();

    // Test v3 naming with underscore aliases
    hypr.parse(
        r#"
        windowrule[v3-effect-aliases] {
            match:class = test
            fullscreen_state = 2
            no_initial_focus = true
            suppress_event = maximize
            no_close_for = 5000
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("v3-effect-aliases").unwrap();

    assert_eq!(rule.get_int("fullscreen_state").unwrap(), 2);
    assert_eq!(rule.get_int("no_initial_focus").unwrap(), 1);
    assert_eq!(rule.get_string("suppress_event").unwrap(), "maximize");
    assert_eq!(rule.get_int("no_close_for").unwrap(), 5000);
}

#[test]
fn test_layerrule_new_effects() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        layerrule[new-effects] {
            match:namespace = test-layer
            blur_popups = true
            dim_around = 0.5
            order = 10
            above_lock = true
            no_screen_share = true
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_layerrule("new-effects").unwrap();

    assert_eq!(rule.get_int("blur_popups").unwrap(), 1);
    assert_eq!(rule.get_float("dim_around").unwrap(), 0.5);
    assert_eq!(rule.get_int("order").unwrap(), 10);
    assert_eq!(rule.get_int("above_lock").unwrap(), 1);
    assert_eq!(rule.get_int("no_screen_share").unwrap(), 1);
}

#[test]
fn test_layerrule_effect_aliases() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        layerrule[effect-aliases] {
            match:namespace = test
            ignore_alpha = 0.3
            no_anim = true
            noscreenshare = true
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_layerrule("effect-aliases").unwrap();

    // Test underscore aliases work
    assert_eq!(rule.get_float("ignore_alpha").unwrap(), 0.3);
    assert_eq!(rule.get_int("no_anim").unwrap(), 1);
    assert_eq!(rule.get_int("noscreenshare").unwrap(), 1);
}
