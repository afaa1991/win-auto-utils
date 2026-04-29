//! Script parser
//!
//! Parses text scripts into AST (Abstract Syntax Tree). In the plugin-based architecture,
//! the Parser only identifies instruction names and extracts arguments. Semantic validation
//! is delegated to Handlers in the registry.
//!
//! # Parsing Process
//! 1. Split script into lines
//! 2. Skip empty lines and comments (lines starting with `//` or `#`)
//! 3. Extract command name and arguments
//! 4. Validate against registered instructions (in strict mode)
//!
//! # Comment Styles
//! The parser supports two comment styles:
//! - **Double slash**: `// This is a comment`
//! - **Hash symbol**: `# This is also a comment`
//!
//! Both styles can be used interchangeably within the same script.
//!
//! # Strict Mode
//! By default, the parser operates in **strict mode** (`strict_mode = true`).
//!
//! **In strict mode:**
//! - Unknown instructions cause a parse error immediately
//! - Helps catch typos and configuration issues early
//! - Recommended for production use
//!
//! **To disable strict mode:**
//! ```no_run
//! use win_auto_utils::script_engine::parser::{Parser, ParserConfig};
//! use win_auto_utils::script_engine::instruction::InstructionRegistry;
//!
//! let registry = InstructionRegistry::new();
//! let mut config = ParserConfig::default();
//! config.strict_mode = false; // Allow unknown instructions during parsing
//! let parser = Parser::new(config, &registry);
//! ```
//!
//! **Note:** Disabling strict mode is useful during development when you're testing
//! new custom instructions that haven't been registered yet. However, unregistered
//! instructions will still fail at compile time when the compiler tries to find their handlers.
//!
//! # Example
//! ```no_run
//! use win_auto_utils::script_engine::parser::{Parser, ParserConfig};
//! use win_auto_utils::script_engine::instruction::InstructionRegistry;
//!
//! let registry = InstructionRegistry::new();
//! let config = ParserConfig::default();
//! let parser = Parser::new(config, &registry);
//!
//! let script = "// comment\nkey A\n# another comment\nsleep 100";
//! let ast = parser.parse(script).unwrap();
//!
//! assert_eq!(ast.len(), 2);  // Only 'key' and 'sleep', comments skipped
//! assert_eq!(ast[0].command, "key");
//! assert_eq!(ast[0].args, vec!["A"]);
//! ```

use super::instruction::{InstructionRegistry, ScriptError};

/// Parsed AST node (simplified version)
#[derive(Debug)]
pub struct AstNode {
    pub line_num: usize,
    pub command: String,
    pub args: Vec<String>,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Strict mode: report error on unknown instructions
    pub strict_mode: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self { strict_mode: true }
    }
}

/// Script parser
pub struct Parser<'a> {
    config: ParserConfig,
    registry: &'a InstructionRegistry,
}

impl<'a> Parser<'a> {
    pub fn new(config: ParserConfig, registry: &'a InstructionRegistry) -> Self {
        Self { config, registry }
    }

    /// Parse script text
    pub fn parse(&self, text: &str) -> Result<Vec<AstNode>, ScriptError> {
        let mut nodes = Vec::new();

        for (line_num, line) in text.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            let node = self.parse_line(line, line_num + 1)?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    /// Parse single line
    fn parse_line(&self, line: &str, line_num: usize) -> Result<AstNode, ScriptError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ScriptError::ParseError(format!(
                "Empty line at {}",
                line_num
            )));
        }

        let command = parts[0].to_lowercase();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        // Validate instruction existence (if strict mode is enabled)
        if self.config.strict_mode && !self.registry.has_instruction(&command) {
            return Err(ScriptError::ParseError(format!(
                "Unknown command '{}' at line {}",
                command, line_num
            )));
        }

        Ok(AstNode {
            line_num,
            command,
            args,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let registry = InstructionRegistry::new();
        let config = ParserConfig { strict_mode: false };
        let parser = Parser::new(config, &registry);

        let ast = parser.parse("key a").unwrap();
        assert_eq!(ast.len(), 1);
        assert_eq!(ast[0].command, "key");
        assert_eq!(ast[0].args, vec!["a"]);
    }

    #[test]
    fn test_parse_with_comments() {
        let registry = InstructionRegistry::new();
        let config = ParserConfig { strict_mode: false };
        let parser = Parser::new(config, &registry);

        let ast = parser
            .parse("// comment\nkey a\n// another comment")
            .unwrap();
        assert_eq!(ast.len(), 1);
        assert_eq!(ast[0].command, "key");
    }
}
