//! Document model for configuration serialization.
//!
//! This module provides data structures for representing parsed configurations with full source fidelity,
//! enabling serialization back to text files. Currently implements synthetic serialization (clean output
//! without original formatting/comments).
//!
//! The main types are:
//! - [`ConfigDocument`] - Represents the entire configuration document
//! - [`DocumentNode`] - Individual nodes in the document tree (assignments, categories, comments, etc.)
//! - [`NodeLocation`] - Index system for fast node lookups during mutations

use crate::error::{ConfigError, ParseResult};
use std::collections::HashMap;

/// Represents a parsed configuration with full source fidelity.
#[derive(Debug, Clone)]
pub struct ConfigDocument {
    /// Root nodes representing the document structure
    pub nodes: Vec<DocumentNode>,

    /// Mapping from logical keys to their node positions
    /// This enables fast lookup for mutations
    key_index: HashMap<String, Vec<NodeLocation>>,

    /// Source file path (if parsed from a file)
    pub source_path: Option<String>,
}

/// A node in the configuration document
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentNode {
    /// Comment (including blank lines)
    Comment {
        /// The comment text (without # prefix)
        text: String,
        /// Line number in source
        line: usize,
    },

    /// Blank line
    BlankLine {
        line: usize,
    },

    /// Variable definition: $VAR = value
    VariableDef {
        name: String,
        value: String,
        /// Original formatting (spacing, etc.)
        raw: String,
        line: usize,
    },

    /// Assignment: key = value
    Assignment {
        key: Vec<String>,
        value: String,
        /// Original formatting
        raw: String,
        line: usize,
    },

    /// Category block: category { ... }
    CategoryBlock {
        name: String,
        nodes: Vec<DocumentNode>,
        /// Opening brace line
        open_line: usize,
        /// Closing brace line
        close_line: usize,
        /// Raw opening line (e.g., "category {")
        raw_open: String,
    },

    /// Special category block: category[key] { ... }
    SpecialCategoryBlock {
        name: String,
        key: Option<String>,
        nodes: Vec<DocumentNode>,
        open_line: usize,
        close_line: usize,
        /// Raw opening line (e.g., "device[mouse] {")
        raw_open: String,
    },

    /// Handler call: keyword [flags] = value
    HandlerCall {
        keyword: String,
        flags: Option<String>,
        value: String,
        raw: String,
        line: usize,
    },

    /// Source directive: source = path
    Source {
        path: String,
        raw: String,
        line: usize,
    },

    /// Comment directive: # hyprlang if/endif/noerror
    CommentDirective {
        directive_type: String,
        args: Option<String>,
        raw: String,
        line: usize,
    },
}

/// Location of a node in the document tree
#[derive(Clone, Debug, PartialEq)]
pub struct NodeLocation {
    /// Path to the node (empty for root-level)
    /// Each index represents the position at that level
    /// Example: [0, 3, 1] means root[0] -> child[3] -> child[1]
    pub path: Vec<usize>,
    /// Node type for verification
    pub node_type: NodeType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
    VariableDef,
    Assignment,
    HandlerCall,
    CategoryBlock,
    SpecialCategoryBlock,
}

