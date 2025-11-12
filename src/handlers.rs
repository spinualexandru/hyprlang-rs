use crate::error::{ConfigError, ParseResult};
use std::collections::HashMap;
use std::rc::Rc;

/// Context for handler execution
pub struct HandlerContext {
    /// The category path where this handler is being called
    pub category: Vec<String>,

    /// The keyword that triggered this handler
    pub keyword: String,

    /// The value passed to the handler
    pub value: String,

    /// Optional flags (e.g., "flagsabc" from "keywordflagsabc = value")
    pub flags: Option<String>,
}

impl HandlerContext {
    pub fn new(keyword: String, value: String) -> Self {
        Self {
            category: Vec::new(),
            keyword,
            value,
            flags: None,
        }
    }

    pub fn with_category(mut self, category: Vec<String>) -> Self {
        self.category = category;
        self
    }

    pub fn with_flags(mut self, flags: String) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Get the full category path as a string
    pub fn category_path(&self) -> String {
        self.category.join(":")
    }
}

/// Trait for implementing custom keyword handlers
pub trait Handler: std::fmt::Debug {
    /// Handle a keyword with the given context
    fn handle(&self, context: &HandlerContext) -> ParseResult<()>;

    /// Get the handler name
    fn name(&self) -> &str;

    /// Check if this handler accepts flags
    fn accepts_flags(&self) -> bool {
        false
    }
}

/// Function-based handler wrapper
#[derive(Clone)]
pub struct FunctionHandler {
    name: String,
    accepts_flags: bool,
    handler: Rc<dyn Fn(&HandlerContext) -> ParseResult<()>>,
}

impl FunctionHandler {
    pub fn new<F>(name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&HandlerContext) -> ParseResult<()> + 'static,
    {
        Self {
            name: name.into(),
            accepts_flags: false,
            handler: Rc::new(handler),
        }
    }

    pub fn with_flags<F>(name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&HandlerContext) -> ParseResult<()> + 'static,
    {
        Self {
            name: name.into(),
            accepts_flags: true,
            handler: Rc::new(handler),
        }
    }
}

impl Handler for FunctionHandler {
    fn handle(&self, context: &HandlerContext) -> ParseResult<()> {
        (self.handler)(context)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn accepts_flags(&self) -> bool {
        self.accepts_flags
    }
}

impl std::fmt::Debug for FunctionHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionHandler")
            .field("name", &self.name)
            .field("accepts_flags", &self.accepts_flags)
            .finish()
    }
}

/// Handler scope type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandlerScope {
    /// Global handler (available everywhere)
    Global,
    /// Category-specific handler
    Category,
}

/// Manager for keyword handlers
pub struct HandlerManager {
    /// Global handlers
    global_handlers: HashMap<String, Box<dyn Handler>>,

    /// Category-scoped handlers: category_path -> keyword -> handler
    category_handlers: HashMap<String, HashMap<String, Box<dyn Handler>>>,
}

impl HandlerManager {
    pub fn new() -> Self {
        Self {
            global_handlers: HashMap::new(),
            category_handlers: HashMap::new(),
        }
    }

    /// Register a global handler
    pub fn register_global<H>(&mut self, keyword: impl Into<String>, handler: H)
    where
        H: Handler + 'static,
    {
        self.global_handlers.insert(keyword.into(), Box::new(handler));
    }

    /// Register a category-scoped handler
    pub fn register_category<H>(
        &mut self,
        category: impl Into<String>,
        keyword: impl Into<String>,
        handler: H,
    )
    where
        H: Handler + 'static,
    {
        self.category_handlers
            .entry(category.into())
            .or_insert_with(HashMap::new)
            .insert(keyword.into(), Box::new(handler));
    }

    /// Find a handler for a keyword in a given category
    pub fn find_handler(
        &self,
        category_path: &[String],
        keyword: &str,
    ) -> Option<&dyn Handler> {
        // First try category-specific handlers (most specific to least specific)
        for i in (0..=category_path.len()).rev() {
            let path = category_path[..i].join(":");
            if let Some(handlers) = self.category_handlers.get(&path) {
                if let Some(handler) = handlers.get(keyword) {
                    return Some(handler.as_ref());
                }
            }
        }

        // Fall back to global handlers
        self.global_handlers.get(keyword)
            .map(|h| h.as_ref())
    }

    /// Check if a handler exists for a keyword
    pub fn has_handler(&self, category_path: &[String], keyword: &str) -> bool {
        self.find_handler(category_path, keyword).is_some()
    }

    /// Execute a handler
    pub fn execute(
        &self,
        category_path: &[String],
        keyword: &str,
        value: &str,
        flags: Option<String>,
    ) -> ParseResult<()> {
        let handler = self.find_handler(category_path, keyword)
            .ok_or_else(|| ConfigError::handler(keyword, "handler not found"))?;

        // Check if flags are provided but not accepted
        if flags.is_some() && !handler.accepts_flags() {
            return Err(ConfigError::handler(keyword, "handler does not accept flags"));
        }

        let context = HandlerContext::new(keyword.to_string(), value.to_string())
            .with_category(category_path.to_vec())
            .with_flags(flags.unwrap_or_default());

        handler.handle(&context)
    }

    /// Clear all handlers
    pub fn clear(&mut self) {
        self.global_handlers.clear();
        self.category_handlers.clear();
    }

    /// Get all registered global handler keywords
    pub fn global_keywords(&self) -> Vec<&str> {
        self.global_handlers.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered category handler keywords for a category
    pub fn category_keywords(&self, category: &str) -> Vec<&str> {
        self.category_handlers.get(category)
            .map(|handlers| handlers.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

impl Default for HandlerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_handler() {
        let mut manager = HandlerManager::new();

        let handler = FunctionHandler::new("test", |ctx| {
            assert_eq!(ctx.keyword, "test");
            assert_eq!(ctx.value, "value");
            Ok(())
        });

        manager.register_global("test", handler);

        assert!(manager.has_handler(&[], "test"));
        manager.execute(&[], "test", "value", None).unwrap();
    }

    #[test]
    fn test_handler_with_flags() {
        let mut manager = HandlerManager::new();

        let handler = FunctionHandler::with_flags("flagged", |ctx| {
            assert_eq!(ctx.flags, Some("abc".to_string()));
            Ok(())
        });

        manager.register_global("flagged", handler);

        manager.execute(&[], "flagged", "value", Some("abc".to_string())).unwrap();
    }

    #[test]
    fn test_category_scoped_handler() {
        let mut manager = HandlerManager::new();

        let handler = FunctionHandler::new("scoped", |ctx| {
            assert_eq!(ctx.category_path(), "category");
            Ok(())
        });

        manager.register_category("category", "scoped", handler);

        assert!(manager.has_handler(&["category".to_string()], "scoped"));
        assert!(!manager.has_handler(&[], "scoped"));

        manager.execute(&["category".to_string()], "scoped", "value", None).unwrap();
    }

    #[test]
    fn test_handler_precedence() {
        let mut manager = HandlerManager::new();

        // Global handler
        let global = FunctionHandler::new("keyword", |_| {
            panic!("Should not call global handler");
        });
        manager.register_global("keyword", global);

        // Category handler (should take precedence)
        let category = FunctionHandler::new("keyword", |_| Ok(()));
        manager.register_category("cat", "keyword", category);

        manager.execute(&["cat".to_string()], "keyword", "value", None).unwrap();
    }
}
