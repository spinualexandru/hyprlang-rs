use pest::Parser;
use pest_derive::Parser;
use crate::error::{ConfigError, ParseResult};
use crate::types::{Color, Vec2};

#[derive(Parser)]
#[grammar = "hyprlang.pest"]
pub struct HyprlangParser;

/// Parse result containing all statements from a config file
#[derive(Debug)]
pub struct ParsedConfig {
    pub statements: Vec<Statement>,
}

/// A statement in the configuration
#[derive(Debug, Clone)]
pub enum Statement {
    /// Variable definition: $VAR = value
    VariableDef {
        name: String,
        value: String,
    },

    /// Assignment: key = value
    Assignment {
        key: Vec<String>,
        value: Value,
    },

    /// Category block: category { statements }
    CategoryBlock {
        name: String,
        statements: Vec<Statement>,
    },

    /// Special category block: category[key] { statements }
    SpecialCategoryBlock {
        name: String,
        key: Option<String>,
        statements: Vec<Statement>,
    },

    /// Handler call: keyword [flags] = value
    HandlerCall {
        keyword: String,
        flags: Option<String>,
        value: String,
    },

    /// Source directive: source = path
    Source {
        path: String,
    },

    /// Comment directive: # hyprlang if/endif/noerror
    CommentDirective {
        directive_type: String,
        args: Option<String>,
    },
}

/// Parsed value types
#[derive(Debug, Clone)]
pub enum Value {
    /// Expression: {{expr}}
    Expression(String),

    /// Variable reference: $VAR
    Variable(String),

    /// Color value
    Color(Color),

    /// Vec2 value
    Vec2(Vec2),

    /// Number (int or float)
    Number(String),

    /// Boolean
    Boolean(bool),

    /// String value
    String(String),

    /// Multiline value
    Multiline(Vec<String>),
}

impl HyprlangParser {
    /// Parse a configuration string
    pub fn parse_config(input: &str) -> ParseResult<ParsedConfig> {
        let pairs = HyprlangParser::parse(Rule::file, input)?;

        let mut statements = Vec::new();

        for pair in pairs {
            if pair.as_rule() == Rule::file {
                for inner in pair.into_inner() {
                    if let Some(stmt) = Self::parse_statement(inner)? {
                        statements.push(stmt);
                    }
                }
            }
        }

        Ok(ParsedConfig { statements })
    }

    fn parse_statement(pair: pest::iterators::Pair<Rule>) -> ParseResult<Option<Statement>> {
        match pair.as_rule() {
            Rule::variable_def => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let value_pair = inner.next().unwrap();
                let value = Self::parse_value_to_string(value_pair)?;
                Ok(Some(Statement::VariableDef { name, value }))
            }

            Rule::assignment => {
                let mut inner = pair.into_inner();
                let key_path = inner.next().unwrap();
                let key = Self::parse_key_path(key_path)?;

                // Value is optional (e.g., "kb_variant =" with empty value)
                let value = if let Some(value_pair) = inner.next() {
                    Self::parse_value(value_pair)?
                } else {
                    Value::String(String::new())
                };

                Ok(Some(Statement::Assignment { key, value }))
            }

            Rule::category_block => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let mut statements = Vec::new();

                for stmt_pair in inner {
                    if let Some(stmt) = Self::parse_statement(stmt_pair)? {
                        statements.push(stmt);
                    }
                }

                Ok(Some(Statement::CategoryBlock { name, statements }))
            }

            Rule::special_category_block => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();

                // Check for optional category_key
                let mut key = None;
                let mut statements = Vec::new();

                for pair in inner {
                    if pair.as_rule() == Rule::category_key {
                        let key_inner = pair.into_inner().next().unwrap();
                        key = Some(key_inner.as_str().to_string());
                    } else if let Some(stmt) = Self::parse_statement(pair)? {
                        statements.push(stmt);
                    }
                }

                Ok(Some(Statement::SpecialCategoryBlock { name, key, statements }))
            }

            Rule::handler_call => {
                let mut inner = pair.into_inner();
                let keyword = inner.next().unwrap().as_str().to_string();

                // Check for flags
                let next = inner.next().unwrap();
                let (flags, value_pair) = if next.as_rule() == Rule::flags {
                    let flags_str = next.as_str().to_string();
                    (Some(flags_str), inner.next().unwrap())
                } else {
                    (None, next)
                };

                let value = Self::parse_value_to_string(value_pair)?;
                Ok(Some(Statement::HandlerCall { keyword, flags, value }))
            }

            Rule::directive => {
                let mut inner = pair.into_inner();
                let value_pair = inner.next().unwrap();
                let path = Self::parse_value_to_string(value_pair)?;
                Ok(Some(Statement::Source { path }))
            }

            Rule::comment_directive => {
                let mut inner = pair.into_inner();
                let directive_type = inner.next().unwrap().as_str().to_string();
                let args = inner.next().map(|p| p.as_str().to_string());
                Ok(Some(Statement::CommentDirective { directive_type, args }))
            }

            Rule::EOI => Ok(None),

            _ => Ok(None),
        }
    }

    fn parse_key_path(pair: pest::iterators::Pair<Rule>) -> ParseResult<Vec<String>> {
        let mut path = Vec::new();
        for inner in pair.into_inner() {
            path.push(inner.as_str().to_string());
        }
        Ok(path)
    }

    fn parse_value(pair: pest::iterators::Pair<Rule>) -> ParseResult<Value> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::single_value => Self::parse_single_value(inner.into_inner().next().unwrap()),
            Rule::multiline_value => {
                let lines: Result<Vec<_>, _> = inner.into_inner()
                    .map(|p| Self::parse_value_to_string(p))
                    .collect();
                Ok(Value::Multiline(lines?))
            }
            _ => Self::parse_single_value(inner),
        }
    }

    fn parse_single_value(pair: pest::iterators::Pair<Rule>) -> ParseResult<Value> {
        match pair.as_rule() {
            Rule::expression => {
                let expr = pair.into_inner().next().unwrap().as_str().to_string();
                Ok(Value::Expression(expr))
            }

            Rule::string_value => {
                let s = pair.as_str();
                // Remove quotes if present
                let s = if s.starts_with('"') && s.ends_with('"') {
                    &s[1..s.len() - 1]
                } else {
                    s
                };
                Ok(Value::String(s.to_string()))
            }

            _ => Ok(Value::String(pair.as_str().to_string())),
        }
    }

    fn parse_value_to_string(pair: pest::iterators::Pair<Rule>) -> ParseResult<String> {
        let value = Self::parse_value(pair)?;
        Ok(match value {
            Value::String(s) => s,
            Value::Number(n) => n,
            Value::Boolean(b) => b.to_string(),
            Value::Expression(e) => format!("{{{{{}}}}}", e),
            Value::Variable(v) => format!("${}", v),
            Value::Color(c) => c.to_string(),
            Value::Vec2(v) => v.to_string(),
            Value::Multiline(lines) => lines.join(" "),
        })
    }

}
