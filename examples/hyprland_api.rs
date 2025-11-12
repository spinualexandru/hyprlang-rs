//! Example demonstrating the Hyprland API wrapper
//!
//! This example shows how to use the high-level Hyprland struct which provides
//! typed access to Hyprland configuration options.
//!
//! Run with: cargo run --example hyprland_api --features hyprland

#[cfg(feature = "hyprland")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use hyprlang::Hyprland;

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║               Hyprlang - Hyprland API Example                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Create a new Hyprland configuration
    let mut hypr = Hyprland::new();

    // Parse a sample configuration
    hypr.parse(
        r#"
        # Variables
        $terminal = kitty
        $mod = SUPER

        # General settings
        general {
            border_size = 2
            gaps_in = 5
            gaps_out = 20
            layout = dwindle
            allow_tearing = false

            col.active_border = rgba(33ccffee)
            col.inactive_border = rgba(595959aa)
        }

        # Decoration settings
        decoration {
            rounding = 10
            active_opacity = 1.0
            inactive_opacity = 0.9

            blur {
                enabled = true
                size = 3
                passes = 1
            }
        }

        # Animations
        animations {
            enabled = true

            bezier = easeOutQuint, 0.23, 1, 0.32, 1
            bezier = easeInOut, 0.65, 0, 0.35, 1

            animation = windows, 1, 4.79, easeOutQuint
            animation = fade, 1, 3.03, easeInOut
            animation = border, 1, 5.39, easeOutQuint
        }

        # Input settings
        input {
            kb_layout = us
            follow_mouse = 1
            sensitivity = 0

            touchpad {
                natural_scroll = false
            }
        }

        # Dwindle layout
        dwindle {
            pseudotile = true
            preserve_split = true
        }

        # Master layout
        master {
            new_status = master
        }

        # Misc settings
        misc {
            disable_hyprland_logo = false
            force_default_wallpaper = -1
        }

        # Keybindings
        bind = $mod, Q, exec, $terminal
        bind = $mod, C, killactive
        bind = $mod, M, exit
        bind = $mod, F, togglefloating
        bind = $mod, left, movefocus, l
        bind = $mod, right, movefocus, r

        # Mouse bindings
        bindm = $mod, mouse:272, movewindow
        bindm = $mod, mouse:273, resizewindow

        # Window rules
        windowrule = float, ^(kitty)$
        windowrule = opacity 0.8, ^(kitty)$

        # Monitor configuration
        monitor = ,preferred,auto,1

        # Environment variables
        env = XCURSOR_SIZE,24
        env = QT_QPA_PLATFORM,wayland

        # Autostart
        exec-once = waybar
        exec-once = hyprpaper
    "#,
    )?;

    println!("✅ Configuration parsed successfully!\n");

    // ========== Access General Settings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                      GENERAL SETTINGS                        │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("  Border size:        {}", hypr.general_border_size()?);
    println!("  Gaps in:            {}", hypr.general_gaps_in()?);
    println!("  Gaps out:           {}", hypr.general_gaps_out()?);
    println!("  Layout:             {}", hypr.general_layout()?);
    println!("  Allow tearing:      {}", hypr.general_allow_tearing()?);

    let active_border = hypr.general_active_border_color()?;
    println!(
        "  Active border:      rgba({}, {}, {}, {})",
        active_border.r, active_border.g, active_border.b, active_border.a
    );

    let inactive_border = hypr.general_inactive_border_color()?;
    println!(
        "  Inactive border:    rgba({}, {}, {}, {})\n",
        inactive_border.r, inactive_border.g, inactive_border.b, inactive_border.a
    );

    // ========== Access Decoration Settings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    DECORATION SETTINGS                       │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("  Rounding:           {}", hypr.decoration_rounding()?);
    println!(
        "  Active opacity:     {}",
        hypr.decoration_active_opacity()?
    );
    println!(
        "  Inactive opacity:   {}",
        hypr.decoration_inactive_opacity()?
    );
    println!("  Blur enabled:       {}", hypr.decoration_blur_enabled()?);
    println!("  Blur size:          {}", hypr.decoration_blur_size()?);
    println!("  Blur passes:        {}\n", hypr.decoration_blur_passes()?);

    // ========== Access Animation Settings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    ANIMATION SETTINGS                        │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("  Animations enabled: {}\n", hypr.animations_enabled()?);

    let beziers = hypr.all_beziers();
    println!("  Bezier curves ({}):", beziers.len());
    for (i, bezier) in beziers.iter().enumerate() {
        println!("    [{}] {}", i + 1, bezier);
    }
    println!();

    let animations = hypr.all_animations();
    println!("  Animations ({}):", animations.len());
    for (i, anim) in animations.iter().enumerate() {
        println!("    [{}] {}", i + 1, anim);
    }
    println!();

    // ========== Access Input Settings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                      INPUT SETTINGS                          │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("  Keyboard layout:    {}", hypr.input_kb_layout()?);
    println!("  Follow mouse:       {}", hypr.input_follow_mouse()?);
    println!("  Sensitivity:        {}", hypr.input_sensitivity()?);
    println!(
        "  Natural scroll:     {}\n",
        hypr.input_touchpad_natural_scroll()?
    );

    // ========== Access Layout Settings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                     LAYOUT SETTINGS                          │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!("  Dwindle pseudotile:     {}", hypr.dwindle_pseudotile()?);
    println!(
        "  Dwindle preserve split: {}",
        hypr.dwindle_preserve_split()?
    );
    println!("  Master new status:      {}\n", hypr.master_new_status()?);

    // ========== Access Misc Settings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                        MISC SETTINGS                         │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    println!(
        "  Disable logo:           {}",
        hypr.misc_disable_hyprland_logo()?
    );
    println!(
        "  Force wallpaper:        {}\n",
        hypr.misc_force_default_wallpaper()?
    );

    // ========== Access Variables ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                          VARIABLES                           │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let vars = hypr.variables();
    for (name, value) in vars {
        println!("  ${:<20} = \"{}\"", name, value);
    }
    println!();

    // ========== Access Keybindings ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                        KEYBINDINGS                           │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let binds = hypr.all_binds();
    println!("  Regular binds ({}):", binds.len());
    for (i, bind) in binds.iter().enumerate() {
        println!("    [{}] {}", i + 1, bind);
    }
    println!();

    let bindm = hypr.all_bindm();
    println!("  Mouse binds ({}):", bindm.len());
    for (i, bind) in bindm.iter().enumerate() {
        println!("    [{}] {}", i + 1, bind);
    }
    println!();

    // ========== Access Window Rules ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                        WINDOW RULES                          │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let windowrules = hypr.all_windowrules();
    println!("  Window rules ({}):", windowrules.len());
    for (i, rule) in windowrules.iter().enumerate() {
        println!("    [{}] {}", i + 1, rule);
    }
    println!();

    // ========== Access Monitor Configuration ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                     MONITOR CONFIGURATION                    │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let monitors = hypr.all_monitors();
    println!("  Monitors ({}):", monitors.len());
    for (i, mon) in monitors.iter().enumerate() {
        println!("    [{}] {}", i + 1, mon);
    }
    println!();

    // ========== Access Environment Variables ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                   ENVIRONMENT VARIABLES                      │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let envs = hypr.all_env();
    println!("  Environment variables ({}):", envs.len());
    for (i, env) in envs.iter().enumerate() {
        println!("    [{}] {}", i + 1, env);
    }
    println!();

    // ========== Access Autostart ==========
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                         AUTOSTART                            │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    let exec_once = hypr.all_exec_once();
    println!("  Exec-once ({}):", exec_once.len());
    for (i, exec) in exec_once.iter().enumerate() {
        println!("    [{}] {}", i + 1, exec);
    }
    println!();

    // ========== Summary ==========
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                           SUMMARY                            │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    println!("  📝 Variables:          {}", vars.len());
    println!("  🎹 Keybindings:        {}", binds.len() + bindm.len());
    println!("  🪟 Window rules:       {}", windowrules.len());
    println!("  🖥️  Monitors:           {}", monitors.len());
    println!("  🌍 Environment vars:   {}", envs.len());
    println!("  🚀 Autostart commands: {}", exec_once.len());
    println!("  🎬 Animations:         {}", animations.len());
    println!("  📐 Bezier curves:      {}\n", beziers.len());

    println!("✨ The Hyprland API provides typed, convenient access to all config values!");

    Ok(())
}

#[cfg(not(feature = "hyprland"))]
fn main() {
    eprintln!("Error: This example requires the 'hyprland' feature.");
    eprintln!("Run with: cargo run --example hyprland_api --features hyprland");
    std::process::exit(1);
}
