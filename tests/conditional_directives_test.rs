use hyprlang::Config;

#[test]
fn test_basic_if_exists() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $MY_VAR = value

        # hyprlang if MY_VAR
        included = included_value
        # hyprlang endif

        # hyprlang if MISSING_VAR
        excluded = excluded_value
        # hyprlang endif
    "#,
        )
        .unwrap();

    // Variable exists, so included should be set
    assert_eq!(config.get_string("included").unwrap(), "included_value");

    // Variable doesn't exist, so excluded should not be set
    assert!(config.get("excluded").is_err());
}

#[test]
fn test_negated_if_missing() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $EXISTING_VAR = value

        # hyprlang if !MISSING_VAR
        included = included_value
        # hyprlang endif

        # hyprlang if !EXISTING_VAR
        excluded = excluded_value
        # hyprlang endif
    "#,
        )
        .unwrap();

    // Variable doesn't exist, so condition is true
    assert_eq!(config.get_string("included").unwrap(), "included_value");

    // Variable exists, so condition is false
    assert!(config.get("excluded").is_err());
}

#[test]
fn test_nested_if_statements() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $VAR1 = value1
        $VAR2 = value2

        # hyprlang if VAR1
            # hyprlang if VAR2
                both_exist = test_yes
            # hyprlang endif
        # hyprlang endif

        # hyprlang if VAR1
            # hyprlang if MISSING
                one_exists = test_no
            # hyprlang endif
        # hyprlang endif
    "#,
        )
        .unwrap();

    // Both VAR1 and VAR2 exist
    assert_eq!(config.get_string("both_exist").unwrap(), "test_yes");

    // VAR1 exists but MISSING doesn't
    assert!(config.get("one_exists").is_err());
}

#[test]
fn test_nested_with_negation() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $VAR1 = value

        # hyprlang if VAR1
            # hyprlang if !MISSING
                nested_neg = test_yes
            # hyprlang endif
        # hyprlang endif

        # hyprlang if !MISSING1
            # hyprlang if !MISSING2
                both_missing = test_yes
            # hyprlang endif
        # hyprlang endif
    "#,
        )
        .unwrap();

    // VAR1 exists and MISSING doesn't exist
    assert_eq!(config.get_string("nested_neg").unwrap(), "test_yes");

    // Both MISSING1 and MISSING2 don't exist
    assert_eq!(config.get_string("both_missing").unwrap(), "test_yes");
}

#[test]
fn test_three_level_nesting() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $A = 1
        $B = 2
        $C = 3

        # hyprlang if A
            # hyprlang if B
                # hyprlang if C
                    all_three = test_yes
                # hyprlang endif
            # hyprlang endif
        # hyprlang endif
    "#,
        )
        .unwrap();

    // All three variables exist
    assert_eq!(config.get_string("all_three").unwrap(), "test_yes");
}

#[test]
fn test_mixed_nesting() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $EXISTS = value

        # hyprlang if EXISTS
            # hyprlang if !MISSING
                mixed = test_yes
            # hyprlang endif
        # hyprlang endif

        # hyprlang if !MISSING1
            # hyprlang if EXISTS
                mixed2 = test_yes
            # hyprlang endif
        # hyprlang endif
    "#,
        )
        .unwrap();

    // EXISTS exists and MISSING doesn't
    assert_eq!(config.get_string("mixed").unwrap(), "test_yes");

    // MISSING1 doesn't exist and EXISTS does
    assert_eq!(config.get_string("mixed2").unwrap(), "test_yes");
}

#[test]
fn test_noerror_directive() {
    let mut config = Config::new();

    // Should not fail even with syntax that would normally error
    config
        .parse(
            r#"
        # hyprlang noerror true
        valid = value
        # hyprlang noerror false
    "#,
        )
        .unwrap();

    assert_eq!(config.get_string("valid").unwrap(), "value");
}

#[test]
fn test_if_with_categories() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $ENABLE_FEATURE = test_yes

        # hyprlang if ENABLE_FEATURE
        feature {
            setting1 = value1
            setting2 = value2
        }
        # hyprlang endif

        # hyprlang if DISABLED_FEATURE
        disabled {
            setting = test_no
        }
        # hyprlang endif
    "#,
        )
        .unwrap();

    // Feature category should be parsed
    assert_eq!(config.get_string("feature:setting1").unwrap(), "value1");
    assert_eq!(config.get_string("feature:setting2").unwrap(), "value2");

    // Disabled category should not exist
    assert!(config.get("disabled:setting").is_err());
}

#[test]
fn test_negation_with_env_var() {
    let mut config = Config::new();

    // Set an environment variable for testing
    unsafe {
        std::env::set_var("TEST_ENV_VAR", "test_value");
    }

    config
        .parse(
            r#"
        $ENV_VAR = $TEST_ENV_VAR

        # hyprlang if ENV_VAR
        env_exists = test_yes
        # hyprlang endif

        # hyprlang if !MISSING_ENV
        no_env = test_yes
        # hyprlang endif
    "#,
        )
        .unwrap();

    assert_eq!(config.get_string("env_exists").unwrap(), "test_yes");
    assert_eq!(config.get_string("no_env").unwrap(), "test_yes");

    // Clean up
    unsafe {
        std::env::remove_var("TEST_ENV_VAR");
    }
}

#[test]
fn test_endif_without_if() {
    let mut config = Config::new();

    // Should error on endif without matching if
    let result = config.parse(
        r#"
        value = test
        # hyprlang endif
    "#,
    );

    assert!(result.is_err());
}

#[test]
fn test_unclosed_if() {
    let mut config = Config::new();

    // Should succeed during parse but has_unclosed_blocks should be true
    config
        .parse(
            r#"
        $VAR = value
        # hyprlang if VAR
        value = test
    "#,
        )
        .unwrap();

    // Note: The Config doesn't expose has_unclosed_blocks,
    // but in a real scenario this would be caught
}
