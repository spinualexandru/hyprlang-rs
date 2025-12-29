//! Hyprland parity tests - based on real Hyprland test configs from hyprtester
//!
//! These tests ensure hyprlang-rs can parse actual Hyprland configuration patterns
//! from the official test suite at: Hyprland/hyprtester/test.conf
//!
//! Note: hyprlang-rs uses `windowrule[name] { ... }` syntax (bracket-keyed),
//! while Hyprland's test.conf uses `windowrule { name = rule-name ... }`.
//! Both are equivalent; these tests use hyprlang-rs's bracket syntax.

#![cfg(feature = "hyprland")]

use hyprlang::Hyprland;

/// Test parsing the exact windowrule v3 syntax from Hyprland's test.conf
#[test]
fn test_hyprland_test_conf_suppress_maximize() {
    let mut hypr = Hyprland::new();

    // From hyprtester/test.conf lines 321-327
    // Using hyprlang-rs bracket syntax
    hypr.parse(
        r#"
        windowrule[suppress-maximize-events] {
            # Ignore maximize requests from apps. You'll probably like this.
            match:class = .*

            suppress_event = maximize
        }
    "#,
    )
    .unwrap();

    let names = hypr.windowrule_names();
    assert!(names.contains(&"suppress-maximize-events".to_string()));

    let rule = hypr.get_windowrule("suppress-maximize-events").unwrap();
    assert_eq!(rule.get_string("match:class").unwrap(), ".*");
    assert_eq!(rule.get_string("suppress_event").unwrap(), "maximize");
}

