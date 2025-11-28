use crate::error::{ConfigError, ParseResult};
use std::collections::{HashMap, HashSet};

/// Variable storage and resolution system
pub struct VariableManager {
    /// User-defined variables
    variables: HashMap<String, String>,

    /// Dependencies between variables (for cycle detection)
    dependencies: HashMap<String, HashSet<String>>,
}

impl VariableManager {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Get a variable value (returns None if not found)
    pub fn get(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Get all variables
    pub fn all(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Expand all variables in a string (including environment variables)
    pub fn expand(&self, input: &str) -> ParseResult<String> {
        self.expand_with_chain(input, &mut Vec::new())
    }

    /// Expand variables with cycle detection
    fn expand_with_chain(&self, input: &str, chain: &mut Vec<String>) -> ParseResult<String> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Read the variable name
                let var_name = self.read_variable_name(&mut chars);

                // Check for circular dependency
                if chain.contains(&var_name) {
                    chain.push(var_name.clone());
                    return Err(ConfigError::circular_dependency(chain.clone()));
                }

                // Try to resolve the variable
                let value = if let Some(val) = self.variables.get(&var_name) {
                    // User-defined variable
                    chain.push(var_name.clone());
                    let expanded = self.expand_with_chain(val, chain)?;
                    chain.pop();
                    expanded
                } else if let Ok(env_val) = std::env::var(&var_name) {
                    // Environment variable
                    env_val
                } else {
                    // Variable not found - return as-is with $
                    result.push('$');
                    result.push_str(&var_name);
                    continue;
                };

                result.push_str(&value);
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Read a variable name from the character stream
    fn read_variable_name(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut name = String::new();

        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                name.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        name
    }

    /// Get all variable names
    pub fn keys(&self) -> Vec<&str> {
        self.variables.keys().map(|s| s.as_str()).collect()
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
        self.dependencies.clear();
    }

    /// Track a dependency between variables
    pub fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies.entry(from).or_default().insert(to);
    }

    /// Get all variables that depend on a given variable
    pub fn get_dependents(&self, var_name: &str) -> Vec<&str> {
        self.dependencies
            .iter()
            .filter(|(_, deps)| deps.contains(var_name))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Remove a variable
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.dependencies.remove(name);
        self.variables.remove(name)
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_expansion() {
        let mut vm = VariableManager::new();
        vm.set("NAME".to_string(), "World".to_string());

        assert_eq!(vm.expand("Hello $NAME!").unwrap(), "Hello World!");
    }

    #[test]
    fn test_recursive_expansion() {
        let mut vm = VariableManager::new();
        vm.set("A".to_string(), "value".to_string());
        vm.set("B".to_string(), "$A".to_string());
        vm.set("C".to_string(), "$B".to_string());

        assert_eq!(vm.expand("$C").unwrap(), "value");
    }

    #[test]
    fn test_circular_dependency() {
        let mut vm = VariableManager::new();
        vm.set("A".to_string(), "$B".to_string());
        vm.set("B".to_string(), "$A".to_string());

        assert!(vm.expand("$A").is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let vm = VariableManager::new();
        // Undefined variables are left as-is
        assert_eq!(vm.expand("$UNDEFINED").unwrap(), "$UNDEFINED");
    }

    #[test]
    fn test_multiple_variables() {
        let mut vm = VariableManager::new();
        vm.set("X".to_string(), "10".to_string());
        vm.set("Y".to_string(), "20".to_string());

        assert_eq!(vm.expand("$X + $Y").unwrap(), "10 + 20");
    }

    #[test]
    fn test_variable_in_middle() {
        let mut vm = VariableManager::new();
        vm.set("VAR".to_string(), "middle".to_string());

        assert_eq!(vm.expand("start $VAR end").unwrap(), "start middle end");
    }
}