impl ConfigDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            key_index: HashMap::new(),
            source_path: None,
        }
    }

    /// Create a document with nodes
    pub fn with_nodes(nodes: Vec<DocumentNode>) -> Self {
        let mut doc = Self {
            nodes,
            key_index: HashMap::new(),
            source_path: None,
        };
        doc.rebuild_index();
        doc
    }

    /// Rebuild the key index from the current nodes
    pub fn rebuild_index(&mut self) {
        self.key_index.clear();
        self.build_index_recursive(&self.nodes.clone(), &Vec::new(), &Vec::new());
    }

    /// Recursively build the index
    fn build_index_recursive(
        &mut self,
        nodes: &[DocumentNode],
        path_prefix: &[usize],
        category_stack: &[String],
    ) {
        for (idx, node) in nodes.iter().enumerate() {
            let mut current_path = path_prefix.to_vec();
            current_path.push(idx);

            match node {
                DocumentNode::VariableDef { name, .. } => {
                    let location = NodeLocation {
                        path: current_path.clone(),
                        node_type: NodeType::VariableDef,
                    };
                    self.key_index
                        .entry(format!("${}", name))
                        .or_insert_with(Vec::new)
                        .push(location);
                }

                DocumentNode::Assignment { key, .. } => {
                    let full_key = if category_stack.is_empty() {
                        key.join(":")
                    } else {
                        format!("{}:{}", category_stack.join(":"), key.join(":"))
                    };

                    let location = NodeLocation {
                        path: current_path.clone(),
                        node_type: NodeType::Assignment,
                    };
                    self.key_index
                        .entry(full_key)
                        .or_insert_with(Vec::new)
                        .push(location);
                }

                DocumentNode::HandlerCall { keyword, .. } => {
                    let handler_key = if category_stack.is_empty() {
                        keyword.clone()
                    } else {
                        format!("{}:{}", category_stack.join(":"), keyword)
                    };

                    let location = NodeLocation {
                        path: current_path.clone(),
                        node_type: NodeType::HandlerCall,
                    };
                    self.key_index
                        .entry(handler_key)
                        .or_insert_with(Vec::new)
                        .push(location);
                }

                DocumentNode::CategoryBlock { name, nodes: child_nodes, .. } => {
                    let mut new_stack = category_stack.to_vec();
                    new_stack.push(name.clone());
                    self.build_index_recursive(child_nodes, &current_path, &new_stack);
                }

                DocumentNode::SpecialCategoryBlock { name, key: category_key, nodes: child_nodes, .. } => {
                    let mut new_stack = category_stack.to_vec();
                    if let Some(k) = category_key {
                        new_stack.push(format!("{}[{}]", name, k));
                    } else {
                        new_stack.push(name.clone());
                    }
                    self.build_index_recursive(child_nodes, &current_path, &new_stack);
                }

                _ => {}
            }
        }
    }

    /// Serialize the document back to string format
    pub fn serialize(&self) -> String {
        let mut output = String::new();
        self.serialize_nodes(&self.nodes, &mut output, 0);
        output
    }

    /// Serialize nodes at a specific indentation level
    fn serialize_nodes(&self, nodes: &[DocumentNode], output: &mut String, indent: usize) {
        for node in nodes {
            match node {
                DocumentNode::Comment { text, .. } => {
                    // Preserve exact spacing in comments
                    if text.is_empty() {
                        output.push_str(&format!("{}#\n", "  ".repeat(indent)));
                    } else {
                        output.push_str(&format!("{}#{}\n", "  ".repeat(indent), text));
                    }
                }

                DocumentNode::BlankLine { .. } => {
                    output.push('\n');
                }

                DocumentNode::VariableDef { raw, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw));
                }

                DocumentNode::Assignment { raw, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw));
                }

                DocumentNode::CategoryBlock { raw_open, nodes: child_nodes, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw_open));
                    self.serialize_nodes(child_nodes, output, indent + 1);
                    output.push_str(&format!("{}}}\n", "  ".repeat(indent)));
                }

                DocumentNode::SpecialCategoryBlock { raw_open, nodes: child_nodes, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw_open));
                    self.serialize_nodes(child_nodes, output, indent + 1);
                    output.push_str(&format!("{}}}\n", "  ".repeat(indent)));
                }

                DocumentNode::HandlerCall { raw, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw));
                }

                DocumentNode::Source { raw, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw));
                }

                DocumentNode::CommentDirective { raw, .. } => {
                    output.push_str(&format!("{}{}\n", "  ".repeat(indent), raw));
                }
            }
        }
    }

    /// Find a node by its location
    pub fn get_node_at(&self, location: &NodeLocation) -> ParseResult<&DocumentNode> {
        let mut current_nodes = &self.nodes;
        let mut node: Option<&DocumentNode> = None;

        for (i, &idx) in location.path.iter().enumerate() {
            if idx >= current_nodes.len() {
                return Err(ConfigError::custom(&format!(
                    "Invalid node path: index {} out of bounds at level {}",
                    idx, i
                )));
            }

            node = Some(&current_nodes[idx]);

            // If not the last index, navigate into child nodes
            if i < location.path.len() - 1 {
                current_nodes = match node.unwrap() {
                    DocumentNode::CategoryBlock { nodes: child_nodes, .. } => child_nodes,
                    DocumentNode::SpecialCategoryBlock { nodes: child_nodes, .. } => child_nodes,
                    _ => {
                        return Err(ConfigError::custom(&format!(
                            "Node at path index {} is not a category block",
                            i
                        )));
                    }
                };
            }
        }

        node.ok_or_else(|| ConfigError::custom("Empty node path"))
    }

    /// Find a mutable node by its location
    pub fn get_node_at_mut(&mut self, location: &NodeLocation) -> ParseResult<&mut DocumentNode> {
        let mut current_nodes = &mut self.nodes;

        for (i, &idx) in location.path.iter().enumerate() {
            if idx >= current_nodes.len() {
                return Err(ConfigError::custom(&format!(
                    "Invalid node path: index {} out of bounds at level {}",
                    idx, i
                )));
            }

            // If this is the last index, return the node
            if i == location.path.len() - 1 {
                return Ok(&mut current_nodes[idx]);
            }

            // Navigate to child nodes
            let node = &mut current_nodes[idx];
            current_nodes = match node {
                DocumentNode::CategoryBlock { nodes: child_nodes, .. } => child_nodes,
                DocumentNode::SpecialCategoryBlock { nodes: child_nodes, .. } => child_nodes,
                _ => {
                    return Err(ConfigError::custom(&format!(
                        "Node at path index {} is not a category block",
                        i
                    )));
                }
            };
        }

        Err(ConfigError::custom("Empty node path"))
    }

    /// Get all locations for a key
    pub fn get_locations(&self, key: &str) -> Option<&Vec<NodeLocation>> {
        self.key_index.get(key)
    }

    /// Update or insert a variable definition
    pub fn update_or_insert_variable(&mut self, name: &str, value: &str) -> ParseResult<()> {
        let key = format!("${}", name);

        if let Some(locations) = self.key_index.get(&key).cloned() {
            // Update existing variable (use first occurrence)
            let location = &locations[0];
            let node = self.get_node_at_mut(location)?;

            if let DocumentNode::VariableDef { value: old_value, raw, .. } = node {
                *old_value = value.to_string();
                *raw = format!("${} = {}", name, value);
            }
        } else {
            // Insert new variable at the beginning
            let new_node = DocumentNode::VariableDef {
                name: name.to_string(),
                value: value.to_string(),
                raw: format!("${} = {}", name, value),
                line: 1,
            };
            self.nodes.insert(0, new_node);
            self.rebuild_index();
        }

        Ok(())
    }

    /// Update or insert a value assignment
    pub fn update_or_insert_value(&mut self, key_path: &str, value: &str) -> ParseResult<()> {
        if let Some(locations) = self.key_index.get(key_path).cloned() {
            // Update existing value (use first occurrence)
            let location = &locations[0];
            let node = self.get_node_at_mut(location)?;

            if let DocumentNode::Assignment { value: old_value, raw, key, .. } = node {
                *old_value = value.to_string();
                *raw = format!("{} = {}", key.join(":"), value);
            }
        } else {
            // Insert new value
            let key_parts: Vec<String> = key_path.split(':').map(|s| s.to_string()).collect();
            let new_node = DocumentNode::Assignment {
                key: key_parts.clone(),
                value: value.to_string(),
                raw: format!("{} = {}", key_path, value),
                line: self.nodes.len() + 1,
            };
            self.nodes.push(new_node);
            self.rebuild_index();
        }

        Ok(())
    }

    /// Update or insert a handler call
    pub fn add_handler_call(&mut self, keyword: &str, value: &str) -> ParseResult<()> {
        let new_node = DocumentNode::HandlerCall {
            keyword: keyword.to_string(),
            flags: None,
            value: value.to_string(),
            raw: format!("{} = {}", keyword, value),
            line: self.nodes.len() + 1,
        };
        self.nodes.push(new_node);
        self.rebuild_index();
        Ok(())
    }

    /// Remove a value by key
    pub fn remove_value(&mut self, key_path: &str) -> ParseResult<()> {
        if let Some(locations) = self.key_index.get(key_path).cloned() {
            // Remove first occurrence
            let location = &locations[0];
            self.remove_node_at(location)?;
            self.rebuild_index();
        }
        Ok(())
    }

    /// Remove a variable
    pub fn remove_variable(&mut self, name: &str) -> ParseResult<()> {
        let key = format!("${}", name);
        if let Some(locations) = self.key_index.get(&key).cloned() {
            let location = &locations[0];
            self.remove_node_at(location)?;
            self.rebuild_index();
        }
        Ok(())
    }

    /// Remove a node at a specific location
    fn remove_node_at(&mut self, location: &NodeLocation) -> ParseResult<()> {
        if location.path.is_empty() {
            return Err(ConfigError::custom("Cannot remove root"));
        }

        let mut current_nodes = &mut self.nodes;

        // Navigate to parent
        for (i, &idx) in location.path.iter().enumerate() {
            if i == location.path.len() - 1 {
                // Last index - remove the node
                if idx >= current_nodes.len() {
                    return Err(ConfigError::custom(&format!(
                        "Invalid node path: index {} out of bounds",
                        idx
                    )));
                }
                current_nodes.remove(idx);
                return Ok(());
            }

            // Navigate deeper
            if idx >= current_nodes.len() {
                return Err(ConfigError::custom(&format!(
                    "Invalid node path: index {} out of bounds at level {}",
                    idx, i
                )));
            }

            let node = &mut current_nodes[idx];
            current_nodes = match node {
                DocumentNode::CategoryBlock { nodes: child_nodes, .. } => child_nodes,
                DocumentNode::SpecialCategoryBlock { nodes: child_nodes, .. } => child_nodes,
                _ => {
                    return Err(ConfigError::custom(&format!(
                        "Node at path index {} is not a category block",
                        i
                    )));
                }
            };
        }

        Err(ConfigError::custom("Failed to remove node"))
    }
}

