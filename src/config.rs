use crate::error::{ConfigError, ParseResult};
use crate::expressions::ExpressionEvaluator;
use crate::features::{DirectiveProcessor, MultilineProcessor, SourceResolver};
use crate::handlers::{FunctionHandler, Handler, HandlerManager};
use crate::parser::{HyprlangParser, Statement, Value};
use crate::special_categories::{SpecialCategoryDescriptor, SpecialCategoryManager};
use crate::types::{Color, ConfigValue, ConfigValueEntry, CustomValueType, Vec2};
use crate::variables::VariableManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Main configuration manager
pub struct Config {
    /// Configuration values: category_path:key -> value
    values: HashMap<String, ConfigValueEntry>,

    /// Handler call values (stored as arrays): handler_name -> [values]
    handler_calls: HashMap<String, Vec<String>>,

    /// Variable manager
    variables: VariableManager,

    /// Expression evaluator
    expressions: ExpressionEvaluator,

    /// Handler manager
    handlers: HandlerManager,

    /// Special category manager
    special_categories: SpecialCategoryManager,

    /// Custom type handlers
    custom_types: HashMap<String, Rc<dyn CustomValueType>>,

    /// Directive processor
    directives: DirectiveProcessor,

    /// Source resolver
    source_resolver: Option<SourceResolver>,

    /// Configuration options
    options: ConfigOptions,

    /// Current category path (for nested categories)
    current_path: Vec<String>,

    /// Collected errors (when throw_all_errors is enabled)
    errors: Vec<ConfigError>,
}

/// Configuration options
#[derive(Debug, Clone)]
pub struct ConfigOptions {
    /// Throw all errors at once instead of stopping at the first error
    pub throw_all_errors: bool,

    /// Allow dynamic parsing (parse after initial parse is complete)
    pub allow_dynamic_parsing: bool,

    /// Base directory for resolving source directives
    pub base_dir: Option<PathBuf>,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            throw_all_errors: false,
            allow_dynamic_parsing: true,
            base_dir: None,
        }
    }
}

impl Config {
    /// Create a new configuration with default options
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            handler_calls: HashMap::new(),
            variables: VariableManager::new(),
            expressions: ExpressionEvaluator::new(),
            handlers: HandlerManager::new(),
            special_categories: SpecialCategoryManager::new(),
            custom_types: HashMap::new(),
            directives: DirectiveProcessor::new(),
            source_resolver: None,
            options: ConfigOptions::default(),
            current_path: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a new configuration with custom options
    pub fn with_options(options: ConfigOptions) -> Self {
        let source_resolver = options.base_dir.as_ref()
            .map(|dir| SourceResolver::new(dir));

        Self {
            values: HashMap::new(),
            handler_calls: HashMap::new(),
            variables: VariableManager::new(),
            expressions: ExpressionEvaluator::new(),
            handlers: HandlerManager::new(),
            special_categories: SpecialCategoryManager::new(),
            custom_types: HashMap::new(),
            directives: DirectiveProcessor::new(),
            source_resolver,
            options,
            current_path: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Initialize the configuration (called before parsing)
    pub fn commence(&mut self) -> ParseResult<()> {
        // Reset state
        self.errors.clear();
        self.directives.reset();
        Ok(())
    }

    /// Parse a configuration file
    pub fn parse_file(&mut self, path: impl AsRef<Path>) -> ParseResult<()> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::io(path.display().to_string(), e.to_string()))?;

        // Set base dir from file path if not already set
        if self.options.base_dir.is_none() {
            if let Some(parent) = path.parent() {
                self.options.base_dir = Some(parent.to_path_buf());
                self.source_resolver = Some(SourceResolver::new(parent));
            }
        }

        self.parse(&content)
    }

