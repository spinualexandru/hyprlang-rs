use crate::error::{ConfigError, ParseResult};
use crate::types::{ConfigValue, ConfigValueEntry};
use std::collections::HashMap;

/// Type of special category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCategoryType {
    /// Key-based: category[key] { ... }
    Keyed,
    /// Static: category { ... } (no key required)
    Static,
    /// Anonymous: auto-assigned keys
    Anonymous,
}

/// Descriptor for a special category configuration
#[derive(Debug, Clone)]
pub struct SpecialCategoryDescriptor {
    /// Name of the category
    pub name: String,

    /// Type of category
    pub category_type: SpecialCategoryType,

    /// Name of the key field (for keyed categories)
    pub key_field: Option<String>,

    /// Default values for properties in this category
    pub default_values: HashMap<String, ConfigValue>,

    /// If true, accessing a non-existent instance returns None instead of an error
    pub ignore_missing: bool,
}

impl SpecialCategoryDescriptor {
    /// Create a new keyed special category
    pub fn keyed(name: impl Into<String>, key_field: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category_type: SpecialCategoryType::Keyed,
            key_field: Some(key_field.into()),
            default_values: HashMap::new(),
            ignore_missing: false,
        }
    }

    /// Create a new static special category
    pub fn static_category(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category_type: SpecialCategoryType::Static,
            key_field: None,
            default_values: HashMap::new(),
            ignore_missing: false,
        }
    }

    /// Create a new anonymous special category
    pub fn anonymous(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category_type: SpecialCategoryType::Anonymous,
            key_field: None,
            default_values: HashMap::new(),
            ignore_missing: false,
        }
    }

    /// Add a default value for a property
    pub fn with_default(mut self, property: impl Into<String>, value: ConfigValue) -> Self {
        self.default_values.insert(property.into(), value);
        self
    }

    /// Set ignore_missing to true - accessing non-existent instances returns None instead of error
    pub fn with_ignore_missing(mut self) -> Self {
        self.ignore_missing = true;
        self
    }
}

/// A single instance of a special category
#[derive(Debug, Clone)]
pub struct SpecialCategoryInstance {
    /// The key for this instance (if keyed or anonymous)
    pub key: Option<String>,

    /// Values within this category instance
    pub values: HashMap<String, ConfigValueEntry>,

    /// Whether this instance was set by the user
    pub set_by_user: bool,
}

impl SpecialCategoryInstance {
    pub fn new(key: Option<String>) -> Self {
        Self {
            key,
            values: HashMap::new(),
            set_by_user: true,
        }
    }

    /// Get a value from this instance
    pub fn get(&self, key: &str) -> Option<&ConfigValueEntry> {
        self.values.get(key)
    }

    /// Set a value in this instance
    pub fn set(&mut self, key: String, value: ConfigValueEntry) {
        self.values.insert(key, value);
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

/// Manager for special categories
pub struct SpecialCategoryManager {
    /// Descriptors for all registered special categories
    descriptors: HashMap<String, SpecialCategoryDescriptor>,

    /// Instances of special categories: category_name -> key -> instance
    instances: HashMap<String, HashMap<String, SpecialCategoryInstance>>,

    /// Next auto-assigned anonymous key (global across categories, matching upstream)
    next_anonymous_key: usize,
}

impl SpecialCategoryManager {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            instances: HashMap::new(),
            next_anonymous_key: 1,
        }
    }

    /// Register a special category descriptor
    pub fn register(&mut self, descriptor: SpecialCategoryDescriptor) {
        let category_name = descriptor.name.clone();
        let category_type = descriptor.category_type;
        let default_values = descriptor.default_values.clone();

        self.descriptors.insert(category_name.clone(), descriptor);

        if category_type == SpecialCategoryType::Static {
            let instances = self.instances.entry(category_name).or_default();
            let instance = instances
                .entry("static".to_string())
                .or_insert_with(|| SpecialCategoryInstance::new(Some("static".to_string())));

            for (prop_name, default_value) in default_values {
                instance.set(prop_name, ConfigValueEntry::with_default(default_value));
            }
        }
    }

