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
}

impl SpecialCategoryDescriptor {
    /// Create a new keyed special category
    pub fn keyed(name: impl Into<String>, key_field: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category_type: SpecialCategoryType::Keyed,
            key_field: Some(key_field.into()),
            default_values: HashMap::new(),
        }
    }

    /// Create a new static special category
    pub fn static_category(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category_type: SpecialCategoryType::Static,
            key_field: None,
            default_values: HashMap::new(),
        }
    }

    /// Create a new anonymous special category
    pub fn anonymous(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category_type: SpecialCategoryType::Anonymous,
            key_field: None,
            default_values: HashMap::new(),
        }
    }

    /// Add a default value for a property
    pub fn with_default(mut self, property: impl Into<String>, value: ConfigValue) -> Self {
        self.default_values.insert(property.into(), value);
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

    /// Counter for anonymous category keys
    anonymous_counters: HashMap<String, usize>,
}

impl SpecialCategoryManager {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            instances: HashMap::new(),
            anonymous_counters: HashMap::new(),
        }
    }

    /// Register a special category descriptor
    pub fn register(&mut self, descriptor: SpecialCategoryDescriptor) {
        self.descriptors.insert(descriptor.name.clone(), descriptor);
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
                if key.is_some() {
                    return Err(ConfigError::custom(format!(
                        "Anonymous category '{}' cannot have an explicit key",
                        category_name
                    )));
                }
                let counter = self
                    .anonymous_counters
                    .entry(category_name.to_string())
                    .or_insert(0);
                let key = format!("anonymous_{}", counter);
                *counter += 1;
                key
            }
        };

        // Create the instance with default values
        let mut instance = SpecialCategoryInstance::new(Some(instance_key.clone()));

        // Apply default values from descriptor
        for (prop_name, default_value) in &descriptor.default_values {
            let raw = default_value.to_string();
            instance.set(
                prop_name.clone(),
                ConfigValueEntry::new(default_value.clone(), raw),
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

    /// Get all keys for a special category
    pub fn list_keys(&self, category_name: &str) -> Vec<String> {
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

    /// Clear all instances (but keep descriptors)
    pub fn clear_instances(&mut self) {
        self.instances.clear();
        self.anonymous_counters.clear();
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

        assert_eq!(key1, "anonymous_0");
        assert_eq!(key2, "anonymous_1");
        assert_eq!(key3, "anonymous_2");
    }
}
