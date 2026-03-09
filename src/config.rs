use crate::error::{ConfigError, ParseResult};
use crate::escaping::{process_escapes, restore_escaped_braces};
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

/// A line that references a variable, tracked for dynamic variable propagation
#[derive(Debug, Clone)]
struct VariableLine {
    /// The raw statement that references the variable
    statement: Statement,
    /// The category path at the time the statement was processed
    category_path: Vec<String>,
}

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

    /// Lines that reference each variable (for dynamic variable propagation)
    variable_lines: HashMap<String, Vec<VariableLine>>,

    /// Whether this parse context should record variable dependency lines.
    ///
    /// Upstream only records dependencies during non-dynamic parsing.
    record_variable_lines: bool,

    /// Document structure (for full-fidelity serialization)
    #[cfg(feature = "mutation")]
    document: Option<crate::document::ConfigDocument>,

    /// Source file path (for save operations)
    #[cfg(feature = "mutation")]
    source_file: Option<PathBuf>,

    /// Multi-file document for tracking source files
    #[cfg(feature = "mutation")]
    multi_document: Option<crate::document::MultiFileDocument>,

    /// Current source file being parsed (for key tracking)
    #[cfg(feature = "mutation")]
    current_source_file: Option<PathBuf>,
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

    /// Parse without setting values (dry-run validation)
    pub verify_only: bool,

    /// Don't error on missing config file
    pub allow_missing_config: bool,

    /// Treat path as raw config string instead of a file path
    pub path_is_stream: bool,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            throw_all_errors: false,
            allow_dynamic_parsing: true,
            base_dir: None,
            verify_only: false,
            allow_missing_config: false,
            path_is_stream: false,
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
            variable_lines: HashMap::new(),
            record_variable_lines: true,
            #[cfg(feature = "mutation")]
            document: None,
            #[cfg(feature = "mutation")]
            source_file: None,
            #[cfg(feature = "mutation")]
            multi_document: None,
            #[cfg(feature = "mutation")]
            current_source_file: None,
        }
    }

    /// Create a new configuration with custom options
    pub fn with_options(options: ConfigOptions) -> Self {
        let source_resolver = options.base_dir.as_ref().map(SourceResolver::new);

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
            variable_lines: HashMap::new(),
            record_variable_lines: true,
            #[cfg(feature = "mutation")]
            document: None,
            #[cfg(feature = "mutation")]
            source_file: None,
            #[cfg(feature = "mutation")]
            multi_document: None,
            #[cfg(feature = "mutation")]
            current_source_file: None,
        }
    }

    /// Initialize the configuration (called before parsing)
    pub fn commence(&mut self) -> ParseResult<()> {
        self.errors.clear();
        self.directives.reset();
        self.current_path.clear();
        Ok(())
    }

    fn reset_top_level_state(&mut self) {
        self.errors.clear();
        self.directives.reset();
        self.current_path.clear();
        self.values.clear();
        self.handler_calls.clear();
        self.variables.clear();
        self.variable_lines.clear();
        self.record_variable_lines = true;
        self.expressions = ExpressionEvaluator::new();
        self.special_categories.reset_for_parse();

        if let Some(resolver) = &mut self.source_resolver {
            resolver.reset();
        }

        #[cfg(feature = "mutation")]
        {
            self.document = None;
            self.source_file = None;
            self.multi_document = None;
            self.current_source_file = None;
        }
    }

    /// Parse a configuration file
    pub fn parse_file(&mut self, path: impl AsRef<Path>) -> ParseResult<()> {
        let path = path.as_ref();

        // If path_is_stream, treat the path string as raw config content
        if self.options.path_is_stream {
            let content = path.to_string_lossy().to_string();
            return self.parse(&content);
        }

        self.reset_top_level_state();

        // Check if file exists - handle allow_missing_config
        if !path.exists() {
            if self.options.allow_missing_config {
                return Ok(());
            }
            return Err(ConfigError::io(
                path.display().to_string(),
                "file not found",
            ));
        }

        let canonical_path = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        if let Some(parent) = path.parent() {
            self.options.base_dir = Some(parent.to_path_buf());
            self.source_resolver = Some(SourceResolver::new(parent));
        }

        // Initialize multi_document if this is the primary file
        #[cfg(feature = "mutation")]
        let is_primary = self.multi_document.is_none();

        #[cfg(feature = "mutation")]
        if is_primary {
            self.multi_document = Some(crate::document::MultiFileDocument::new(
                canonical_path.clone(),
            ));
            self.source_file = Some(canonical_path.clone());
        }

        // Parse the file with path tracking
        self.parse_file_internal(&canonical_path, true)
    }

    /// Internal method to parse a file with path tracking
    fn parse_file_internal(&mut self, path: &Path, top_level: bool) -> ParseResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::io(path.display().to_string(), e.to_string()))?;

        // Set current source file for key tracking
        #[cfg(feature = "mutation")]
        {
            self.current_source_file = Some(path.to_path_buf());
        }

        // Parse the content
        self.parse_with_path(&content, Some(path), top_level)
    }

    /// Parse content with an associated file path
    fn parse_with_path(
        &mut self,
        input: &str,
        _source_path: Option<&Path>,
        top_level: bool,
    ) -> ParseResult<()> {
        if top_level {
            self.commence()?;
        }

        #[cfg(feature = "mutation")]
        let previous_source_file = self.current_source_file.clone();

        #[cfg(feature = "mutation")]
        let (parsed, mut document) = HyprlangParser::parse_with_document(input)?;
        #[cfg(not(feature = "mutation"))]
        let parsed = HyprlangParser::parse_config(input)?;

        #[cfg(feature = "mutation")]
        {
            // Set the source path on the document
            if let Some(path) = _source_path {
                document.source_path = Some(path.to_path_buf());
            }

            // Store document in multi_document if available
            if let (Some(multi_doc), Some(path)) = (&mut self.multi_document, _source_path) {
                multi_doc.add_document(path.to_path_buf(), document.clone());
            }

            // Also keep backward-compatible single document
            if top_level || _source_path.is_none() {
                self.document = Some(document.clone());
            }
        }

        for statement in parsed.statements {
            if let Err(e) = self.process_statement(&statement) {
                if self.directives.should_suppress_errors() {
                    continue;
                } else if self.options.throw_all_errors {
                    self.errors.push(e);
                } else {
                    #[cfg(feature = "mutation")]
                    {
                        self.current_source_file = previous_source_file;
                    }
                    return Err(e);
                }
            }
        }

        if !self.errors.is_empty() {
            #[cfg(feature = "mutation")]
            {
                self.current_source_file = previous_source_file;
            }
            return Err(ConfigError::multiple(std::mem::take(&mut self.errors)));
        }

        #[cfg(feature = "mutation")]
        {
            self.current_source_file = previous_source_file;
        }

        Ok(())
    }

    /// Parse a configuration string
    pub fn parse(&mut self, input: &str) -> ParseResult<()> {
        self.reset_top_level_state();
        self.parse_with_path(input, None, true)
    }

    /// Parse a single line dynamically (after initial parse)
    ///
    /// If a variable is being set, all lines that previously referenced that variable
    /// will be re-evaluated with the new value (dynamic variable propagation).
    ///
    /// Supports bracket syntax for special categories: `special[key]:prop = value`
    pub fn parse_dynamic(&mut self, line: &str) -> ParseResult<()> {
        if !self.options.allow_dynamic_parsing {
            return Err(ConfigError::custom("Dynamic parsing is not enabled"));
        }

        // Handle special category bracket syntax: special[key]:prop = value
        if let Some(result) = self.try_parse_dynamic_special_category(line)? {
            return Ok(result);
        }

        let parsed = HyprlangParser::parse_config(line)?;
        let previous_recording = std::mem::replace(&mut self.record_variable_lines, false);

        let result = (|| {
            for statement in parsed.statements {
                // Check if this is a variable definition - we'll need to propagate
                let var_name = if let Statement::VariableDef { ref name, .. } = statement {
                    Some(name.clone())
                } else {
                    None
                };

                self.process_statement(&statement)?;

                // If a variable was set, re-evaluate all dependent lines
                if let Some(name) = var_name {
                    self.propagate_variable(&name)?;
                }
            }
            Ok(())
        })();

        self.record_variable_lines = previous_recording;
        result
    }

    /// Parse a dynamic assignment with separate key and value arguments.
    ///
    /// Equivalent to `parse_dynamic("key = value")`.
    pub fn parse_dynamic_kv(&mut self, key: &str, value: &str) -> ParseResult<()> {
        self.parse_dynamic(&format!("{} = {}", key, value))
    }

    /// Try to handle bracket syntax in dynamic parsing: `special[key]:prop = value`
    fn try_parse_dynamic_special_category(&mut self, line: &str) -> ParseResult<Option<()>> {
        // Look for pattern: name[key]:prop = value
        let Some(bracket_start) = line.find('[') else {
            return Ok(None);
        };
        let Some(bracket_end) = line[bracket_start..].find(']') else {
            return Ok(None);
        };
        let bracket_end = bracket_start + bracket_end;

        // Extract parts
        let cat_name = line[..bracket_start].trim();
        let instance_key = &line[bracket_start + 1..bracket_end];

        let rest = &line[bracket_end + 1..];
        if !rest.starts_with(':') {
            return Ok(None);
        }
        let rest = &rest[1..]; // skip ':'

        let Some(eq_pos) = rest.find('=') else {
            return Ok(None);
        };
        let prop_name = rest[..eq_pos].trim();
        let raw_value = rest[eq_pos + 1..].trim();

        // Check if this is a registered special category
        if !self.special_categories.is_registered(cat_name) {
            return Ok(None);
        }

        let resolved_value = self.resolve_raw_string(raw_value, true)?;
        let full_key = format!("{}[{}]:{}", cat_name, instance_key, prop_name);
        let _ = self.store_special_assignment(
            cat_name,
            instance_key,
            prop_name,
            &resolved_value,
            full_key,
        )?;

        Ok(Some(()))
    }

    /// Re-evaluate all lines that reference the given variable
    fn propagate_variable(&mut self, var_name: &str) -> ParseResult<()> {
        if let Some(lines) = self.variable_lines.get(var_name).cloned() {
            let previous_recording = std::mem::replace(&mut self.record_variable_lines, false);
            for var_line in &lines {
                let saved_path =
                    std::mem::replace(&mut self.current_path, var_line.category_path.clone());
                let _ = self.process_statement(&var_line.statement);
                self.current_path = saved_path;
            }
            self.record_variable_lines = previous_recording;
        }
        Ok(())
    }

    /// Track which variables are referenced in a value string
    fn track_variable_references(&mut self, value: &str, statement: &Statement) {
        if !self.record_variable_lines {
            return;
        }

        let var_line = VariableLine {
            statement: statement.clone(),
            category_path: self.current_path.clone(),
        };

        // Extract all variable names referenced in the value
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' {
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !name.is_empty() {
                    self.variable_lines
                        .entry(name)
                        .or_default()
                        .push(var_line.clone());
                }
            }
        }
    }

    fn split_path_segment(segment: &str) -> (String, Option<String>) {
        if let Some(start) = segment.find('[')
            && segment.ends_with(']')
            && start < segment.len() - 1
        {
            return (
                segment[..start].to_string(),
                Some(segment[start + 1..segment.len() - 1].to_string()),
            );
        }

        (segment.to_string(), None)
    }

    fn plain_current_path(&self) -> Vec<String> {
        self.current_path
            .iter()
            .map(|segment| Self::split_path_segment(segment).0)
            .collect()
    }

    fn active_special_context(&self) -> Option<(String, String, Vec<String>)> {
        let plain_path = self.plain_current_path();

        for idx in (0..self.current_path.len()).rev() {
            let (_, maybe_key) = Self::split_path_segment(&self.current_path[idx]);
            let Some(instance_key) = maybe_key else {
                continue;
            };

            let category_name = plain_path[..=idx].join(":");
            if self.special_categories.is_registered(&category_name) {
                return Some((category_name, instance_key, plain_path[idx + 1..].to_vec()));
            }
        }

        None
    }

    fn resolve_special_assignment(
        &mut self,
        key: &[String],
        resolved_value: &str,
    ) -> ParseResult<Option<(String, String, String, String)>> {
        if let Some((category_name, instance_key, nested_path)) = self.active_special_context() {
            let mut property_segments = nested_path;
            property_segments.extend(key.iter().cloned());
            let property_name = property_segments.join(":");
            let storage_key = format!("{}[{}]:{}", category_name, instance_key, property_name);
            return Ok(Some((category_name, instance_key, property_name, storage_key)));
        }

        let plain_path = self.plain_current_path();
        let mut combined_path = plain_path.clone();
        combined_path.extend(key.iter().cloned());

        let Some((category_name, prefix_len)) = self
            .special_categories
            .find_matching_category_segments(&combined_path)
        else {
            return Ok(None);
        };

        let descriptor = self
            .special_categories
            .get_descriptor(&category_name)
            .cloned()
            .ok_or_else(|| ConfigError::category_not_found(&category_name, None))?;
        let property_segments = combined_path[prefix_len..].to_vec();
        if property_segments.is_empty() {
            return Ok(None);
        }
        let property_name = property_segments.join(":");

        match descriptor.category_type {
            crate::special_categories::SpecialCategoryType::Static => Ok(Some((
                category_name.clone(),
                "static".to_string(),
                property_name.clone(),
                format!("{}:{}", category_name, property_name),
            ))),
            crate::special_categories::SpecialCategoryType::Keyed => {
                if plain_path.len() < prefix_len {
                    return Ok(None);
                }

                let key_field = descriptor.key_field.clone().ok_or_else(|| {
                    ConfigError::custom(format!(
                        "Keyed category '{}' is missing its key field descriptor",
                        category_name
                    ))
                })?;

                let (_, existing_key) = Self::split_path_segment(&self.current_path[prefix_len - 1]);
                let instance_key = if let Some(existing_key) = existing_key {
                    existing_key
                } else {
                    if property_name != key_field {
                        return Err(ConfigError::custom(format!(
                            "special category's first value must be the key. Key for <{}> is <{}>",
                            category_name, key_field
                        )));
                    }

                    let instance_key = resolved_value.to_string();
                    self.special_categories
                        .create_instance(&category_name, Some(instance_key.clone()))?;
                    self.current_path[prefix_len - 1] =
                        format!("{}[{}]", plain_path[prefix_len - 1], instance_key);
                    instance_key
                };

                Ok(Some((
                    category_name.clone(),
                    instance_key.clone(),
                    property_name.clone(),
                    format!("{}[{}]:{}", category_name, instance_key, property_name),
                )))
            }
            crate::special_categories::SpecialCategoryType::Anonymous => {
                if plain_path.len() < prefix_len {
                    return Ok(None);
                }

                let (_, existing_key) = Self::split_path_segment(&self.current_path[prefix_len - 1]);
                let instance_key = if let Some(existing_key) = existing_key {
                    existing_key
                } else {
                    let instance_key = self.special_categories.create_instance(&category_name, None)?;
                    self.current_path[prefix_len - 1] =
                        format!("{}[{}]", plain_path[prefix_len - 1], instance_key);
                    instance_key
                };

                Ok(Some((
                    category_name.clone(),
                    instance_key.clone(),
                    property_name.clone(),
                    format!("{}[{}]:{}", category_name, instance_key, property_name),
                )))
            }
        }
    }

    fn store_special_assignment(
        &mut self,
        category_name: &str,
        instance_key: &str,
        property_name: &str,
        resolved_value: &str,
        storage_key: String,
    ) -> ParseResult<bool> {
        let descriptor = self
            .special_categories
            .get_descriptor(category_name)
            .cloned()
            .ok_or_else(|| ConfigError::category_not_found(category_name, None))?;

        let config_value = if descriptor.key_field.as_deref() == Some(property_name) {
            ConfigValue::String(resolved_value.to_string())
        } else if let Some(default_value) = descriptor.default_values.get(property_name) {
            self.parse_value_for_expected_type(resolved_value, default_value)?
        } else if descriptor.ignore_missing {
            return Ok(false);
        } else {
            return Err(ConfigError::custom(format!(
                "config option <{}> does not exist.",
                storage_key
            )));
        };

        if !self.options.verify_only {
            match descriptor.category_type {
                crate::special_categories::SpecialCategoryType::Static => {
                    self.special_categories.create_instance(category_name, None)?;
                }
                crate::special_categories::SpecialCategoryType::Keyed
                | crate::special_categories::SpecialCategoryType::Anonymous => {
                    self.special_categories
                        .create_instance(category_name, Some(instance_key.to_string()))?;
                }
            }

            let entry = ConfigValueEntry::new(config_value, resolved_value.to_string());
            if let Ok(instance) = self
                .special_categories
                .get_instance_mut(category_name, instance_key)
            {
                instance.set(property_name.to_string(), entry.clone());
            }
            self.values.insert(storage_key, entry);
        }

        Ok(!self.options.verify_only)
    }

    fn process_statement(&mut self, statement: &Statement) -> ParseResult<()> {
        // Check if we should execute this statement based on directives
        if !self.directives.should_execute() {
            // Still need to process directives even when not executing
            if let Statement::CommentDirective {
                directive_type,
                args,
            } = statement
            {
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
                let expanded = self.resolve_raw_string_with_restore(value, true, false)?;

                // Track which variables this definition references (for propagation)
                if value.contains('$') {
                    self.track_variable_references(value, statement);
                }

                // Track variable origin in multi_document
                #[cfg(feature = "mutation")]
                if let (Some(multi_doc), Some(source_file)) =
                    (&mut self.multi_document, &self.current_source_file)
                {
                    multi_doc.register_key(format!("${}", name), source_file.clone());
                }

                if !self.options.verify_only {
                    self.variables.set(name.clone(), expanded.clone());

                    // Update expression evaluator if it's a number
                    if let Ok(num) = ConfigValue::parse_int(&expanded) {
                        self.expressions.set_variable(name.clone(), num);
                    }
                }

                Ok(())
            }

            Statement::Assignment { key, value } => {
                // Track variable references for dynamic propagation
                let value_str = self.value_to_string(value);
                if value_str.contains('$') {
                    self.track_variable_references(&value_str, statement);
                }

                let resolved_value = self.resolve_value_to_string(value)?;

                if let Some((category_name, instance_key, property_name, storage_key)) =
                    self.resolve_special_assignment(key, &resolved_value)?
                {
                    let _stored = self.store_special_assignment(
                        &category_name,
                        &instance_key,
                        &property_name,
                        &resolved_value,
                        storage_key.clone(),
                    )?;

                    #[cfg(feature = "mutation")]
                    if _stored
                        && let (Some(multi_doc), Some(source_file)) =
                            (&mut self.multi_document, &self.current_source_file)
                    {
                        multi_doc.register_key(storage_key.clone(), source_file.clone());
                    }
                } else if key.len() == 1
                    && let Some(resolved_handler) =
                        self.handlers.resolve_invocation(&self.current_path, &key[0])
                {
                    // Treat as handler call
                    // Create full key including category path for handler calls
                    let full_key = if self.current_path.is_empty() {
                        resolved_handler.handler_keyword.clone()
                    } else {
                        format!(
                            "{}:{}",
                            self.current_path.join(":"),
                            resolved_handler.handler_keyword
                        )
                    };

                    self.handler_calls
                        .entry(full_key.clone())
                        .or_default()
                        .push(resolved_value.clone());

                    // Track handler origin in multi_document
                    #[cfg(feature = "mutation")]
                    if let (Some(multi_doc), Some(source_file)) =
                        (&mut self.multi_document, &self.current_source_file)
                    {
                        multi_doc.register_handler(full_key, source_file.clone());
                    }

                    self.handlers
                        .execute_resolved(&self.current_path, &resolved_handler, &resolved_value)?;
                } else {
                    // Regular assignment
                    let full_key = self.make_full_key(key);
                    let config_value = self.parse_config_value(value)?;

                    if !self.options.verify_only {
                        let raw = self.value_to_string(value);

                        // Track key origin in multi_document
                        #[cfg(feature = "mutation")]
                        if let (Some(multi_doc), Some(source_file)) =
                            (&mut self.multi_document, &self.current_source_file)
                        {
                            multi_doc.register_key(full_key.clone(), source_file.clone());
                        }

                        self.values
                            .insert(full_key, ConfigValueEntry::new(config_value, raw));
                    }
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

            Statement::SpecialCategoryBlock {
                name,
                key,
                statements,
            } => {
                if key.is_none() {
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
                    return Ok(());
                }

                if !self.special_categories.is_registered(name) {
                    return Err(ConfigError::category_not_found(name, None));
                }

                let instance_key = self
                    .special_categories
                    .create_instance(name, key.clone())?;

                self.current_path
                    .push(format!("{}[{}]", name, instance_key));

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

                self.current_path.pop();
                Ok(())
            }

            Statement::HandlerCall {
                keyword,
                flags,
                value,
            } => {
                let expanded_value = self.resolve_raw_string(value, true)?;

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
                        .entry(full_key.clone())
                        .or_default()
                        .push(expanded_value.clone());

                    // Track handler origin in multi_document
                    #[cfg(feature = "mutation")]
                    if let (Some(multi_doc), Some(source_file)) =
                        (&mut self.multi_document, &self.current_source_file)
                    {
                        multi_doc.register_handler(full_key, source_file.clone());
                    }
                }

                // Execute the handler if one is registered
                self.handlers
                    .execute(&self.current_path, keyword, &expanded_value, flags.clone())
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

                // Canonicalize the resolved path
                let canonical_resolved = resolved
                    .canonicalize()
                    .unwrap_or_else(|_| resolved.clone());

                // Parse the sourced file using internal method (avoids re-initializing multi_document)
                let result = self.parse_file_internal(&canonical_resolved, false);

                // End load
                if let Some(resolver) = &mut self.source_resolver {
                    resolver.end_load();
                }

                result
            }

            Statement::CommentDirective {
                directive_type,
                args,
            } => {
                self.directives
                    .process_directive(directive_type, args.as_deref(), &self.variables)
            }
        }
    }

    fn resolve_raw_string_with_restore(
        &mut self,
        raw: &str,
        evaluate_expressions: bool,
        restore_escaped: bool,
    ) -> ParseResult<String> {
        let escaped = process_escapes(raw);
        let expanded = self.variables.expand(&escaped)?;
        let with_exprs = if evaluate_expressions {
            self.evaluate_expressions_in_string(&expanded)?
        } else {
            expanded
        };

        if restore_escaped {
            Ok(restore_escaped_braces(&with_exprs))
        } else {
            Ok(with_exprs)
        }
    }

    fn resolve_raw_string(&mut self, raw: &str, evaluate_expressions: bool) -> ParseResult<String> {
        self.resolve_raw_string_with_restore(raw, evaluate_expressions, true)
    }

    fn resolve_value_to_string(&mut self, value: &Value) -> ParseResult<String> {
        match value {
            Value::Expression(expr) => Ok(self.expressions.evaluate(expr)?.to_string()),
            Value::Variable(name) => self.variables.expand(&format!("${}", name)),
            Value::Color(color) => Ok(color.to_string()),
            Value::Vec2(vec) => Ok(vec.to_string()),
            Value::Number(num) => Ok(num.clone()),
            Value::Boolean(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            Value::String(s) => self.resolve_raw_string(s, true),
            Value::Multiline(lines) => {
                let joined = MultilineProcessor::join_lines(lines);
                self.resolve_raw_string(&joined, true)
            }
        }
    }

    fn parse_int_value(&self, s: &str) -> ParseResult<ConfigValue> {
        if let Ok(b) = ConfigValue::parse_bool(s) {
            return Ok(ConfigValue::Int(if b { 1 } else { 0 }));
        }

        if s.starts_with("rgba(") && s.ends_with(')') {
            return self
                .parse_rgba_string(s)
                .map(|color| ConfigValue::Int(color.to_argb() as i64));
        }

        if s.starts_with("rgb(") && s.ends_with(')') {
            return self
                .parse_rgb_string(s)
                .map(|color| ConfigValue::Int(color.to_argb() as i64));
        }

        Ok(ConfigValue::Int(ConfigValue::parse_int(s)?))
    }

    fn parse_value_for_expected_type(
        &mut self,
        resolved_value: &str,
        expected: &ConfigValue,
    ) -> ParseResult<ConfigValue> {
        match expected {
            ConfigValue::Int(_) => self.parse_int_value(resolved_value),
            ConfigValue::Float(_) => Ok(ConfigValue::Float(ConfigValue::parse_float(
                resolved_value,
            )?)),
            ConfigValue::String(_) => Ok(ConfigValue::String(resolved_value.to_string())),
            ConfigValue::Vec2(_) => Ok(ConfigValue::Vec2(self.parse_vec2_string(resolved_value)?)),
            ConfigValue::Color(_) => {
                if resolved_value.starts_with("rgba(") && resolved_value.ends_with(')') {
                    return Ok(ConfigValue::Color(self.parse_rgba_string(resolved_value)?));
                }
                if resolved_value.starts_with("rgb(") && resolved_value.ends_with(')') {
                    return Ok(ConfigValue::Color(self.parse_rgb_string(resolved_value)?));
                }
                Ok(ConfigValue::Color(Color::from_hex(resolved_value)?))
            }
            ConfigValue::Custom { type_name, .. } => {
                let handler = self
                    .custom_types
                    .get(type_name)
                    .ok_or_else(|| ConfigError::custom(format!("Unknown custom type '{}'", type_name)))?;
                let value = handler.parse(resolved_value)?;
                Ok(ConfigValue::Custom {
                    type_name: type_name.clone(),
                    value: Rc::from(value),
                })
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
                let resolved = self.resolve_raw_string(s, true)?;
                self.parse_string_value(&resolved)
            }

            Value::Multiline(lines) => {
                let joined = MultilineProcessor::join_lines(lines);
                let final_value = self.resolve_raw_string(&joined, true)?;
                Ok(ConfigValue::String(final_value))
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

    /// Evaluate all {{expr}} expressions in a string
    fn evaluate_expressions_in_string(&self, input: &str) -> ParseResult<String> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second {

                    // Find the closing }}
                    let mut expr = String::new();
                    let mut depth = 1;

                    while let Some(c) = chars.next() {
                        if c == '{' && chars.peek() == Some(&'{') {
                            depth += 1;
                            expr.push(c);
                            chars.next();
                            expr.push('{');
                        } else if c == '}' && chars.peek() == Some(&'}') {
                            depth -= 1;
                            if depth == 0 {
                                chars.next(); // consume second }
                                break;
                            }
                            expr.push(c);
                            chars.next();
                            expr.push('}');
                        } else {
                            expr.push(c);
                        }
                    }

                    // Evaluate the expression
                    let value = self.expressions.evaluate(&expr)?;
                    result.push_str(&value.to_string());
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn parse_rgba_string(&self, s: &str) -> ParseResult<Color> {
        // rgba(hex) or rgba(r, g, b, a)
        let inner = &s[5..s.len() - 1]; // Remove "rgba(" and ")"

        if !inner.contains(',') {
            // Hex format: rgba(RRGGBBAA)
            Color::from_hex(inner)
        } else {
            // Component format: rgba(r, g, b, a)
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() != 4 {
                return Err(ConfigError::invalid_color(s, "rgba needs 4 components"));
            }

            let r = parts[0]
                .parse::<u8>()
                .map_err(|_| ConfigError::invalid_color(s, "invalid r"))?;
            let g = parts[1]
                .parse::<u8>()
                .map_err(|_| ConfigError::invalid_color(s, "invalid g"))?;
            let b = parts[2]
                .parse::<u8>()
                .map_err(|_| ConfigError::invalid_color(s, "invalid b"))?;

            // Alpha can be float (0.0-1.0) or int (0-255)
            let a = if parts[3].contains('.') {
                let a_float = parts[3]
                    .parse::<f64>()
                    .map_err(|_| ConfigError::invalid_color(s, "invalid a"))?;
                (a_float * 255.0).round() as u8
            } else {
                parts[3]
                    .parse::<u8>()
                    .map_err(|_| ConfigError::invalid_color(s, "invalid a"))?
            };

            Ok(Color::from_rgba(r, g, b, a))
        }
    }

    fn parse_rgb_string(&self, s: &str) -> ParseResult<Color> {
        // rgb(r, g, b)
        let inner = &s[4..s.len() - 1]; // Remove "rgb(" and ")"
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

        if parts.len() != 3 {
            return Err(ConfigError::invalid_color(s, "rgb needs 3 components"));
        }

        let r = parts[0]
            .parse::<u8>()
            .map_err(|_| ConfigError::invalid_color(s, "invalid r"))?;
        let g = parts[1]
            .parse::<u8>()
            .map_err(|_| ConfigError::invalid_color(s, "invalid g"))?;
        let b = parts[2]
            .parse::<u8>()
            .map_err(|_| ConfigError::invalid_color(s, "invalid b"))?;

        Ok(Color::from_rgb(r, g, b))
    }

    fn parse_vec2_string(&self, s: &str) -> ParseResult<Vec2> {
        // Try (x, y) format
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

            if parts.len() == 2
                && let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>())
            {
                return Ok(Vec2::new(x, y));
            }
        } else if s.contains(',') {
            // Try x, y format without parentheses
            let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();

            if parts.len() == 2
                && let (Ok(x), Ok(y)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>())
            {
                return Ok(Vec2::new(x, y));
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

    /// Change the root path for resolving source directives.
    ///
    /// This updates the base directory used for resolving relative paths in
    /// `source = path` directives. Useful when re-parsing from a different location.
    pub fn change_root_path(&mut self, path: &Path) {
        let base_dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };

        self.options.base_dir = Some(base_dir.clone());
        self.source_resolver = Some(SourceResolver::new(base_dir));
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> ParseResult<&ConfigValue> {
        self.values
            .get(key)
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

        // Update document tree if mutation feature is enabled
        #[cfg(feature = "mutation")]
        {
            // Try to update in the correct source file using multi_document
            let updated_in_multi = if let Some(multi_doc) = &mut self.multi_document {
                // Find which file this key belongs to
                let source_file = multi_doc
                    .get_key_source(&key)
                    .cloned()
                    .unwrap_or_else(|| multi_doc.primary_path.clone());

                // Update the document in that file
                if let Some(doc) = multi_doc.get_document_mut(&source_file) {
                    let _ = doc.update_or_insert_value(&key, &raw);
                    multi_doc.mark_dirty(&source_file);

                    // If this is a new key, register it with the primary file
                    if multi_doc.get_key_source(&key).is_none() {
                        multi_doc.register_key(key.clone(), source_file);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Fallback: update single document if multi_document didn't handle it
            if !updated_in_multi
                && let Some(doc) = &mut self.document
            {
                let _ = doc.update_or_insert_value(&key, &raw);
            }
        }

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
        self.handlers
            .register_global(keyword.clone(), FunctionHandler::new(keyword, handler));
    }

    /// Unregister a previously registered handler by keyword.
    ///
    /// Returns `true` if the handler was found and removed.
    pub fn unregister_handler(&mut self, keyword: &str) -> bool {
        self.handlers.unregister_global(keyword)
    }

    /// Unregister a category-specific handler.
    ///
    /// Returns `true` if the handler was found and removed.
    pub fn unregister_category_handler(&mut self, category: &str, keyword: &str) -> bool {
        self.handlers.unregister_category(category, keyword)
    }

    /// Register a category-specific handler
    pub fn register_category_handler<H>(
        &mut self,
        category: impl Into<String>,
        keyword: impl Into<String>,
        handler: H,
    ) where
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
    ) where
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

    /// Register a default value for a special category property
    /// This adds a default value that will be applied to all instances of the category
    pub fn register_special_category_value(
        &mut self,
        category: impl Into<String>,
        property: impl Into<String>,
        default_value: ConfigValue,
    ) {
        let category = category.into();
        let property = property.into();
        let _ = self
            .special_categories
            .add_default_value(&category, property, default_value);
    }

    /// Remove a default value from a special category and all instantiated categories.
    pub fn remove_special_category_value(
        &mut self,
        category: &str,
        property: &str,
    ) -> ParseResult<()> {
        self.special_categories
            .remove_default_value(category, property)
    }

    /// Remove a registered special category and all of its instances.
    pub fn remove_special_category(&mut self, category: &str) {
        self.special_categories.remove_category(category);
    }

    /// Get a special category instance
    pub fn get_special_category(
        &self,
        category: &str,
        key: &str,
    ) -> ParseResult<HashMap<String, &ConfigValue>> {
        let instance = self.special_categories.get_instance(category, key)?;
        let mut result = HashMap::new();

        for (k, v) in &instance.values {
            result.insert(k.clone(), &v.value);
        }

        Ok(result)
    }

    /// Check if a special category instance exists for a given key.
    pub fn special_category_exists_for_key(&self, category: &str, key: &str) -> bool {
        self.special_categories.instance_exists(category, key)
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
            self.expressions.set_variable(name.clone(), num);
        }

        // Update document tree if mutation feature is enabled
        #[cfg(feature = "mutation")]
        {
            let var_key = format!("${}", name);

            // Try to update in the correct source file using multi_document
            let updated_in_multi = if let Some(multi_doc) = &mut self.multi_document {
                // Find which file this variable belongs to
                let source_file = multi_doc
                    .get_key_source(&var_key)
                    .cloned()
                    .unwrap_or_else(|| multi_doc.primary_path.clone());

                // Update the document in that file
                if let Some(doc) = multi_doc.get_document_mut(&source_file) {
                    let _ = doc.update_or_insert_variable(&name, &value);
                    multi_doc.mark_dirty(&source_file);

                    // If this is a new variable, register it with the primary file
                    if multi_doc.get_key_source(&var_key).is_none() {
                        multi_doc.register_key(var_key, source_file);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Fallback: update single document if multi_document didn't handle it
            if !updated_in_multi
                && let Some(doc) = &mut self.document
            {
                let _ = doc.update_or_insert_variable(&name, &value);
            }
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

    // ========== MUTATION METHODS (mutation feature) ==========

    /// Set an integer configuration value.
    ///
    /// This is a convenience method for [`set`](Config::set) that wraps the value in [`ConfigValue::Int`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.set_int("border_size", 5);
    /// assert_eq!(config.get_int("border_size").unwrap(), 5);
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn set_int(&mut self, key: impl Into<String>, value: i64) {
        self.set(key, ConfigValue::Int(value))
    }

    /// Set a float configuration value.
    ///
    /// This is a convenience method for [`set`](Config::set) that wraps the value in [`ConfigValue::Float`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.set_float("opacity", 0.95);
    /// assert_eq!(config.get_float("opacity").unwrap(), 0.95);
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn set_float(&mut self, key: impl Into<String>, value: f64) {
        self.set(key, ConfigValue::Float(value))
    }

    /// Set a string configuration value.
    ///
    /// This is a convenience method for [`set`](Config::set) that wraps the value in [`ConfigValue::String`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.set_string("terminal", "kitty");
    /// assert_eq!(config.get_string("terminal").unwrap(), "kitty");
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.set(key, ConfigValue::String(value.into()))
    }

    /// Remove a configuration value and return it.
    ///
    /// Returns an error if the key doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, ConfigValue};
    ///
    /// let mut config = Config::new();
    /// config.set_int("test", 42);
    ///
    /// let removed = config.remove("test").unwrap();
    /// assert_eq!(removed.as_int().unwrap(), 42);
    /// assert!(!config.contains("test"));
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn remove(&mut self, key: &str) -> ParseResult<ConfigValue> {
        let entry = self
            .values
            .remove(key)
            .ok_or_else(|| ConfigError::key_not_found(key))?;

        #[cfg(feature = "mutation")]
        {
            if let Some(doc) = &mut self.document {
                let _ = doc.remove_value(key);
            }
        }

        Ok(entry.value)
    }

    // ========== VARIABLE MUTATIONS ==========

    /// Get a mutable reference to a variable.
    ///
    /// Returns a [`MutableVariable`](crate::MutableVariable) that allows you to read and modify
    /// the variable value. Returns `None` if the variable doesn't exist.
    ///
    /// This provides an alternative API to [`set_variable`](Config::set_variable) that allows
    /// you to work with variables using a reference-based approach.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse("$GAPS = 10").unwrap();
    ///
    /// // Get mutable reference and modify
    /// if let Some(mut gaps) = config.get_variable_mut("GAPS") {
    ///     assert_eq!(gaps.get(), "10");
    ///     gaps.set("20").unwrap();
    ///     assert_eq!(gaps.get(), "20");
    /// }
    ///
    /// assert_eq!(config.get_variable("GAPS"), Some("20"));
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn get_variable_mut(&mut self, name: &str) -> Option<crate::mutation::MutableVariable<'_>> {
        if self.variables.contains(name) {
            // We need to use unsafe here to work around the borrow checker
            // This is safe because we're only accessing disjoint fields
            let manager_ptr = &mut self.variables as *mut VariableManager;
            let doc_ptr = &mut self.document as *mut Option<crate::document::ConfigDocument>;

            unsafe {
                Some(crate::mutation::MutableVariable::new(
                    name.to_string(),
                    &mut *manager_ptr,
                    (*doc_ptr).as_mut(),
                ))
            }
        } else {
            None
        }
    }

    /// Remove a variable and return its value.
    ///
    /// Returns the variable value if it existed, or `None` if it didn't.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse("$OLD = value").unwrap();
    ///
    /// let removed = config.remove_variable("OLD");
    /// assert_eq!(removed, Some("value".to_string()));
    /// assert_eq!(config.get_variable("OLD"), None);
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn remove_variable(&mut self, name: &str) -> Option<String> {
        let value = self.variables.remove(name);

        #[cfg(feature = "mutation")]
        {
            if let Some(doc) = &mut self.document {
                let _ = doc.remove_variable(name);
            }
        }

        value
    }

    // ========== HANDLER MUTATIONS ==========

    /// Add a handler call.
    ///
    /// Handler calls are stored as arrays, so you can add multiple calls for the same handler.
    /// The handler must be registered before parsing, but you can add calls dynamically after parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.register_handler_fn("bind", |_| Ok(()));
    ///
    /// config.add_handler_call("bind", "SUPER, Q, exec, terminal".to_string()).unwrap();
    /// config.add_handler_call("bind", "SUPER, C, killactive".to_string()).unwrap();
    ///
    /// let binds = config.get_handler_calls("bind").unwrap();
    /// assert_eq!(binds.len(), 2);
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn add_handler_call(
        &mut self,
        handler: impl Into<String>,
        value: String,
    ) -> ParseResult<()> {
        let handler = handler.into();

        // Update in-memory state
        self.handler_calls
            .entry(handler.clone())
            .or_default()
            .push(value.clone());

        #[cfg(feature = "mutation")]
        {
            // Try to update in the correct source file using multi_document
            let updated_in_multi = if let Some(multi_doc) = &mut self.multi_document {
                // Find which file has this handler, or use primary file
                let source_file = multi_doc
                    .get_handler_source(&handler)
                    .cloned()
                    .unwrap_or_else(|| multi_doc.primary_path.clone());

                if let Some(doc) = multi_doc.get_document_mut(&source_file) {
                    let _ = doc.add_handler_call(&handler, &value);
                    multi_doc.mark_dirty(&source_file);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Fallback: update single document if multi_document didn't handle it
            if !updated_in_multi {
                if let Some(doc) = &mut self.document {
                    let _ = doc.add_handler_call(&handler, &value);
                }
            }
        }

        Ok(())
    }

    /// Remove all handler calls for a specific handler.
    ///
    /// Returns the array of handler call values if the handler had any calls, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.register_handler_fn("bind", |_| Ok(()));
    /// config.parse("bind = SUPER, Q, exec, terminal\nbind = SUPER, C, killactive").unwrap();
    ///
    /// let removed = config.remove_handler_calls("bind");
    /// assert_eq!(removed.unwrap().len(), 2);
    /// assert!(config.get_handler_calls("bind").is_none());
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn remove_handler_calls(&mut self, handler: &str) -> Option<Vec<String>> {
        // TODO: Remove from document tree
        // if let Some(doc) = &mut self.document {
        //     let _ = doc.remove_handler_calls(handler);
        // }

        self.handler_calls.remove(handler)
    }

    /// Remove a specific handler call by index.
    ///
    /// Returns an error if the handler doesn't exist or if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.register_handler_fn("bind", |_| Ok(()));
    /// config.parse("bind = SUPER, Q, exec, terminal\nbind = SUPER, C, killactive").unwrap();
    ///
    /// // Remove the first bind
    /// let removed = config.remove_handler_call("bind", 0).unwrap();
    /// assert_eq!(removed, "SUPER, Q, exec, terminal");
    ///
    /// // Only one bind remains
    /// assert_eq!(config.get_handler_calls("bind").unwrap().len(), 1);
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn remove_handler_call(&mut self, handler: &str, index: usize) -> ParseResult<String> {
        let calls = self
            .handler_calls
            .get_mut(handler)
            .ok_or_else(|| ConfigError::handler(handler, "no calls found"))?;

        if index >= calls.len() {
            return Err(ConfigError::custom("index out of bounds"));
        }

        let value = calls.remove(index);

        // Remove from document tree for serialization consistency
        // Try multi_document first, then fall back to single document
        let removed_in_multi = if let Some(multi_doc) = &mut self.multi_document {
            // Find which file has this handler
            if let Some(source_file) = multi_doc.get_handler_source(handler).cloned() {
                if let Some(doc) = multi_doc.get_document_mut(&source_file) {
                    let _ = doc.remove_handler_call(handler, index);
                    multi_doc.mark_dirty(&source_file);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !removed_in_multi {
            if let Some(doc) = &mut self.document {
                let _ = doc.remove_handler_call(handler, index);
            }
        }

        Ok(value)
    }

    // ========== SPECIAL CATEGORY MUTATIONS ==========

    /// Get a mutable reference to a special category instance.
    ///
    /// Returns a [`MutableCategoryInstance`](crate::MutableCategoryInstance) that allows you to read,
    /// modify, and remove values within the category instance.
    ///
    /// Returns an error if the category or instance doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
    ///
    /// // Get mutable reference and modify
    /// let mut mouse = config.get_special_category_mut("device", "mouse").unwrap();
    /// mouse.set("sensitivity", ConfigValue::Float(2.5)).unwrap();
    ///
    /// // Verify the change
    /// let mouse_data = config.get_special_category("device", "mouse").unwrap();
    /// assert_eq!(mouse_data.get("sensitivity").unwrap().as_float().unwrap(), 2.5);
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn get_special_category_mut(
        &mut self,
        category: &str,
        key: &str,
    ) -> ParseResult<crate::mutation::MutableCategoryInstance<'_>> {
        // Verify it exists
        if !self.special_categories.instance_exists(category, key) {
            return Err(ConfigError::category_not_found(
                category,
                Some(key.to_string()),
            ));
        }

        Ok(crate::mutation::MutableCategoryInstance::new(
            category.to_string(),
            key.to_string(),
            &mut self.special_categories,
        ))
    }

    /// Remove a special category instance.
    ///
    /// Removes the entire category instance and all values within it.
    /// Returns an error if the category or instance doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.register_special_category_value("device", "sensitivity", ConfigValue::Float(0.0));
    /// config.register_special_category_value("device", "repeat_rate", ConfigValue::Int(0));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n}\ndevice[keyboard] {\n  repeat_rate = 50\n}").unwrap();
    ///
    /// config.remove_special_category_instance("device", "mouse").unwrap();
    ///
    /// assert!(config.get_special_category("device", "mouse").is_err());
    /// assert!(config.get_special_category("device", "keyboard").is_ok());
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn remove_special_category_instance(
        &mut self,
        category: &str,
        key: &str,
    ) -> ParseResult<()> {
        self.special_categories.remove_instance(category, key)?;

        // Remove from document tree for serialization consistency
        if let Some(doc) = &mut self.document {
            // Ignore error if document doesn't have this category (e.g., manually added)
            let _ = doc.remove_special_category_instance(category, key);
        }

        Ok(())
    }

    // ========== SERIALIZATION METHODS (mutation feature) ==========

    /// Serialize the configuration to a string.
    ///
    /// Generates a clean, well-formatted configuration string containing all values, variables,
    /// and handler calls. The current implementation uses synthetic serialization, which means:
    /// - All config data is preserved
    /// - Output is clean and consistently formatted
    /// - Original comments and formatting are not preserved
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse("$GAPS = 10\nborder_size = 3").unwrap();
    /// config.set_int("opacity", 1);
    ///
    /// let output = config.serialize();
    /// assert!(output.contains("$GAPS = 10"));
    /// assert!(output.contains("border_size = 3"));
    /// assert!(output.contains("opacity = 1"));
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn serialize(&self) -> String {
        if let Some(doc) = &self.document {
            doc.serialize()
        } else {
            // Fallback: generate from scratch (no formatting preserved)
            self.serialize_synthetic()
        }
    }

    /// Save the configuration to its source file.
    ///
    /// This method is only available if the configuration was loaded from a file using
    /// [`parse_file`](Config::parse_file). Returns an error if no source file is associated
    /// with this configuration.
    ///
    /// Use [`save_as`](Config::save_as) to save to a different file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    /// use std::path::Path;
    ///
    /// let mut config = Config::new();
    /// config.parse_file(Path::new("config.conf")).unwrap();
    ///
    /// config.set_int("border_size", 5);
    ///
    /// // Save back to config.conf
    /// config.save().unwrap();
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn save(&self) -> ParseResult<()> {
        let path = self.source_file.as_ref().ok_or_else(|| {
            ConfigError::custom(
                "No source file associated with this config. Use save_as() instead.",
            )
        })?;

        let content = self.serialize();
        std::fs::write(path, content)
            .map_err(|e| ConfigError::io(path.display().to_string(), e.to_string()))
    }

    /// Save the configuration to a specific file.
    ///
    /// This method works whether or not the configuration was loaded from a file.
    /// The serialized output includes all values, variables, and handler calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse("$GAPS = 10\nborder_size = 3").unwrap();
    /// config.set_int("opacity", 1);
    ///
    /// // Save to a new file
    /// config.save_as("modified_config.conf").unwrap();
    ///
    /// // Create a backup
    /// config.save_as("config.backup").unwrap();
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn save_as(&self, path: impl AsRef<Path>) -> ParseResult<()> {
        let content = self.serialize();
        std::fs::write(&path, content)
            .map_err(|e| ConfigError::io(path.as_ref().display().to_string(), e.to_string()))
    }

    /// Save all modified files.
    ///
    /// When configuration is loaded from multiple files via `source = path` directives,
    /// this method saves only the files that have been modified since parsing.
    ///
    /// Returns a list of file paths that were written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// // Assume main.conf includes vars.conf and appearance.conf via source directives
    /// config.parse_file("main.conf").unwrap();
    ///
    /// // Modify a value from appearance.conf
    /// config.set_int("decoration:rounding", 15);
    ///
    /// // Save only the modified files (appearance.conf in this case)
    /// let saved_files = config.save_all().unwrap();
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn save_all(&mut self) -> ParseResult<Vec<PathBuf>> {
        let mut saved = Vec::new();

        if let Some(multi_doc) = &self.multi_document {
            let dirty_files: Vec<PathBuf> = multi_doc.get_dirty_files().iter().map(|p| (*p).clone()).collect();

            for path in dirty_files {
                if let Some(doc) = multi_doc.get_document(&path) {
                    let content = doc.serialize();
                    std::fs::write(&path, content)
                        .map_err(|e| ConfigError::io(path.display().to_string(), e.to_string()))?;
                    saved.push(path);
                }
            }
        }

        // Clear dirty flags after successful save
        if let Some(multi_doc) = &mut self.multi_document {
            multi_doc.clear_dirty();
        }

        Ok(saved)
    }

    /// Serialize a specific source file.
    ///
    /// Returns the serialized content of the specified source file, or an error
    /// if the file is not part of this configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    /// use std::path::Path;
    ///
    /// let mut config = Config::new();
    /// config.parse_file("main.conf").unwrap();
    ///
    /// // Get the serialized content of a specific source file
    /// let content = config.serialize_file(Path::new("/path/to/vars.conf")).unwrap();
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn serialize_file(&self, path: &Path) -> ParseResult<String> {
        if let Some(multi_doc) = &self.multi_document
            && let Some(doc) = multi_doc.get_document(path)
        {
            return Ok(doc.serialize());
        }

        Err(ConfigError::custom(format!(
            "File not found in configuration: {}",
            path.display()
        )))
    }

    /// Get which source file a key is defined in.
    ///
    /// Returns the path to the source file that contains the given key,
    /// or `None` if the key doesn't exist or the configuration wasn't loaded from files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse_file("main.conf").unwrap();
    ///
    /// if let Some(path) = config.get_key_source_file("decoration:rounding") {
    ///     println!("Key is defined in: {}", path.display());
    /// }
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn get_key_source_file(&self, key: &str) -> Option<&Path> {
        self.multi_document
            .as_ref()
            .and_then(|multi_doc| multi_doc.get_key_source(key))
            .map(|p| p.as_path())
    }

    /// Get all source files that are part of this configuration.
    ///
    /// Returns a list of all file paths that were parsed as part of this configuration,
    /// including the primary file and any files included via `source` directives.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse_file("main.conf").unwrap();
    ///
    /// for path in config.get_source_files() {
    ///     println!("Source file: {}", path.display());
    /// }
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn get_source_files(&self) -> Vec<&Path> {
        self.multi_document
            .as_ref()
            .map(|multi_doc| multi_doc.get_all_paths().iter().map(|p| p.as_path()).collect())
            .unwrap_or_default()
    }

    /// Get all files that have been modified since parsing.
    ///
    /// Returns a list of file paths that have pending changes to be saved.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::Config;
    ///
    /// let mut config = Config::new();
    /// config.parse_file("main.conf").unwrap();
    ///
    /// config.set_int("decoration:rounding", 15);
    ///
    /// for path in config.get_modified_files() {
    ///     println!("Modified file: {}", path.display());
    /// }
    /// # }
    /// ```
    #[cfg(feature = "mutation")]
    pub fn get_modified_files(&self) -> Vec<&Path> {
        self.multi_document
            .as_ref()
            .map(|multi_doc| multi_doc.get_dirty_files().iter().map(|p| p.as_path()).collect())
            .unwrap_or_default()
    }

    /// Generate a synthetic config (when no document exists)
    #[cfg(feature = "mutation")]
    fn serialize_synthetic(&self) -> String {
        let mut output = String::new();

        // Variables
        let vars = self.variables.all();
        if !vars.is_empty() {
            for (name, value) in vars {
                output.push_str(&format!("${} = {}\n", name, value));
            }
            output.push('\n');
        }

        // Regular values (need to reconstruct categories)
        let mut keys: Vec<_> = self.values.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(entry) = self.values.get(key.as_str()) {
                if key.contains(':') {
                    // Nested key - format with categories
                    let parts: Vec<&str> = key.split(':').collect();
                    output.push_str(&format!("{} = {}\n", parts.join(":"), entry.raw));
                } else {
                    // Root-level key
                    output.push_str(&format!("{} = {}\n", key, entry.raw));
                }
            }
        }

        if !self.values.is_empty() {
            output.push('\n');
        }

        // Handler calls
        let mut handler_names: Vec<_> = self.handler_calls.keys().collect();
        handler_names.sort();

        for handler in handler_names {
            if let Some(calls) = self.handler_calls.get(handler.as_str()) {
                for call in calls {
                    output.push_str(&format!("{} = {}\n", handler, call));
                }
            }
        }

        output
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