    /// Parse a configuration string
    pub fn parse(&mut self, input: &str) -> ParseResult<()> {
        self.commence()?;

        let parsed = HyprlangParser::parse_config(input)?;

        for statement in parsed.statements {
            if let Err(e) = self.process_statement(&statement) {
                if self.options.throw_all_errors {
                    self.errors.push(e);
                } else {
                    return Err(e);
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(ConfigError::multiple(self.errors.clone()));
        }

        Ok(())
    }

    /// Parse a single line dynamically (after initial parse)
    pub fn parse_dynamic(&mut self, line: &str) -> ParseResult<()> {
        if !self.options.allow_dynamic_parsing {
            return Err(ConfigError::custom("Dynamic parsing is not enabled"));
        }

        let parsed = HyprlangParser::parse_config(line)?;

        for statement in parsed.statements {
            self.process_statement(&statement)?;
        }

        Ok(())
    }

    fn process_statement(&mut self, statement: &Statement) -> ParseResult<()> {
        // Check if we should execute this statement based on directives
        if !self.directives.should_execute() {
            // Still need to process directives even when not executing
            if let Statement::CommentDirective { directive_type, args } = statement {
                return self.directives.process_directive(
                    directive_type,
                    args.as_deref(),
                    &self.variables,
                );
            }
            return Ok(());
        }

        match statement {
            Statement::VariableDef { name, value } => {
                let expanded = self.variables.expand(value)?;
                self.variables.set(name.clone(), expanded.clone());

                // Update expression evaluator if it's a number
                if let Ok(num) = ConfigValue::parse_int(&expanded) {
                    self.expressions.set_variable(name.clone(), num);
                }

                Ok(())
            }

            Statement::Assignment { key, value } => {
                // Check if this is a potential handler call (single identifier and registered handler)
                let is_potential_handler = key.len() == 1;
                let keyword = &key[0];

                if is_potential_handler && self.handlers.has_handler(&self.current_path, keyword) {
                    // Treat as handler call
                    let expanded_value = match value {
                        Value::String(s) => self.variables.expand(s)?,
                        _ => self.value_to_string(value),
                    };

                    // Create full key including category path for handler calls
                    let full_key = if self.current_path.is_empty() {
                        keyword.clone()
                    } else {
                        format!("{}:{}", self.current_path.join(":"), keyword)
                    };

                    self.handler_calls
                        .entry(full_key)
                        .or_insert_with(Vec::new)
                        .push(expanded_value.clone());

                    self.handlers.execute(
                        &self.current_path,
                        keyword,
                        &expanded_value,
                        None,
                    )?;
                } else {
                    // Regular assignment
                    let full_key = self.make_full_key(key);
                    let config_value = self.parse_config_value(value)?;
                    let raw = self.value_to_string(value);

                    self.values.insert(
                        full_key,
                        ConfigValueEntry::new(config_value, raw),
                    );
                }

                Ok(())
            }

            Statement::CategoryBlock { name, statements } => {
                self.current_path.push(name.clone());

                for stmt in statements {
                    if let Err(e) = self.process_statement(stmt) {
                        if self.options.throw_all_errors {
                            self.errors.push(e);
                        } else {
                            self.current_path.pop();
                            return Err(e);
                        }
                    }
                }

                self.current_path.pop();
                Ok(())
            }

            Statement::SpecialCategoryBlock { name, key, statements } => {
                // Ensure the category is registered
                if !self.special_categories.is_registered(name) {
                    return Err(ConfigError::category_not_found(name, None));
                }

                // Create the instance with the provided key (or auto-generate if none)
                let instance_key = self.special_categories.create_instance(name, key.clone())?;

                self.current_path.push(format!("{}[{}]", name, instance_key));

                // Process statements within the category
                for stmt in statements {
                    if let Err(e) = self.process_statement(stmt) {
                        if self.options.throw_all_errors {
                            self.errors.push(e);
                        } else {
                            self.current_path.pop();
                            return Err(e);
                        }
                    }
                }

                // Store values in the special category instance
                let full_path = self.current_path.last().unwrap();
                for (key, value) in &self.values {
                    if key.starts_with(full_path) {
                        let sub_key = key.strip_prefix(full_path)
                            .unwrap()
                            .trim_start_matches(':');

                        if let Ok(instance) = self.special_categories.get_instance_mut(name, &instance_key) {
                            instance.set(sub_key.to_string(), value.clone());
                        }
                    }
                }

                self.current_path.pop();
                Ok(())
            }

            Statement::HandlerCall { keyword, flags, value } => {
                let expanded_value = self.variables.expand(value)?;

                // Store the handler call value only if it's registered or at root level
                let should_store = self.handlers.has_handler(&self.current_path, keyword)
                    || self.current_path.is_empty();

                if should_store {
                    let full_key = if self.current_path.is_empty() {
                        keyword.clone()
                    } else {
                        format!("{}:{}", self.current_path.join(":"), keyword)
                    };

                    self.handler_calls
                        .entry(full_key)
                        .or_insert_with(Vec::new)
                        .push(expanded_value.clone());
                }

                // Execute the handler if one is registered
                self.handlers.execute(
                    &self.current_path,
                    keyword,
                    &expanded_value,
                    flags.clone(),
                )
            }

            Statement::Source { path } => {
                let expanded_path = self.variables.expand(path)?;

                // Resolve and begin load
                let resolved = if let Some(resolver) = &mut self.source_resolver {
                    let resolved = resolver.resolve_path(&expanded_path)?;
                    resolver.begin_load(&resolved)?;
                    resolved
                } else {
                    return Err(ConfigError::custom("Source resolver not initialized"));
                };

                // Parse the file
                let result = self.parse_file(&resolved);

                // End load
                if let Some(resolver) = &mut self.source_resolver {
                    resolver.end_load();
                }

                result
            }

            Statement::CommentDirective { directive_type, args } => {
                self.directives.process_directive(
                    directive_type,
                    args.as_deref(),
                    &self.variables,
                )
            }
        }
    }

    fn parse_config_value(&mut self, value: &Value) -> ParseResult<ConfigValue> {
        match value {
            Value::Expression(expr) => {
                let result = self.expressions.evaluate(expr)?;
                Ok(ConfigValue::Int(result))
            }

            Value::Variable(name) => {
                let expanded = self.variables.expand(&format!("${}", name))?;
                // Try to parse as a known type
                self.parse_string_value(&expanded)
            }

            Value::Color(color) => Ok(ConfigValue::Color(*color)),

            Value::Vec2(vec) => Ok(ConfigValue::Vec2(*vec)),

            Value::Number(num) => {
                // Try int first, then float
                if let Ok(i) = ConfigValue::parse_int(num) {
                    Ok(ConfigValue::Int(i))
                } else if let Ok(f) = ConfigValue::parse_float(num) {
                    Ok(ConfigValue::Float(f))
                } else {
                    Err(ConfigError::invalid_number(num, "not a valid number"))
                }
            }

            Value::Boolean(b) => Ok(ConfigValue::Int(if *b { 1 } else { 0 })),

            Value::String(s) => {
                let expanded = self.variables.expand(s)?;
                self.parse_string_value(&expanded)
            }

            Value::Multiline(lines) => {
                let joined = MultilineProcessor::join_lines(lines);
                let expanded = self.variables.expand(&joined)?;
                Ok(ConfigValue::String(expanded))
            }
        }
    }

    fn parse_string_value(&self, s: &str) -> ParseResult<ConfigValue> {
        let s = s.trim();

        // Try to parse as various types
        if let Ok(b) = ConfigValue::parse_bool(s) {
            return Ok(ConfigValue::Int(if b { 1 } else { 0 }));
        }

        // Try color formats: rgba(...), rgb(...), 0xHEXHEX
        if s.starts_with("rgba(") && s.ends_with(')') {
            if let Ok(color) = self.parse_rgba_string(s) {
                return Ok(ConfigValue::Color(color));
            }
        } else if s.starts_with("rgb(") && s.ends_with(')') {
            if let Ok(color) = self.parse_rgb_string(s) {
                return Ok(ConfigValue::Color(color));
            }
        } else if s.starts_with("0x") && s.len() >= 8 && s.len() <= 10 {
            // Hex color: 0xRRGGBB or 0xRRGGBBAA
            if let Ok(color) = Color::from_hex(s) {
                return Ok(ConfigValue::Color(color));
            }
        }

        // Try Vec2: (x, y) or x, y
        if let Ok(vec2) = self.parse_vec2_string(s) {
            return Ok(ConfigValue::Vec2(vec2));
        }

        if let Ok(i) = ConfigValue::parse_int(s) {
            return Ok(ConfigValue::Int(i));
        }

        if let Ok(f) = ConfigValue::parse_float(s) {
            return Ok(ConfigValue::Float(f));
        }

        // Default to string (remove any trailing whitespace)
        Ok(ConfigValue::String(s.to_string()))
    }

    fn parse_rgba_string(&self, s: &str) -> ParseResult<Color> {
        // rgba(hex) or rgba(r, g, b, a)
        let inner = &s[5..s.len()-1]; // Remove "rgba(" and ")"

        if !inner.contains(',') {
            // Hex format: rgba(RRGGBBAA)
            Color::from_hex(inner)
        } else {
            // Component format: rgba(r, g, b, a)
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() != 4 {
                return Err(ConfigError::invalid_color(s, "rgba needs 4 components"));
            }

            let r = parts[0].parse::<u8>()
                .map_err(|_| ConfigError::invalid_color(s, "invalid r"))?;
            let g = parts[1].parse::<u8>()
                .map_err(|_| ConfigError::invalid_color(s, "invalid g"))?;
            let b = parts[2].parse::<u8>()
                .map_err(|_| ConfigError::invalid_color(s, "invalid b"))?;

            // Alpha can be float (0.0-1.0) or int (0-255)
            let a = if parts[3].contains('.') {
                let a_float = parts[3].parse::<f64>()
                    .map_err(|_| ConfigError::invalid_color(s, "invalid a"))?;
                (a_float * 255.0).round() as u8
            } else {
                parts[3].parse::<u8>()
                    .map_err(|_| ConfigError::invalid_color(s, "invalid a"))?
            };

            Ok(Color::from_rgba(r, g, b, a))
        }
    }

    fn parse_rgb_string(&self, s: &str) -> ParseResult<Color> {
        // rgb(r, g, b)
        let inner = &s[4..s.len()-1]; // Remove "rgb(" and ")"
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

        if parts.len() != 3 {
            return Err(ConfigError::invalid_color(s, "rgb needs 3 components"));
        }

        let r = parts[0].parse::<u8>()
            .map_err(|_| ConfigError::invalid_color(s, "invalid r"))?;
        let g = parts[1].parse::<u8>()
            .map_err(|_| ConfigError::invalid_color(s, "invalid g"))?;
        let b = parts[2].parse::<u8>()
            .map_err(|_| ConfigError::invalid_color(s, "invalid b"))?;

        Ok(Color::from_rgb(r, g, b))
    }

    fn parse_vec2_string(&self, s: &str) -> ParseResult<Vec2> {
        // Try (x, y) format
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len()-1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    return Ok(Vec2::new(x, y));
                }
            }
        } else if s.contains(',') {
            // Try x, y format without parentheses
            let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();

            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    return Ok(Vec2::new(x, y));
                }
            }
        }

        Err(ConfigError::custom("not a valid Vec2"))
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Expression(e) => format!("{{{{{}}}}}", e),
            Value::Variable(v) => format!("${}", v),
            Value::Color(c) => c.to_string(),
            Value::Vec2(v) => v.to_string(),
            Value::Multiline(lines) => lines.join("\n"),
        }
    }

    fn make_full_key(&self, key: &[String]) -> String {
        if self.current_path.is_empty() {
            key.join(":")
        } else {
            format!("{}:{}", self.current_path.join(":"), key.join(":"))
        }
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> ParseResult<&ConfigValue> {
        self.values.get(key)
            .map(|entry| &entry.value)
            .ok_or_else(|| ConfigError::key_not_found(key))
    }

    /// Get a configuration value as a specific type
    pub fn get_int(&self, key: &str) -> ParseResult<i64> {
        self.get(key)?.as_int()
    }

    pub fn get_float(&self, key: &str) -> ParseResult<f64> {
        self.get(key)?.as_float()
    }

    pub fn get_string(&self, key: &str) -> ParseResult<&str> {
        self.get(key)?.as_string()
    }

    pub fn get_vec2(&self, key: &str) -> ParseResult<Vec2> {
        self.get(key)?.as_vec2()
    }

    pub fn get_color(&self, key: &str) -> ParseResult<Color> {
        self.get(key)?.as_color()
    }

    /// Set a configuration value directly
    pub fn set(&mut self, key: impl Into<String>, value: ConfigValue) {
        let key = key.into();
        let raw = value.to_string();
        self.values.insert(key, ConfigValueEntry::new(value, raw));
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Register a handler
    pub fn register_handler<H>(&mut self, keyword: impl Into<String>, handler: H)
    where
        H: Handler + 'static,
    {
        self.handlers.register_global(keyword, handler);
    }

    /// Register a function handler
    pub fn register_handler_fn<F>(&mut self, keyword: impl Into<String>, handler: F)
    where
        F: Fn(&crate::handlers::HandlerContext) -> ParseResult<()> + 'static,
    {
        let keyword = keyword.into();
        self.handlers.register_global(
            keyword.clone(),
            FunctionHandler::new(keyword, handler),
        );
    }

    /// Register a category-specific handler
    pub fn register_category_handler<H>(
        &mut self,
        category: impl Into<String>,
        keyword: impl Into<String>,
        handler: H,
    )
    where
        H: Handler + 'static,
    {
        self.handlers.register_category(category, keyword, handler);
    }

    /// Register a category-specific function handler
    pub fn register_category_handler_fn<F>(
        &mut self,
        category: impl Into<String>,
        keyword: impl Into<String>,
        handler: F,
    )
    where
        F: Fn(&crate::handlers::HandlerContext) -> ParseResult<()> + 'static,
    {
        let keyword_str = keyword.into();
        let category_str = category.into();
        self.handlers.register_category(
            category_str,
            keyword_str.clone(),
            FunctionHandler::new(keyword_str, handler),
        );
    }

    /// Register a special category
    pub fn register_special_category(&mut self, descriptor: SpecialCategoryDescriptor) {
        self.special_categories.register(descriptor);
    }

    /// Get a special category instance
    pub fn get_special_category(&self, category: &str, key: &str) -> ParseResult<HashMap<String, &ConfigValue>> {
        let instance = self.special_categories.get_instance(category, key)?;
        let mut result = HashMap::new();

        for (k, v) in &instance.values {
            result.insert(k.clone(), &v.value);
        }

        Ok(result)
    }

    /// List all keys for a special category
    pub fn list_special_category_keys(&self, category: &str) -> Vec<String> {
        self.special_categories.list_keys(category)
    }

    /// Register a custom value type
    pub fn register_custom_type<T>(&mut self, type_name: impl Into<String>, handler: T)
    where
        T: CustomValueType + 'static,
    {
        self.custom_types.insert(type_name.into(), Rc::new(handler));
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.set(name.clone(), value.clone());

        // Update expression evaluator if it's a number
        if let Ok(num) = ConfigValue::parse_int(&value) {
            self.expressions.set_variable(name, num);
        }
    }

    /// Get all configuration keys
    pub fn keys(&self) -> Vec<&str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }

    /// Get all variables
    pub fn variables(&self) -> &HashMap<String, String> {
        self.variables.all()
    }

    /// Get all handler calls for a specific handler
    pub fn get_handler_calls(&self, handler: &str) -> Option<&Vec<String>> {
        self.handler_calls.get(handler)
    }

    /// Get all handler names that have been called
    pub fn handler_names(&self) -> Vec<&str> {
        self.handler_calls.keys().map(|s| s.as_str()).collect()
    }

    /// Get all handler calls as a map
    pub fn all_handler_calls(&self) -> &HashMap<String, Vec<String>> {
        &self.handler_calls
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
