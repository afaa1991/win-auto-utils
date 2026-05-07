//! Copy instruction handler
//!
//! Implements the `copy` instruction for copying text to clipboard.

use crate::clipboard;
use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Copy handler - copy text to clipboard
///
/// Syntax: `copy <text>`
///
/// # Examples
/// ```text
/// copy Hello World        # Copy "Hello World" to clipboard
/// copy "Hello World"      # Copy "Hello World" to clipboard (with quotes)
/// copy "Line 1\nLine 2"   # Copy multi-line text (escaped newlines)
/// ```
///
/// # Notes
/// - Text with spaces should be quoted with double quotes
/// - Escape sequences like \n, \t are supported
/// - Maximum clipboard text size depends on system limits (typically ~1GB)
pub struct CopyHandler;

impl InstructionHandler for CopyHandler {
    fn name(&self) -> &str {
        "copy"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.is_empty() {
            return Err(ScriptError::ParseError(
                "Missing text to copy. Usage: copy <text>".into(),
            ));
        }

        // Join all arguments (handles quoted strings split by parser)
        let text = args.join(" ");
        
        // Process escape sequences
        let processed_text = process_escape_sequences(&text);

        Ok(InstructionData::Custom(Box::new(processed_text)))
    }

    fn execute(
        &self,
        _vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let text = data.extract_custom::<String>("Invalid copy data")?;

        clipboard::set_text(text).map_err(|e| {
            ScriptError::ExecutionError(format!("Failed to copy to clipboard: {}", e))
        })
    }
}

/// Process escape sequences in the input string
fn process_escape_sequences(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next_char) = chars.peek() {
                match next_char {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    '"' => {
                        result.push('"');
                        chars.next();
                    }
                    _ => {
                        result.push('\\');
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_copy_simple() {
        let handler = CopyHandler;
        let result = handler.parse(&["Hello", "World"]).unwrap();

        match result {
            InstructionData::Custom(boxed) => {
                let text = boxed.downcast_ref::<String>().unwrap();
                assert_eq!(*text, "Hello World");
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_copy_empty() {
        let handler = CopyHandler;
        assert!(handler.parse(&[]).is_err());
    }

    #[test]
    fn test_escape_sequences() {
        assert_eq!(process_escape_sequences("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(process_escape_sequences("Line1\\tLine2"), "Line1\tLine2");
        assert_eq!(process_escape_sequences("C:\\\\Users"), "C:\\Users");
        assert_eq!(process_escape_sequences("Quote:\\\"test\\\""), "Quote:\"test\"");
    }
}
