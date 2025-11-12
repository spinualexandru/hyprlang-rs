use crate::error::{ConfigError, ParseResult};
use crate::variables::VariableManager;
use std::path::{Path, PathBuf};

/// Directive processor for handling comment directives
pub struct DirectiveProcessor {
    /// Stack of active if conditions
    if_stack: Vec<bool>,

    /// Whether to suppress errors
    suppress_errors: bool,
}

impl DirectiveProcessor {
    pub fn new() -> Self {
        Self {
            if_stack: Vec::new(),
            suppress_errors: false,
        }
    }

    /// Process a comment directive
    pub fn process_directive(
        &mut self,
        directive_type: &str,
        args: Option<&str>,
        variables: &VariableManager,
    ) -> ParseResult<()> {
        match directive_type {
            "if" => {
                let var_name = args.ok_or_else(|| {
                    ConfigError::custom("'if' directive requires a variable name")
                })?;

                let var_name = var_name.trim();
                let condition = variables.contains(var_name);
                self.if_stack.push(condition);
                Ok(())
            }

            "endif" => {
                if self.if_stack.is_empty() {
                    return Err(ConfigError::custom("'endif' without matching 'if'"));
                }
                self.if_stack.pop();
                Ok(())
            }

            "noerror" => {
                let value = args.ok_or_else(|| {
                    ConfigError::custom("'noerror' directive requires a value (true/false)")
                })?;

                let value = value.trim();
                self.suppress_errors = value == "true";
                Ok(())
            }

            _ => Err(ConfigError::custom(format!(
                "Unknown directive: {}",
                directive_type
            ))),
        }
    }

    /// Check if current code should be executed (based on if conditions)
    pub fn should_execute(&self) -> bool {
        // Execute if all conditions in the stack are true (or stack is empty)
        self.if_stack.iter().all(|&cond| cond)
    }

    /// Check if errors should be suppressed
    pub fn should_suppress_errors(&self) -> bool {
        self.suppress_errors
    }

    /// Reset the processor state
    pub fn reset(&mut self) {
        self.if_stack.clear();
        self.suppress_errors = false;
    }

    /// Check if there are unclosed if blocks
    pub fn has_unclosed_blocks(&self) -> bool {
        !self.if_stack.is_empty()
    }
}

impl Default for DirectiveProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Source file resolver for handling source directives
pub struct SourceResolver {
    /// Base directory for resolving relative paths
    base_dir: PathBuf,

    /// Stack of currently loading files (for cycle detection)
    loading_stack: Vec<PathBuf>,

    /// Maximum recursion depth
    max_depth: usize,
}

impl SourceResolver {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            loading_stack: Vec::new(),
            max_depth: 50,
        }
    }

    /// Set the maximum recursion depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Resolve a source path relative to the base directory
    pub fn resolve_path(&self, path: &str) -> ParseResult<PathBuf> {
        let path_obj = Path::new(path);

        let resolved = if path_obj.is_absolute() {
            path_obj.to_path_buf()
        } else {
            self.base_dir.join(path_obj)
        };

        // Canonicalize to resolve . and .. components
        resolved.canonicalize()
            .map_err(|e| ConfigError::io(path, format!("failed to resolve path: {}", e)))
    }

    /// Begin loading a file (checks for cycles and depth)
    pub fn begin_load(&mut self, path: &Path) -> ParseResult<()> {
        // Check depth
        if self.loading_stack.len() >= self.max_depth {
            return Err(ConfigError::custom(format!(
                "Maximum source directive recursion depth ({}) exceeded",
                self.max_depth
            )));
        }

        // Check for cycles
        if self.loading_stack.contains(&path.to_path_buf()) {
            return Err(ConfigError::custom(format!(
                "Circular source directive detected: {}",
                path.display()
            )));
        }

        self.loading_stack.push(path.to_path_buf());
        Ok(())
    }

    /// End loading a file
    pub fn end_load(&mut self) {
        self.loading_stack.pop();
    }

    /// Get the current loading stack depth
    pub fn depth(&self) -> usize {
        self.loading_stack.len()
    }

    /// Reset the resolver
    pub fn reset(&mut self) {
        self.loading_stack.clear();
    }
}

/// Multiline value processor
pub struct MultilineProcessor;

impl MultilineProcessor {
    /// Join multiline values into a single string
    pub fn join_lines(lines: &[String]) -> String {
        lines.join(" ")
    }

    /// Check if a line ends with a backslash (continuation)
    pub fn is_continuation(line: &str) -> bool {
        line.trim_end().ends_with('\\')
    }

    /// Remove the trailing backslash from a line
    pub fn remove_backslash(line: &str) -> String {
        let trimmed = line.trim_end();
        if trimmed.ends_with('\\') {
            trimmed[..trimmed.len() - 1].to_string()
        } else {
            line.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directive_if() {
        let mut processor = DirectiveProcessor::new();
        let mut variables = VariableManager::new();

        variables.set("TEST".to_string(), "value".to_string());

        // Variable exists
        processor.process_directive("if", Some("TEST"), &variables).unwrap();
        assert!(processor.should_execute());

        processor.process_directive("endif", None, &variables).unwrap();

        // Variable doesn't exist
        processor.process_directive("if", Some("MISSING"), &variables).unwrap();
        assert!(!processor.should_execute());

        processor.process_directive("endif", None, &variables).unwrap();
    }

    #[test]
    fn test_directive_noerror() {
        let mut processor = DirectiveProcessor::new();
        let variables = VariableManager::new();

        assert!(!processor.should_suppress_errors());

        processor.process_directive("noerror", Some("true"), &variables).unwrap();
        assert!(processor.should_suppress_errors());

        processor.process_directive("noerror", Some("false"), &variables).unwrap();
        assert!(!processor.should_suppress_errors());
    }

    #[test]
    fn test_multiline_join() {
        let lines = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ];

        assert_eq!(MultilineProcessor::join_lines(&lines), "line1 line2 line3");
    }

    #[test]
    fn test_multiline_continuation() {
        assert!(MultilineProcessor::is_continuation("line \\"));
        assert!(MultilineProcessor::is_continuation("line\\  "));
        assert!(!MultilineProcessor::is_continuation("line"));
    }

    #[test]
    fn test_remove_backslash() {
        assert_eq!(MultilineProcessor::remove_backslash("line\\"), "line");
        assert_eq!(MultilineProcessor::remove_backslash("line\\  "), "line");
        assert_eq!(MultilineProcessor::remove_backslash("line"), "line");
    }
}