/// Test complex multi-condition windowrule from Hyprland's test.conf
#[test]
fn test_hyprland_test_conf_fix_xwayland_drags() {
    let mut hypr = Hyprland::new();

    // From hyprtester/test.conf lines 329-340
    hypr.parse(
        r#"
        windowrule[fix-xwayland-drags] {
            # Fix some dragging issues with XWayland
            match:class = ^$
            match:title = ^$
            match:xwayland = true
            match:float = true
            match:fullscreen = false
            match:pin = false

            no_focus = true
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("fix-xwayland-drags").unwrap();

    // Verify all match conditions
    assert_eq!(rule.get_string("match:class").unwrap(), "^$");
    assert_eq!(rule.get_string("match:title").unwrap(), "^$");
    assert_eq!(rule.get_int("match:xwayland").unwrap(), 1);
    assert_eq!(rule.get_int("match:float").unwrap(), 1);
    assert_eq!(rule.get_int("match:fullscreen").unwrap(), 0);
    assert_eq!(rule.get_int("match:pin").unwrap(), 0);

    // Verify effect
    assert_eq!(rule.get_int("no_focus").unwrap(), 1);
}

/// Test smart gaps windowrule with workspace expression
#[test]
fn test_hyprland_test_conf_smart_gaps() {
    let mut hypr = Hyprland::new();

    // From hyprtester/test.conf lines 345-361
    hypr.parse(
        r#"
        windowrule[smart-gaps-1] {
            match:float = false
            match:workspace = n[s:window] w[tv1]

            border_size = 0
            rounding = 0
        }

        windowrule[smart-gaps-2] {
            match:float = false
            match:workspace = n[s:window] f[1]

            border_size = 0
            rounding = 0
        }
    "#,
    )
    .unwrap();

    let rule1 = hypr.get_windowrule("smart-gaps-1").unwrap();
    assert_eq!(rule1.get_int("match:float").unwrap(), 0);
    // The workspace expression is stored as-is
    assert_eq!(
        rule1.get_string("match:workspace").unwrap(),
        "n[s:window] w[tv1]"
    );
    assert_eq!(rule1.get_int("border_size").unwrap(), 0);
    assert_eq!(rule1.get_int("rounding").unwrap(), 0);

    let rule2 = hypr.get_windowrule("smart-gaps-2").unwrap();
    assert_eq!(
        rule2.get_string("match:workspace").unwrap(),
        "n[s:window] f[1]"
    );
}

/// Test windowrule with basic float, size, and pin
#[test]
fn test_hyprland_test_conf_wr_kitty_stuff() {
    let mut hypr = Hyprland::new();

    // From hyprtester/test.conf lines 363-370
    hypr.parse(
        r#"
        windowrule[wr-kitty-stuff] {
            match:class = wr_kitty

            float = true
            size = 200 200
            pin = false
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("wr-kitty-stuff").unwrap();
    assert_eq!(rule.get_string("match:class").unwrap(), "wr_kitty");
    assert_eq!(rule.get_int("float").unwrap(), 1);
    assert_eq!(rule.get_string("size").unwrap(), "200 200");
    assert_eq!(rule.get_int("pin").unwrap(), 0);
}

/// Test tag-based matching and tag assignment
#[test]
fn test_hyprland_test_conf_tags() {
    let mut hypr = Hyprland::new();

    // From hyprtester/test.conf lines 372-384
    hypr.parse(
        r#"
        windowrule[tagged-kitty-floats] {
            match:tag = tag_kitty

            float = true
        }

        windowrule[static-kitty-tag] {
            match:class = tag_kitty

            tag = +tag_kitty
        }
    "#,
    )
    .unwrap();

    let float_rule = hypr.get_windowrule("tagged-kitty-floats").unwrap();
    assert_eq!(float_rule.get_string("match:tag").unwrap(), "tag_kitty");
    assert_eq!(float_rule.get_int("float").unwrap(), 1);

    let tag_rule = hypr.get_windowrule("static-kitty-tag").unwrap();
    assert_eq!(tag_rule.get_string("match:class").unwrap(), "tag_kitty");
    // Tag with + prefix for adding
    assert_eq!(tag_rule.get_string("tag").unwrap(), "+tag_kitty");
}

/// Test windowrule with opacity override (from window.cpp tests)
#[test]
fn test_hyprland_opacity_override() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp line 694
    hypr.parse(
        r#"
        windowrule[wr-kitty-stuff] {
            match:class = wr_kitty
            opacity = 0.5 0.5 override
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("wr-kitty-stuff").unwrap();
    // Opacity with override is stored as full string
    assert_eq!(rule.get_string("opacity").unwrap(), "0.5 0.5 override");
}

/// Test minsize/maxsize rules (from window.cpp tests)
#[test]
fn test_hyprland_minmax_size_rules() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp lines 583-585
    hypr.parse(
        r#"
        windowrule[kitty-max-rule] {
            match:class = kitty_maxsize
            max_size = 1500 500
            min_size = 1200 500
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("kitty-max-rule").unwrap();
    assert_eq!(rule.get_string("match:class").unwrap(), "kitty_maxsize");
    assert_eq!(rule.get_string("max_size").unwrap(), "1500 500");
    assert_eq!(rule.get_string("min_size").unwrap(), "1200 500");
}

/// Test workspace assignment to special workspace (from window.cpp tests)
#[test]
fn test_hyprland_special_workspace_rule() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp lines 701-702
    hypr.parse(
        r#"
        windowrule[special-magic-kitty] {
            match:class = magic_kitty
            workspace = special:magic
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("special-magic-kitty").unwrap();
    assert_eq!(rule.get_string("match:class").unwrap(), "magic_kitty");
    assert_eq!(rule.get_string("workspace").unwrap(), "special:magic");
}

/// Test persistent_size rule (from window.cpp tests)
#[test]
fn test_hyprland_persistent_size() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp line 746
    hypr.parse(
        r#"
        windowrule[persistent-float] {
            match:class = persistent_size_kitty
            persistent_size = true
            float = true
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("persistent-float").unwrap();
    assert_eq!(
        rule.get_string("match:class").unwrap(),
        "persistent_size_kitty"
    );
    assert_eq!(rule.get_int("persistent_size").unwrap(), 1);
    assert_eq!(rule.get_int("float").unwrap(), 1);
}

/// Test expression-based rules (from window.cpp tests)
#[test]
fn test_hyprland_expression_rules() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp line 802
    // Note: hyprlang-rs parses these as strings, actual evaluation is done by Hyprland
    hypr.parse(
        r#"
        windowrule[expr-rule] {
            match:class = expr_kitty
            float = true
            size = monitor_w*0.5 monitor_h*0.5
            move = 20+(monitor_w*0.1) monitor_h*0.5
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("expr-rule").unwrap();
    assert_eq!(rule.get_string("match:class").unwrap(), "expr_kitty");
    // "yes" gets parsed as boolean true (1)
    assert_eq!(rule.get_int("float").unwrap(), 1);
    assert_eq!(rule.get_string("size").unwrap(), "monitor_w*0.5 monitor_h*0.5");
    assert_eq!(
        rule.get_string("move").unwrap(),
        "20+(monitor_w*0.1) monitor_h*0.5"
    );
}

/// Test dynamic match:float rule (from window.cpp tests)
#[test]
fn test_hyprland_dynamic_float_match() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp line 774
    hypr.parse(
        r#"
        windowrule[float-border] {
            match:float = true
            border_size = 10
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("float-border").unwrap();
    assert_eq!(rule.get_int("match:float").unwrap(), 1);
    assert_eq!(rule.get_int("border_size").unwrap(), 10);
}

/// Test group rules with workspace expressions (from window.cpp tests)
#[test]
fn test_hyprland_group_workspace_rules() {
    let mut hypr = Hyprland::new();

    // From hyprtester/src/tests/main/window.cpp lines 144-145
    hypr.parse(
        r#"
        windowrule[ws-tv1-border] {
            match:workspace = w[tv1]
            border_size = 0
        }

        windowrule[ws-f1-border] {
            match:workspace = f[1]
            border_size = 0
        }
    "#,
    )
    .unwrap();

    let rule1 = hypr.get_windowrule("ws-tv1-border").unwrap();
    assert_eq!(rule1.get_string("match:workspace").unwrap(), "w[tv1]");
    assert_eq!(rule1.get_int("border_size").unwrap(), 0);

    let rule2 = hypr.get_windowrule("ws-f1-border").unwrap();
    assert_eq!(rule2.get_string("match:workspace").unwrap(), "f[1]");
}

/// Test parsing the full Hyprland test.conf structure
#[test]
fn test_hyprland_full_config_structure() {
    let mut hypr = Hyprland::new();

    // Simplified version of hyprtester/test.conf
    hypr.parse(
        r#"
        # Variables
        $terminal = kitty
        $mainMod = SUPER

        # Monitors
        monitor = HEADLESS-1, 1920x1080@60, auto-right, 1

        # Environment
        env = XCURSOR_SIZE, 24

        # General settings
        general {
            gaps_in = 5
            gaps_out = 20
            border_size = 2
            col.active_border = rgba(33ccffee) rgba(00ff99ee) 45deg
            col.inactive_border = rgba(595959aa)
            layout = dwindle
        }

        # Decoration
        decoration {
            rounding = 10
            rounding_power = 2

            blur {
                enabled = true
                size = 3
                passes = 1
            }
        }

        # Animations
        animations {
            enabled = 0
            bezier = easeOutQuint, 0.23, 1, 0.32, 1
            animation = global, 1, 10, default
        }

        # Device config
        device[test-mouse-1] {
            enabled = true
        }

        # Layout
        dwindle {
            pseudotile = true
            preserve_split = true
        }

        master {
            new_status = master
        }

        # Input
        input {
            kb_layout = us
            follow_mouse = 1
            sensitivity = 0
        }

        # Keybindings
        bind = $mainMod, Q, exec, $terminal
        bind = $mainMod, C, killactive,
        bindm = $mainMod, mouse:272, movewindow

        # Windowrules
        windowrule[test-rule] {
            match:class = .*
            suppress_event = maximize
        }
    "#,
    )
    .unwrap();

    // Verify various config values
    assert_eq!(hypr.general_border_size().unwrap(), 2);
    assert_eq!(hypr.general_layout().unwrap(), "dwindle");
    assert_eq!(hypr.decoration_rounding().unwrap(), 10);
    assert!(hypr.dwindle_pseudotile().unwrap());
    assert_eq!(hypr.master_new_status().unwrap(), "master");

    // Verify handlers
    let binds = hypr.all_binds();
    assert_eq!(binds.len(), 2);

    let bindm = hypr.all_bindm();
    assert_eq!(bindm.len(), 1);

    // Verify windowrule
    let rule = hypr.get_windowrule("test-rule").unwrap();
    assert_eq!(rule.get_string("suppress_event").unwrap(), "maximize");
}

/// Test that alternate bracket syntax works: windowrule[name] { ... }
#[test]
fn test_hyprland_bracket_name_syntax() {
    let mut hypr = Hyprland::new();

    hypr.parse(
        r#"
        windowrule[my-bracket-rule] {
            match:class = test
            float = true
        }
    "#,
    )
    .unwrap();

    let rule = hypr.get_windowrule("my-bracket-rule").unwrap();
    assert_eq!(rule.get_string("match:class").unwrap(), "test");
    assert_eq!(rule.get_int("float").unwrap(), 1);
}

/// Test overlapping rules (from window.cpp tests)
#[test]
fn test_hyprland_overlapping_rules() {
    let mut hypr = Hyprland::new();

    // From window.cpp lines 731-732 - rules that overlap effects but not props
    hypr.parse(
        r#"
        windowrule[class-overlap] {
            match:class = overlap_kitty
            border_size = 0
        }

        windowrule[fullscreen-overlap] {
            match:fullscreen = false
            border_size = 10
        }
    "#,
    )
    .unwrap();

    let rule1 = hypr.get_windowrule("class-overlap").unwrap();
    assert_eq!(rule1.get_string("match:class").unwrap(), "overlap_kitty");
    assert_eq!(rule1.get_int("border_size").unwrap(), 0);

    let rule2 = hypr.get_windowrule("fullscreen-overlap").unwrap();
    assert_eq!(rule2.get_int("match:fullscreen").unwrap(), 0);
    assert_eq!(rule2.get_int("border_size").unwrap(), 10);
}
