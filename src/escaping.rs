//! Expression escape processing
//!
//! This module handles escaping of expression syntax (`{{}}`) to allow literal
//! braces in configuration values.
//!
//! Supported escape sequences:
//! - `\{{expr}}` → `"{{expr}}"` (backslash escape)
//! - `{\{expr}}` → `"{{expr}}"` (brace escape)
//! - `\\{{expr}}` → `"\<evaluated>"` (escaped backslash, expression evaluated)
//!
//! Implementation: Escaped braces are replaced with placeholders during processing,
//! then restored after expression evaluation.

const ESCAPED_OPEN: &str = "\x00ESC_OPEN\x00";
const ESCAPED_CLOSE: &str = "\x00ESC_CLOSE\x00";

/// Process escape sequences, replacing escaped braces with placeholders
///
/// This prevents escaped expressions from being evaluated. After expression
/// evaluation, call `restore_escaped_braces` to convert back to literal braces.
pub fn process_escapes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        let mut temp = chars.clone();
                        temp.next(); // consume {

                        if temp.peek() == Some(&'{') {
                            // \{{ - escape for {{
                            chars.next(); // consume {
                            chars.next(); // consume {
                            result.push_str(ESCAPED_OPEN);

                            // Find and escape the closing }}
                            let mut depth = 1;
                            while let Some(c) = chars.next() {
                                if c == '{' && chars.peek() == Some(&'{') {
                                    depth += 1;
                                    result.push(c);
                                    result.push(chars.next().unwrap());
                                } else if c == '}' && chars.peek() == Some(&'}') {
                                    depth -= 1;
                                    if depth == 0 {
                                        chars.next(); // consume second }
                                        result.push_str(ESCAPED_CLOSE);
                                        break;
                                    }
                                    result.push(c);
                                    result.push(chars.next().unwrap());
                                } else {
                                    result.push(c);
                                }
                            }
                            continue;
                        } else if temp.peek() == Some(&'\\') {
                            // \{\ - check for \{\{
                            temp.next(); // consume \
                            if temp.peek() == Some(&'{') {
                                // \{\{ - escape for {{
                                chars.next(); // consume {
                                chars.next(); // consume \
                                chars.next(); // consume {
                                result.push_str(ESCAPED_OPEN);

                                // Find and escape the closing }}
                                let mut depth = 1;
                                while let Some(c) = chars.next() {
                                    if c == '{' && chars.peek() == Some(&'{') {
                                        depth += 1;
                                        result.push(c);
                                        result.push(chars.next().unwrap());
                                    } else if c == '}' && chars.peek() == Some(&'}') {
                                        depth -= 1;
                                        if depth == 0 {
                                            chars.next(); // consume second }
                                            result.push_str(ESCAPED_CLOSE);
                                            break;
                                        }
                                        result.push(c);
                                        result.push(chars.next().unwrap());
                                    } else {
                                        result.push(c);
                                    }
                                }
                                continue;
                            }
                        }
                    } else if next == '\\' {
                        // Check for \\{{
                        let mut temp = chars.clone();
                        temp.next(); // consume \
                        if temp.peek() == Some(&'{') {
                            temp.next(); // consume {
                            if temp.peek() == Some(&'{') {
                                // \\{{ - keep one \, expression will be evaluated
                                chars.next(); // consume second \
                                result.push('\\');
                                continue;
                            }
                        }
                    }
                }
                result.push(ch);
            }

            '{' => {
                if let Some(&next) = chars.peek() {
                    if next == '\\' {
                        let mut temp = chars.clone();
                        temp.next(); // consume \
                        if temp.peek() == Some(&'{') {
                            // {\{ - escape for {{
                            chars.next(); // consume \
                            chars.next(); // consume {
                            result.push_str(ESCAPED_OPEN);

                            // Find and escape the closing }}
                            let mut depth = 1;
                            while let Some(c) = chars.next() {
                                if c == '{' && chars.peek() == Some(&'{') {
                                    depth += 1;
                                    result.push(c);
                                    result.push(chars.next().unwrap());
                                } else if c == '}' && chars.peek() == Some(&'}') {
                                    depth -= 1;
                                    if depth == 0 {
                                        chars.next(); // consume second }
                                        result.push_str(ESCAPED_CLOSE);
                                        break;
                                    }
                                    result.push(c);
                                    result.push(chars.next().unwrap());
                                } else {
                                    result.push(c);
                                }
                            }
                            continue;
                        }
                    }
                }
                result.push(ch);
            }

            _ => result.push(ch),
        }
    }

    result
}

/// Restore escaped braces from placeholders to literal {{ and }}
///
/// Call this after expression evaluation to convert placeholders back to
/// the literal brace sequences.
pub fn restore_escaped_braces(input: &str) -> String {
    input
        .replace(ESCAPED_OPEN, "{{")
        .replace(ESCAPED_CLOSE, "}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backslash_escape() {
        let escaped = process_escapes(r"\{{expr}}");
        let restored = restore_escaped_braces(&escaped);
        assert_eq!(restored, "{{expr}}");
    }

    #[test]
    fn test_brace_escape() {
        let escaped = process_escapes(r"{\{expr}}");
        let restored = restore_escaped_braces(&escaped);
        assert_eq!(restored, "{{expr}}");
    }

    #[test]
    fn test_double_brace_escape() {
        let escaped = process_escapes(r"\{\{3 + 8}}");
        let restored = restore_escaped_braces(&escaped);
        assert_eq!(restored, "{{3 + 8}}");
    }

    #[test]
    fn test_escaped_backslash() {
        // \\{{ should keep one backslash, expression not escaped
        let escaped = process_escapes(r"\\{{expr}}");
        assert_eq!(escaped, r"\{{expr}}");
        // No placeholders, so restore should not change it
        let restored = restore_escaped_braces(&escaped);
        assert_eq!(restored, r"\{{expr}}");
    }

    #[test]
    fn test_mixed_escapes() {
        let input = r"{{8 - 10}} \{{ \{{50 + 50}} / \{{10 * 5}} }}";
        let escaped = process_escapes(input);
        // First {{8 - 10}} stays normal, rest get placeholders
        assert!(escaped.contains("{{8 - 10}}"));
        assert!(escaped.contains(ESCAPED_OPEN));

        let restored = restore_escaped_braces(&escaped);
        // After restore, escaped parts become literal braces
        assert!(restored.contains("{{"));
    }

    #[test]
    fn test_no_escape() {
        let input = "{{10 + 5}}";
        let escaped = process_escapes(input);
        assert_eq!(escaped, input);
        let restored = restore_escaped_braces(&escaped);
        assert_eq!(restored, input);
    }

    #[test]
    fn test_partial_escape() {
        let input = r"\{single";
        let escaped = process_escapes(input);
        assert_eq!(escaped, input);
    }

    #[test]
    fn test_placeholders_not_in_normal_text() {
        let input = "normal {{expr}} text";
        let escaped = process_escapes(input);
        assert!(!escaped.contains(ESCAPED_OPEN));
        assert!(!escaped.contains(ESCAPED_CLOSE));
    }
}
