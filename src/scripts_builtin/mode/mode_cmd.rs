//! Mode configuration instruction
//!
//! Implements the `mode` instruction for setting module-specific configuration modes.
//! Uses a key-value system where each module defines its own mode keys.
//!
//! # Syntax
//! ```text
//! mode <key> <value>
//! ```
//!
//! # Parameters
//! - `<key>` (required): Configuration key name (module-defined)
//! - `<value>` (required): Configuration value
//!
//! # Examples
//! ```text
//! # Keyboard/Mouse input mode configuration
//! mode input_mode post       # Set background input mode
//! key A                      # Uses PostMessage
//! click 100 200              # Uses PostMessage
//!
//! mode input_mode send       # Switch to foreground mode
//! key B                      # Uses SendInput
//!
//! # Future: Other module configurations
//! mode debug_level verbose   # Enable verbose debugging
//! mode performance_level max # Maximize performance
//! ```


use crate::script_engine::instruction::{
    InstructionData, InstructionHandler, InstructionMetadata, ScriptError,
};
use crate::script_engine::VMContext;

/// Mode configuration parameters
#[derive(Debug, Clone)]
pub struct ModeParams {
    /// Configuration key (module-defined)
    pub key: String,
    /// Configuration value
    pub value: String,
}

/// Mode instruction handler
pub struct ModeHandler;

impl InstructionHandler for ModeHandler {
    fn name(&self) -> &str {
        "mode"
    }

    fn parse(&self, args: &[&str]) -> Result<InstructionData, ScriptError> {
        if args.len() != 2 {
            return Err(ScriptError::ParseError(
                "Invalid parameters. Usage: mode <key> <value>".into(),
            ));
        }

        let key = args[0].to_string();
        let value = args[1].to_string();

        // Validate that key and value are not empty
        if key.is_empty() || value.is_empty() {
            return Err(ScriptError::ParseError(
                "Key and value cannot be empty".into(),
            ));
        }

        Ok(InstructionData::Custom(Box::new(ModeParams { key, value })))
    }

    fn execute(
        &self,
        vm: &mut VMContext,
        data: &InstructionData,
        _metadata: Option<&InstructionMetadata>,
    ) -> Result<(), ScriptError> {
        let params = data.extract_custom::<ModeParams>("Invalid mode parameters")?;

        // Set the mode configuration in VM context
        // Each module will read its relevant keys and adapt behavior
        vm.set_persistent_state(&params.key, params.value.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode_input_mode_post() {
        let handler = ModeHandler;
        let result = handler.parse(&["input_mode", "post"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<ModeParams>().unwrap();
                assert_eq!(params.key, "input_mode");
                assert_eq!(params.value, "post");
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_mode_input_mode_send() {
        let handler = ModeHandler;
        let result = handler.parse(&["input_mode", "send"]).unwrap();
        match result {
            InstructionData::Custom(boxed) => {
                let params = boxed.downcast_ref::<ModeParams>().unwrap();
                assert_eq!(params.key, "input_mode");
                assert_eq!(params.value, "send");
            }
            _ => panic!("Expected Custom data"),
        }
    }

    #[test]
    fn test_parse_mode_missing_parameters() {
        let handler = ModeHandler;
        
        // No parameters
        let result = handler.parse(&[]);
        assert!(result.is_err());
        
        // Only one parameter
        let result = handler.parse(&["input_mode"]);
        assert!(result.is_err());
        
        // Too many parameters
        let result = handler.parse(&["input_mode", "post", "extra"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_mode_configuration() {
        let handler = ModeHandler;
        let mut vm = VMContext::new();

        // Set input_mode to post
        let data = handler.parse(&["input_mode", "post"]).unwrap();
        handler.execute(&mut vm, &data, None).unwrap();

        // Verify the mode was set
        let mode = vm.get_persistent_state::<String>("input_mode").cloned();
        assert_eq!(mode, Some("post".to_string()));

        // Change to send
        let data = handler.parse(&["input_mode", "send"]).unwrap();
        handler.execute(&mut vm, &data, None).unwrap();

        let mode = vm.get_persistent_state::<String>("input_mode").cloned();
        assert_eq!(mode, Some("send".to_string()));
    }

    #[test]
    fn test_mode_independence_between_keys() {
        let handler = ModeHandler;
        let mut vm = VMContext::new();

        // Set different mode keys
        let data1 = handler.parse(&["input_mode", "post"]).unwrap();
        handler.execute(&mut vm, &data1, None).unwrap();

        let data2 = handler.parse(&["debug_level", "verbose"]).unwrap();
        handler.execute(&mut vm, &data2, None).unwrap();

        // Verify both are set independently
        let input_mode = vm.get_persistent_state::<String>("input_mode").cloned();
        let debug_level = vm.get_persistent_state::<String>("debug_level").cloned();

        assert_eq!(input_mode, Some("post".to_string()));
        assert_eq!(debug_level, Some("verbose".to_string()));
    }

    #[test]
    fn test_mode_overwrite_same_key() {
        let handler = ModeHandler;
        let mut vm = VMContext::new();

        // Set initial value
        let data1 = handler.parse(&["input_mode", "post"]).unwrap();
        handler.execute(&mut vm, &data1, None).unwrap();

        // Overwrite with new value
        let data2 = handler.parse(&["input_mode", "send"]).unwrap();
        handler.execute(&mut vm, &data2, None).unwrap();

        // Verify it was overwritten
        let mode = vm.get_persistent_state::<String>("input_mode").cloned();
        assert_eq!(mode, Some("send".to_string()));
    }
}