    /// Check if a category is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.descriptors.contains_key(name)
    }

    /// Get the descriptor for a category
    pub fn get_descriptor(&self, name: &str) -> Option<&SpecialCategoryDescriptor> {
        self.descriptors.get(name)
    }

    /// Create a new instance of a special category
    pub fn create_instance(
        &mut self,
        category_name: &str,
        key: Option<String>,
    ) -> ParseResult<String> {
        let descriptor = self
            .descriptors
            .get(category_name)
            .ok_or_else(|| ConfigError::category_not_found(category_name, None))?
            .clone(); // Clone to avoid borrow checker issues

        let instance_key = match descriptor.category_type {
            SpecialCategoryType::Keyed => key.ok_or_else(|| {
                ConfigError::custom(format!("Keyed category '{}' requires a key", category_name))
            })?,
            SpecialCategoryType::Static => {
                if key.is_some() {
                    return Err(ConfigError::custom(format!(
                        "Static category '{}' cannot have a key",
                        category_name
                    )));
                }
                "static".to_string()
            }
            SpecialCategoryType::Anonymous => {
                if let Some(explicit_key) = key {
                    explicit_key
                } else {
                    let key = self.next_anonymous_key.to_string();
                    self.next_anonymous_key += 1;
                    key
                }
            }
        };

        if self.instance_exists(category_name, &instance_key) {
            return Ok(instance_key);
        }

        // Create the instance with default values
        let mut instance = SpecialCategoryInstance::new(Some(instance_key.clone()));

        // Apply default values from descriptor
        for (prop_name, default_value) in &descriptor.default_values {
            instance.set(
                prop_name.clone(),
                ConfigValueEntry::with_default(default_value.clone()),
            );
        }

        self.instances
            .entry(category_name.to_string())
            .or_default()
            .insert(instance_key.clone(), instance);

        Ok(instance_key)
    }

    /// Get a special category instance
    pub fn get_instance(
        &self,
        category_name: &str,
        key: &str,
    ) -> ParseResult<&SpecialCategoryInstance> {
        self.instances
            .get(category_name)
            .and_then(|instances| instances.get(key))
            .ok_or_else(|| ConfigError::category_not_found(category_name, Some(key.to_string())))
    }

    /// Get a mutable special category instance
    pub fn get_instance_mut(
        &mut self,
        category_name: &str,
        key: &str,
    ) -> ParseResult<&mut SpecialCategoryInstance> {
        self.instances
            .get_mut(category_name)
            .and_then(|instances| instances.get_mut(key))
            .ok_or_else(|| ConfigError::category_not_found(category_name, Some(key.to_string())))
    }

    /// Try to get a special category instance, returning None if not found
    ///
    /// This is useful when the category has `ignore_missing` set to true,
    /// or when you want to check if an instance exists without erroring.
    pub fn try_get_instance(
        &self,
        category_name: &str,
        key: &str,
    ) -> Option<&SpecialCategoryInstance> {
        self.instances
            .get(category_name)
            .and_then(|instances| instances.get(key))
    }

    /// Try to get a mutable special category instance, returning None if not found
    pub fn try_get_instance_mut(
        &mut self,
        category_name: &str,
        key: &str,
    ) -> Option<&mut SpecialCategoryInstance> {
        self.instances
            .get_mut(category_name)
            .and_then(|instances| instances.get_mut(key))
    }

    /// Get a special category instance, respecting the `ignore_missing` flag
    ///
    /// If the category has `ignore_missing` set to true and the instance doesn't exist,
    /// returns `Ok(None)` instead of an error.
    pub fn get_instance_optional(
        &self,
        category_name: &str,
        key: &str,
    ) -> ParseResult<Option<&SpecialCategoryInstance>> {
        let ignore_missing = self
            .descriptors
            .get(category_name)
            .map(|d| d.ignore_missing)
            .unwrap_or(false);

        match self.try_get_instance(category_name, key) {
            Some(instance) => Ok(Some(instance)),
            None if ignore_missing => Ok(None),
            None => Err(ConfigError::category_not_found(
                category_name,
                Some(key.to_string()),
            )),
        }
    }

    /// Get all keys for a special category
    pub fn list_keys(&self, category_name: &str) -> Vec<String> {
        if self
            .descriptors
            .get(category_name)
            .map(|descriptor| descriptor.category_type == SpecialCategoryType::Static)
            .unwrap_or(false)
        {
            return Vec::new();
        }

        self.instances
            .get(category_name)
            .map(|instances| instances.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all instances for a category
    pub fn get_all_instances(&self, category_name: &str) -> Vec<&SpecialCategoryInstance> {
        self.instances
            .get(category_name)
            .map(|instances| instances.values().collect())
            .unwrap_or_default()
    }

    /// Remove a special category instance
    pub fn remove_instance(&mut self, category_name: &str, key: &str) -> ParseResult<()> {
        if let Some(instances) = self.instances.get_mut(category_name) {
            instances.remove(key).ok_or_else(|| {
                ConfigError::category_not_found(category_name, Some(key.to_string()))
            })?;
            Ok(())
        } else {
            Err(ConfigError::category_not_found(category_name, None))
        }
    }

    /// Check if a category instance exists
    pub fn instance_exists(&self, category_name: &str, key: &str) -> bool {
        self.instances
            .get(category_name)
            .map(|instances| instances.contains_key(key))
            .unwrap_or(false)
    }

    /// Find a registered special category whose name is a prefix of the given key path.
    ///
    /// For colon-scoped categories like `specialGeneric:one`, this matches when the
    /// full key is `specialGeneric:one:value`. Returns (category_name, property_name).
    pub fn find_matching_category(&self, full_key: &str) -> Option<(String, String)> {
        // Check longest names first to match most specific category
        let mut matches: Vec<_> = self
            .descriptors
            .keys()
            .filter(|name| {
                full_key.starts_with(name.as_str()) && full_key[name.len()..].starts_with(':')
            })
            .collect();
        matches.sort_by(|a, b| b.len().cmp(&a.len()));

        matches.first().map(|name| {
            let prop = full_key[name.len() + 1..].to_string();
            (name.to_string(), prop)
        })
    }

    /// Find a registered special category whose segment path is a prefix of the given segments.
    ///
    /// Returns the full category name and the number of path segments it consumed.
    pub fn find_matching_category_segments(
        &self,
        path_segments: &[String],
    ) -> Option<(String, usize)> {
        let mut best_match: Option<(String, usize)> = None;

        for name in self.descriptors.keys() {
            let descriptor_segments: Vec<&str> = name.split(':').collect();
            if descriptor_segments.len() > path_segments.len() {
                continue;
            }

            if descriptor_segments
                .iter()
                .zip(path_segments.iter())
                .all(|(expected, actual)| expected == actual)
            {
                match &best_match {
                    Some((_, best_len)) if *best_len >= descriptor_segments.len() => {}
                    _ => best_match = Some((name.clone(), descriptor_segments.len())),
                }
            }
        }

        best_match
    }

    /// Add a default value to a special category descriptor.
    ///
    /// Static categories are updated immediately to mirror upstream behavior.
    pub fn add_default_value(
        &mut self,
        category_name: &str,
        property: String,
        default_value: ConfigValue,
    ) -> ParseResult<()> {
        let is_static = {
            let descriptor = self
                .descriptors
                .get_mut(category_name)
                .ok_or_else(|| ConfigError::category_not_found(category_name, None))?;
            descriptor
                .default_values
                .insert(property.clone(), default_value.clone());
            descriptor.category_type == SpecialCategoryType::Static
        };

        if is_static {
            let instance_key = self.create_instance(category_name, None)?;
            if let Ok(instance) = self.get_instance_mut(category_name, &instance_key) {
                instance.set(property, ConfigValueEntry::with_default(default_value));
            }
        }

        Ok(())
    }

    /// Remove a default value from a special category descriptor and all existing instances.
    pub fn remove_default_value(&mut self, category_name: &str, property: &str) -> ParseResult<()> {
        let descriptor = self
            .descriptors
            .get_mut(category_name)
            .ok_or_else(|| ConfigError::category_not_found(category_name, None))?;
        descriptor.default_values.remove(property);

        if let Some(instances) = self.instances.get_mut(category_name) {
            for instance in instances.values_mut() {
                instance.values.remove(property);
            }
        }

        Ok(())
    }

    /// Remove a registered special category and all instances.
    pub fn remove_category(&mut self, category_name: &str) {
        self.descriptors.remove(category_name);
        self.instances.remove(category_name);
    }

    /// Reset instances to their pre-parse state.
    ///
    /// Static categories keep a static instance populated from defaults. Keyed and anonymous
    /// categories are cleared completely.
    pub fn reset_for_parse(&mut self) {
        self.next_anonymous_key = 1;

        let descriptors: Vec<_> = self.descriptors.values().cloned().collect();
        self.instances.clear();

        for descriptor in descriptors {
            if descriptor.category_type != SpecialCategoryType::Static {
                continue;
            }

            let mut instance = SpecialCategoryInstance::new(Some("static".to_string()));
            for (prop_name, default_value) in descriptor.default_values {
                instance.set(prop_name, ConfigValueEntry::with_default(default_value));
            }

            self.instances
                .entry(descriptor.name)
                .or_default()
                .insert("static".to_string(), instance);
        }
    }
}

impl Default for SpecialCategoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyed_category() {
        let mut manager = SpecialCategoryManager::new();
        manager.register(SpecialCategoryDescriptor::keyed("device", "name"));

        let key1 = manager
            .create_instance("device", Some("mouse".to_string()))
            .unwrap();
        assert_eq!(key1, "mouse");

        let key2 = manager
            .create_instance("device", Some("keyboard".to_string()))
            .unwrap();
        assert_eq!(key2, "keyboard");

        let keys = manager.list_keys("device");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"mouse".to_string()));
        assert!(keys.contains(&"keyboard".to_string()));
    }

    #[test]
    fn test_static_category() {
        let mut manager = SpecialCategoryManager::new();
        manager.register(SpecialCategoryDescriptor::static_category("global"));

        let key = manager.create_instance("global", None).unwrap();
        assert_eq!(key, "static");

        assert!(manager.instance_exists("global", "static"));
    }

    #[test]
    fn test_anonymous_category() {
        let mut manager = SpecialCategoryManager::new();
        manager.register(SpecialCategoryDescriptor::anonymous("item"));

        let key1 = manager.create_instance("item", None).unwrap();
        let key2 = manager.create_instance("item", None).unwrap();
        let key3 = manager.create_instance("item", None).unwrap();

        assert_eq!(key1, "1");
        assert_eq!(key2, "2");
        assert_eq!(key3, "3");
    }

    #[test]
    fn test_ignore_missing_flag() {
        let descriptor = SpecialCategoryDescriptor::keyed("device", "name").with_ignore_missing();
        assert!(descriptor.ignore_missing);

        let descriptor_default = SpecialCategoryDescriptor::keyed("device2", "name");
        assert!(!descriptor_default.ignore_missing);
    }

    #[test]
    fn test_try_get_instance_returns_none() {
        let mut manager = SpecialCategoryManager::new();
        manager.register(SpecialCategoryDescriptor::keyed("device", "name"));

        // Non-existent instance returns None
        assert!(manager.try_get_instance("device", "nonexistent").is_none());

        // Create instance and verify it returns Some
        manager
            .create_instance("device", Some("mouse".to_string()))
            .unwrap();
        assert!(manager.try_get_instance("device", "mouse").is_some());
    }

    #[test]
    fn test_get_instance_optional_without_ignore_missing() {
        let mut manager = SpecialCategoryManager::new();
        // Register without ignore_missing (default is false)
        manager.register(SpecialCategoryDescriptor::keyed("device", "name"));

        // Non-existent instance returns error
        let result = manager.get_instance_optional("device", "nonexistent");
        assert!(result.is_err());

        // Create instance and verify it returns Ok(Some)
        manager
            .create_instance("device", Some("mouse".to_string()))
            .unwrap();
        let result = manager.get_instance_optional("device", "mouse");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_get_instance_optional_with_ignore_missing() {
        let mut manager = SpecialCategoryManager::new();
        // Register with ignore_missing = true
        manager.register(SpecialCategoryDescriptor::keyed("device", "name").with_ignore_missing());

        // Non-existent instance returns Ok(None) instead of error
        let result = manager.get_instance_optional("device", "nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Create instance and verify it returns Ok(Some)
        manager
            .create_instance("device", Some("mouse".to_string()))
            .unwrap();
        let result = manager.get_instance_optional("device", "mouse");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
