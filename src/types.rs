use crate::error::{ConfigError, ParseResult};
use std::any::Any;
use std::fmt;
use std::rc::Rc;

/// A 2D vector with x and y components
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color from RGBA components
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB components (alpha = 255)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a color from a hex string (RRGGBBAA)
    pub fn from_hex(hex: &str) -> ParseResult<Self> {
        let hex = hex.trim_start_matches("0x");

        if hex.len() != 6 && hex.len() != 8 {
            return Err(ConfigError::invalid_color(
                hex,
                "hex color must be 6 or 8 characters",
            ));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| ConfigError::invalid_color(hex, "invalid hex digits"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| ConfigError::invalid_color(hex, "invalid hex digits"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| ConfigError::invalid_color(hex, "invalid hex digits"))?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| ConfigError::invalid_color(hex, "invalid hex digits"))?
        } else {
            255
        };

        Ok(Self { r, g, b, a })
    }

    /// Create a color from RGBA float components (0.0-1.0)
    pub fn from_rgba_float(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: (r * 255.0).round() as u8,
            g: (g * 255.0).round() as u8,
            b: (b * 255.0).round() as u8,
            a: (a * 255.0).round() as u8,
        }
    }

    /// Convert to hex ARGB format (as u32)
    pub fn to_argb(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert to hex RGBA format (as u32)
    pub fn to_rgba(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

/// Trait for custom value types
pub trait CustomValueType: Any + fmt::Debug {
    /// Parse a value from a string
    fn parse(&self, value: &str) -> ParseResult<Box<dyn Any>>;

    /// Get a human-readable type name
    fn type_name(&self) -> &str;

    /// Clone the custom value
    fn clone_value(&self, value: &dyn Any) -> Box<dyn Any>;
}

/// Configuration value types
#[derive(Clone)]
pub enum ConfigValue {
    /// 64-bit signed integer
    Int(i64),

    /// 64-bit floating point
    Float(f64),

    /// String value
    String(String),

    /// 2D vector
    Vec2(Vec2),

    /// RGBA color
    Color(Color),

    /// Custom type with handler
    Custom {
        type_name: String,
        value: Rc<dyn Any>,
    },
}

impl ConfigValue {
    /// Try to get the value as an integer
    pub fn as_int(&self) -> ParseResult<i64> {
        match self {
            ConfigValue::Int(v) => Ok(*v),
            _ => Err(ConfigError::type_error("value", "Int", self.type_name())),
        }
    }

    /// Try to get the value as a float
    pub fn as_float(&self) -> ParseResult<f64> {
        match self {
            ConfigValue::Float(v) => Ok(*v),
            ConfigValue::Int(v) => Ok(*v as f64),
            _ => Err(ConfigError::type_error("value", "Float", self.type_name())),
        }
    }

    /// Try to get the value as a string
    pub fn as_string(&self) -> ParseResult<&str> {
        match self {
            ConfigValue::String(v) => Ok(v),
            _ => Err(ConfigError::type_error("value", "String", self.type_name())),
        }
    }

    /// Try to get the value as a Vec2
    pub fn as_vec2(&self) -> ParseResult<Vec2> {
        match self {
            ConfigValue::Vec2(v) => Ok(*v),
            _ => Err(ConfigError::type_error("value", "Vec2", self.type_name())),
        }
    }

    /// Try to get the value as a Color
    pub fn as_color(&self) -> ParseResult<Color> {
        match self {
            ConfigValue::Color(v) => Ok(*v),
            _ => Err(ConfigError::type_error("value", "Color", self.type_name())),
        }
    }

    /// Try to get the value as a custom type
    pub fn as_custom<T: 'static>(&self) -> ParseResult<&T> {
        match self {
            ConfigValue::Custom { value, .. } => {
                value.downcast_ref::<T>()
                    .ok_or_else(|| ConfigError::type_error("value", "Custom", self.type_name()))
            }
            _ => Err(ConfigError::type_error("value", "Custom", self.type_name())),
        }
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &str {
        match self {
            ConfigValue::Int(_) => "Int",
            ConfigValue::Float(_) => "Float",
            ConfigValue::String(_) => "String",
            ConfigValue::Vec2(_) => "Vec2",
            ConfigValue::Color(_) => "Color",
            ConfigValue::Custom { type_name, .. } => type_name,
        }
    }

    /// Parse a boolean value (true/false/on/off/yes/no)
    pub fn parse_bool(s: &str) -> ParseResult<bool> {
        match s.to_lowercase().as_str() {
            "true" | "on" | "yes" | "1" => Ok(true),
            "false" | "off" | "no" | "0" => Ok(false),
            _ => Err(ConfigError::invalid_number(s, "not a valid boolean")),
        }
    }

    /// Parse an integer (decimal or hex)
    pub fn parse_int(s: &str) -> ParseResult<i64> {
        if let Some(hex) = s.strip_prefix("0x") {
            i64::from_str_radix(hex, 16)
                .map_err(|_| ConfigError::invalid_number(s, "invalid hex integer"))
        } else {
            s.parse::<i64>()
                .map_err(|_| ConfigError::invalid_number(s, "invalid integer"))
        }
    }

    /// Parse a float
    pub fn parse_float(s: &str) -> ParseResult<f64> {
        s.parse::<f64>()
            .map_err(|_| ConfigError::invalid_number(s, "invalid float"))
    }
}

impl fmt::Debug for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Int(v) => write!(f, "Int({})", v),
            ConfigValue::Float(v) => write!(f, "Float({})", v),
            ConfigValue::String(v) => write!(f, "String({:?})", v),
            ConfigValue::Vec2(v) => write!(f, "Vec2({:?})", v),
            ConfigValue::Color(v) => write!(f, "Color({:?})", v),
            ConfigValue::Custom { type_name, .. } => write!(f, "Custom({})", type_name),
        }
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Int(v) => write!(f, "{}", v),
            ConfigValue::Float(v) => write!(f, "{}", v),
            ConfigValue::String(v) => write!(f, "{}", v),
            ConfigValue::Vec2(v) => write!(f, "{}", v),
            ConfigValue::Color(v) => write!(f, "{}", v),
            ConfigValue::Custom { type_name, .. } => write!(f, "<{}>", type_name),
        }
    }
}

/// Wrapper for config values with metadata
#[derive(Clone)]
pub struct ConfigValueEntry {
    /// The actual value
    pub value: ConfigValue,

    /// Whether this value was set by the user (vs default)
    pub set_by_user: bool,

    /// The raw string representation (before parsing)
    pub raw: String,
}

impl ConfigValueEntry {
    pub fn new(value: ConfigValue, raw: String) -> Self {
        Self {
            value,
            set_by_user: true,
            raw,
        }
    }

    pub fn with_default(value: ConfigValue) -> Self {
        Self {
            value: value.clone(),
            set_by_user: false,
            raw: value.to_string(),
        }
    }
}

impl fmt::Debug for ConfigValueEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigValueEntry")
            .field("value", &self.value)
            .field("set_by_user", &self.set_by_user)
            .field("raw", &self.raw)
            .finish()
    }
}