impl Default for ConfigDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_document() {
        let doc = ConfigDocument::new();
        assert_eq!(doc.serialize(), "");
    }

    #[test]
    fn test_simple_assignment() {
        let nodes = vec![
            DocumentNode::Assignment {
                key: vec!["border_size".to_string()],
                value: "2".to_string(),
                raw: "border_size = 2".to_string(),
                line: 1,
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        assert_eq!(doc.serialize(), "border_size = 2\n");
    }

    #[test]
    fn test_variable_def() {
        let nodes = vec![
            DocumentNode::VariableDef {
                name: "GAPS".to_string(),
                value: "10".to_string(),
                raw: "$GAPS = 10".to_string(),
                line: 1,
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        assert_eq!(doc.serialize(), "$GAPS = 10\n");
    }

    #[test]
    fn test_comment_preservation() {
        let nodes = vec![
            DocumentNode::Comment {
                text: " This is a comment".to_string(),
                line: 1,
            },
            DocumentNode::Assignment {
                key: vec!["key".to_string()],
                value: "value".to_string(),
                raw: "key = value".to_string(),
                line: 2,
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        assert_eq!(doc.serialize(), "# This is a comment\nkey = value\n");
    }

    #[test]
    fn test_blank_line_preservation() {
        let nodes = vec![
            DocumentNode::Assignment {
                key: vec!["key1".to_string()],
                value: "value1".to_string(),
                raw: "key1 = value1".to_string(),
                line: 1,
            },
            DocumentNode::BlankLine { line: 2 },
            DocumentNode::Assignment {
                key: vec!["key2".to_string()],
                value: "value2".to_string(),
                raw: "key2 = value2".to_string(),
                line: 3,
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        assert_eq!(doc.serialize(), "key1 = value1\n\nkey2 = value2\n");
    }

    #[test]
    fn test_category_block() {
        let nodes = vec![
            DocumentNode::CategoryBlock {
                name: "general".to_string(),
                nodes: vec![
                    DocumentNode::Assignment {
                        key: vec!["border_size".to_string()],
                        value: "2".to_string(),
                        raw: "border_size = 2".to_string(),
                        line: 2,
                    },
                ],
                open_line: 1,
                close_line: 3,
                raw_open: "general {".to_string(),
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        assert_eq!(doc.serialize(), "general {\n  border_size = 2\n}\n");
    }

    #[test]
    fn test_nested_categories() {
        let nodes = vec![
            DocumentNode::CategoryBlock {
                name: "decoration".to_string(),
                nodes: vec![
                    DocumentNode::CategoryBlock {
                        name: "shadow".to_string(),
                        nodes: vec![
                            DocumentNode::Assignment {
                                key: vec!["enabled".to_string()],
                                value: "true".to_string(),
                                raw: "enabled = true".to_string(),
                                line: 3,
                            },
                        ],
                        open_line: 2,
                        close_line: 4,
                        raw_open: "shadow {".to_string(),
                    },
                ],
                open_line: 1,
                close_line: 5,
                raw_open: "decoration {".to_string(),
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        assert_eq!(doc.serialize(), "decoration {\n  shadow {\n    enabled = true\n  }\n}\n");
    }

    #[test]
    fn test_index_building() {
        let nodes = vec![
            DocumentNode::VariableDef {
                name: "GAPS".to_string(),
                value: "10".to_string(),
                raw: "$GAPS = 10".to_string(),
                line: 1,
            },
            DocumentNode::Assignment {
                key: vec!["border_size".to_string()],
                value: "2".to_string(),
                raw: "border_size = 2".to_string(),
                line: 2,
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);

        assert!(doc.get_locations("$GAPS").is_some());
        assert!(doc.get_locations("border_size").is_some());
        assert!(doc.get_locations("nonexistent").is_none());
    }

    #[test]
    fn test_get_node_at() {
        let nodes = vec![
            DocumentNode::Assignment {
                key: vec!["key1".to_string()],
                value: "value1".to_string(),
                raw: "key1 = value1".to_string(),
                line: 1,
            },
            DocumentNode::Assignment {
                key: vec!["key2".to_string()],
                value: "value2".to_string(),
                raw: "key2 = value2".to_string(),
                line: 2,
            },
        ];

        let doc = ConfigDocument::with_nodes(nodes);
        let location = NodeLocation {
            path: vec![0],
            node_type: NodeType::Assignment,
        };

        let node = doc.get_node_at(&location).unwrap();
        match node {
            DocumentNode::Assignment { key, .. } => {
                assert_eq!(key, &vec!["key1".to_string()]);
            }
            _ => panic!("Expected Assignment node"),
        }
    }
}
