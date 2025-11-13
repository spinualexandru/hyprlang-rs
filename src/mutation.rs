//! Configuration mutation API.
//!
//! This module provides types for mutating configuration values after parsing.
//! It requires the `mutation` cargo feature to be enabled.
//!
//! The main types are:
//! - [`MutableVariable`] - A mutable reference to a variable
//! - [`MutableCategoryInstance`] - A mutable reference to a special category instance
//!
//! These types are typically obtained through methods on [`Config`](crate::Config):
//! - [`Config::get_variable_mut`](crate::Config::get_variable_mut)
//! - [`Config::get_special_category_mut`](crate::Config::get_special_category_mut)
//!
//! # Examples
//!
//! ## Mutating Variables
//!
//! ```
//! # #[cfg(feature = "mutation")] {
//! use hyprlang::Config;
//!
//! let mut config = Config::new();
//! config.parse("$GAPS = 10").unwrap();
//!
//! if let Some(mut gaps) = config.get_variable_mut("GAPS") {
//!     gaps.set("20").unwrap();
//! }
//!
//! assert_eq!(config.get_variable("GAPS"), Some("20"));
//! # }
//! ```
//!
//! ## Mutating Special Category Instances
//!
//! ```
//! # #[cfg(feature = "mutation")] {
//! use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
//!
//! let mut config = Config::new();
//! config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
//! config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
//!
//! let mut mouse = config.get_special_category_mut("device", "mouse").unwrap();
//! mouse.set("sensitivity", ConfigValue::Float(2.5)).unwrap();
//! # }
//! ```

use crate::document::ConfigDocument;
use crate::error::{ConfigError, ParseResult};
use crate::special_categories::SpecialCategoryManager;
use crate::types::{ConfigValue, ConfigValueEntry};
use crate::variables::VariableManager;

/// A mutable reference to a variable.
///
/// This type is returned by [`Config::get_variable_mut`](crate::Config::get_variable_mut) and
/// provides an alternative API for modifying variables. Instead of calling `set_variable` directly,
/// you can get a mutable reference and call methods on it.
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
/// if let Some(mut gaps) = config.get_variable_mut("GAPS") {
///     println!("Variable name: {}", gaps.name());
///     println!("Current value: {}", gaps.get());
///
///     gaps.set("20").unwrap();
///     println!("New value: {}", gaps.get());
/// }
/// # }
/// ```
pub struct MutableVariable<'a> {
    name: String,
    manager: &'a mut VariableManager,
    #[allow(dead_code)]
    document: Option<&'a mut ConfigDocument>,
}

impl<'a> MutableVariable<'a> {
    /// Create a new mutable variable reference (internal use only)
    pub(crate) fn new(
        name: String,
        manager: &'a mut VariableManager,
        document: Option<&'a mut ConfigDocument>,
    ) -> Self {
        Self {
            name,
            manager,
            document,
        }
    }

    /// Get the current value of the variable.
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
    /// if let Some(gaps) = config.get_variable_mut("GAPS") {
    ///     assert_eq!(gaps.get(), "10");
    /// }
    /// # }
    /// ```
    pub fn get(&self) -> &str {
        self.manager.get(&self.name).unwrap_or("")
    }

    /// Set a new value for the variable.
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
    /// if let Some(mut gaps) = config.get_variable_mut("GAPS") {
    ///     gaps.set("20").unwrap();
    ///     assert_eq!(gaps.get(), "20");
    /// }
    ///
    /// assert_eq!(config.get_variable("GAPS"), Some("20"));
    /// # }
    /// ```
    pub fn set(&mut self, value: impl Into<String>) -> ParseResult<()> {
        let value = value.into();
        self.manager.set(self.name.clone(), value.clone());

        if let Some(doc) = &mut self.document {
            doc.update_or_insert_variable(&self.name, &value)?;
        }

        Ok(())
    }

    /// Get the name of this variable.
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
    /// if let Some(gaps) = config.get_variable_mut("GAPS") {
    ///     assert_eq!(gaps.name(), "GAPS");
    /// }
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A mutable reference to a special category instance.
///
/// This type is returned by [`Config::get_special_category_mut`](crate::Config::get_special_category_mut)
/// and allows you to read, modify, and remove values within a special category instance.
///
/// Special categories are categories that can have multiple instances, such as `device[mouse]`,
/// `device[keyboard]`, etc.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "mutation")] {
/// use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
///
/// let mut config = Config::new();
/// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
/// config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
///
/// let mut mouse = config.get_special_category_mut("device", "mouse").unwrap();
/// mouse.set("sensitivity", ConfigValue::Float(2.5)).unwrap();
/// mouse.set("accel_profile", ConfigValue::String("flat".to_string())).unwrap();
///
/// let sensitivity = mouse.get("sensitivity").unwrap();
/// assert_eq!(sensitivity.as_float().unwrap(), 2.5);
/// # }
/// ```
pub struct MutableCategoryInstance<'a> {
    category: String,
    key: String,
    manager: &'a mut SpecialCategoryManager,
    #[allow(dead_code)]
    document: Option<&'a mut ConfigDocument>,
}

