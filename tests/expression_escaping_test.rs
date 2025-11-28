use hyprlang::Config;

#[test]
fn test_backslash_escape_expr() {
    let mut config = Config::new();
    config.parse(r"testValue = \{{10 + 5}}").unwrap();

    // Should be literal string, not evaluated
    assert_eq!(config.get_string("testValue").unwrap(), "{{10 + 5}}");
}

#[test]
fn test_brace_escape_expr() {
    let mut config = Config::new();
    config.parse(r"testValue = {\{10 + 5}}").unwrap();

    // Should be literal string, not evaluated
    assert_eq!(config.get_string("testValue").unwrap(), "{{10 + 5}}");
}

#[test]
fn test_double_backslash_brace_escape() {
    let mut config = Config::new();
    config.parse(r"testValue = \{\{10 + 5}}").unwrap();

    // \{\{ should produce literal {{
    assert_eq!(config.get_string("testValue").unwrap(), "{{10 + 5}}");
}

#[test]
fn test_escaped_backslash_with_expr() {
    let mut config = Config::new();
    config.parse(r"testValue = \\{{10 - 5}}").unwrap();

    // Should evaluate the expression but keep one backslash
    assert_eq!(config.get_string("testValue").unwrap(), r"\5");
}

#[test]
fn test_mixed_escaped_and_eval() {
    let mut config = Config::new();
    config
        .parse(r#"testValue = "{{8 - 10}} and \{{literal}}""#)
        .unwrap();

    // First expression evaluates, second is literal
    assert_eq!(
        config.get_string("testValue").unwrap(),
        "-2 and {{literal}}"
    );
}

#[test]
fn test_escaped_expr_in_variable() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $ESCAPED = \{{10 + 10}}
        testValue = $ESCAPED
    "#,
        )
        .unwrap();

    // Variable should contain literal {{}}
    assert_eq!(config.get_string("testValue").unwrap(), "{{10 + 10}}");
}

#[test]
fn test_complex_mixed_escapes() {
    let mut config = Config::new();
    config
        .parse(r#"testValue = "{{8 - 10}} \{{ literal }} more""#)
        .unwrap();

    // First expression evaluates, escaped part stays literal
    assert_eq!(
        config.get_string("testValue").unwrap(),
        "-2 {{ literal }} more"
    );
}

#[test]
fn test_escaped_with_variables() {
    let mut config = Config::new();
    config
        .parse(
            r#"
        $MOVING_VAR = 500
        $DYNAMIC = moved: {{$MOVING_VAR / 2}} expr: \{{$MOVING_VAR / 2}}
        testValue = \{{ $DYNAMIC }}
    "#,
        )
        .unwrap();

    // The testValue should have escaped outer braces, but DYNAMIC should be expanded
    assert_eq!(
        config.get_string("testValue").unwrap(),
        "{{ moved: 250 expr: {{500 / 2}} }}"
    );
}

#[test]
fn test_no_escape_normal_expr() {
    let mut config = Config::new();
    config.parse(r"testValue = {{10 + 5}}").unwrap();

    // Normal expression should evaluate
    assert_eq!(config.get_int("testValue").unwrap(), 15);
}

#[test]
fn test_escaped_not_expression() {
    let mut config = Config::new();
    config.parse(r"testValue = \{notAnExpression").unwrap();

    // Single brace escape should not be processed
    assert_eq!(
        config.get_string("testValue").unwrap(),
        r"\{notAnExpression"
    );
}

#[test]
fn test_config_file_escapes() {
    // Test the actual patterns from the test config file
    let mut config = Config::new();

    config
        .parse(
            r#"
        testInt = 123
        testEscapedExpr = "\{{testInt + 7}}"
        testEscapedExpr2 = "{\{testInt + 7}}"
        testEscapedExpr3 = "\{\{3 + 8}}"
        testEscapedEscape = "\\{{10 - 5}}"
        testSimpleMix = "{{8 - 10}} and \{{ literal }}"
    "#,
        )
        .unwrap();

    assert_eq!(
        config.get_string("testEscapedExpr").unwrap(),
        "{{testInt + 7}}"
    );
    assert_eq!(
        config.get_string("testEscapedExpr2").unwrap(),
        "{{testInt + 7}}"
    );
    assert_eq!(config.get_string("testEscapedExpr3").unwrap(), "{{3 + 8}}");
    assert_eq!(config.get_string("testEscapedEscape").unwrap(), r"\5");
    assert_eq!(
        config.get_string("testSimpleMix").unwrap(),
        "-2 and {{ literal }}"
    );
}