impl<'a> MutableCategoryInstance<'a> {
    /// Create a new mutable category instance reference (internal use only)
    pub(crate) fn new(
        category: String,
        key: String,
        manager: &'a mut SpecialCategoryManager,
        document: Option<&'a mut ConfigDocument>,
    ) -> Self {
        Self {
            category,
            key,
            manager,
            document,
        }
    }

    /// Get a value from this category instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
    ///
    /// let mouse = config.get_special_category_mut("device", "mouse").unwrap();
    /// let sensitivity = mouse.get("sensitivity").unwrap();
    /// assert_eq!(sensitivity.as_float().unwrap(), 1.0);
    /// # }
    /// ```
    pub fn get(&self, key: &str) -> ParseResult<&ConfigValue> {
        let instance = self.manager.get_instance(&self.category, &self.key)?;
        instance.get(key)
            .map(|entry| &entry.value)
            .ok_or_else(|| ConfigError::key_not_found(key))
    }

    /// Set a value in this category instance.
    ///
    /// If the key already exists, it will be updated. Otherwise, a new key is added.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
    ///
    /// let mut mouse = config.get_special_category_mut("device", "mouse").unwrap();
    /// mouse.set("sensitivity", ConfigValue::Float(2.5)).unwrap();
    /// mouse.set("accel_profile", ConfigValue::String("flat".to_string())).unwrap();
    ///
    /// assert_eq!(mouse.get("sensitivity").unwrap().as_float().unwrap(), 2.5);
    /// # }
    /// ```
    pub fn set(&mut self, key: impl Into<String>, value: ConfigValue) -> ParseResult<()> {
        let key = key.into();
        let raw = value.to_string();
        let entry = ConfigValueEntry::new(value, raw.clone());

        let instance = self.manager.get_instance_mut(&self.category, &self.key)?;
        instance.set(key.clone(), entry);

        // TODO: Update document if available
        // if let Some(doc) = &mut self.document {
        //     doc.update_or_insert_category_value(&self.category, &self.key, &key, &raw)?;
        // }

        Ok(())
    }

    /// Remove a value from this category instance.
    ///
    /// Returns the removed value, or an error if the key doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, ConfigValue, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n  accel_profile = flat\n}").unwrap();
    ///
    /// let mut mouse = config.get_special_category_mut("device", "mouse").unwrap();
    /// let removed = mouse.remove("accel_profile").unwrap();
    ///
    /// assert_eq!(removed.as_string().unwrap(), "flat");
    /// assert!(mouse.get("accel_profile").is_err());
    /// # }
    /// ```
    pub fn remove(&mut self, key: &str) -> ParseResult<ConfigValue> {
        let instance = self.manager.get_instance_mut(&self.category, &self.key)?;
        let entry = instance.values.remove(key)
            .ok_or_else(|| ConfigError::key_not_found(key))?;

        // TODO: Remove from document
        // if let Some(doc) = &mut self.document {
        //     doc.remove_category_value(&self.category, &self.key, key)?;
        // }

        Ok(entry.value)
    }

    /// Get the name of the special category.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
    ///
    /// let mouse = config.get_special_category_mut("device", "mouse").unwrap();
    /// assert_eq!(mouse.category(), "device");
    /// # }
    /// ```
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Get the key of this category instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "mutation")] {
    /// use hyprlang::{Config, SpecialCategoryDescriptor};
    ///
    /// let mut config = Config::new();
    /// config.register_special_category(SpecialCategoryDescriptor::keyed("device", "name"));
    /// config.parse("device[mouse] {\n  sensitivity = 1.0\n}").unwrap();
    ///
    /// let mouse = config.get_special_category_mut("device", "mouse").unwrap();
    /// assert_eq!(mouse.key(), "mouse");
    /// # }
    /// ```
    pub fn key(&self) -> &str {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutable_variable() {
        let mut manager = VariableManager::new();
        manager.set("TEST".to_string(), "value1".to_string());

        {
            let mut var = MutableVariable::new("TEST".to_string(), &mut manager, None);
            assert_eq!(var.get(), "value1");
            assert_eq!(var.name(), "TEST");

            var.set("value2").unwrap();
            assert_eq!(var.get(), "value2");
        }

        // Verify the change persisted
        assert_eq!(manager.get("TEST").unwrap(), "value2");
    }
}
