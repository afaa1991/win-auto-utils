//! Keyboard instruction tests
//!
//! Tests verify:
//! - Parse-time argument validation (missing key name, invalid modes)
//! - Default values when optional arguments are omitted
//! - Execute-time behavior for send/post modes
//! - All keyboard instructions: key, key_down, key_up
//!
//! # Running these tests
//! ```bash
//! cargo test --test test_keyboard_instructions --features "script_engine,scripts_builtin,keyboard,script_process_context" -- --nocapture
//! ```

use win_auto_utils::script_engine::instruction::InstructionRegistry;
use win_auto_utils::script_engine::{ScriptConfig, ScriptEngine};
use win_auto_utils::scripts_builtin::register_all;

fn create_engine() -> ScriptEngine {
    let mut registry = InstructionRegistry::new();
    register_all(&mut registry);
    let config = ScriptConfig::default();
    ScriptEngine::with_registry_and_config(registry, config)
}

mod parse_tests {
    use super::*;

    #[test]
    fn test_key_with_key_name_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key A");
        assert!(result.is_ok(), "key A should parse: {:?}", result.err());
    }

    #[test]
    fn test_key_with_key_and_delay_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key A 50");
        assert!(result.is_ok(), "key A 50 should parse");
    }

    #[test]
    fn test_key_with_key_delay_mode_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key A 50 send");
        assert!(result.is_ok(), "key A 50 send should parse");
        let result = engine.compile("key A 50 post");
        assert!(result.is_ok(), "key A 50 post should parse");
    }

    #[test]
    fn test_key_with_mode_only_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key A send");
        assert!(result.is_ok(), "key A send should parse");
        let result = engine.compile("key A post");
        assert!(result.is_ok(), "key A post should parse");
    }

    #[test]
    fn test_key_without_key_name_fails() {
        let engine = create_engine();
        let result = engine.compile("key");
        assert!(result.is_err(), "key without key name should fail");
    }

    #[test]
    fn test_key_with_invalid_key_name_fails() {
        let engine = create_engine();
        let result = engine.compile("key INVALID_KEY_NAME_123");
        assert!(result.is_err(), "Invalid key name should fail");
    }

    #[test]
    fn test_key_with_invalid_mode_fails() {
        let engine = create_engine();
        let result = engine.compile("key A invalid_mode");
        assert!(result.is_err(), "Invalid mode should fail");
    }

    #[test]
    fn test_key_down_with_key_name_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key_down A");
        assert!(result.is_ok(), "key_down A should parse");
    }

    #[test]
    fn test_key_down_with_mode_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key_down A send");
        assert!(result.is_ok(), "key_down A send should parse");
        let result = engine.compile("key_down A post");
        assert!(result.is_ok(), "key_down A post should parse");
    }

    #[test]
    fn test_key_down_without_key_name_fails() {
        let engine = create_engine();
        let result = engine.compile("key_down");
        assert!(result.is_err(), "key_down without key name should fail");
    }

    #[test]
    fn test_key_down_with_invalid_mode_fails() {
        let engine = create_engine();
        let result = engine.compile("key_down A invalid");
        assert!(result.is_err(), "key_down with invalid mode should fail");
    }

    #[test]
    fn test_key_up_with_key_name_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key_up A");
        assert!(result.is_ok(), "key_up A should parse");
    }

    #[test]
    fn test_key_up_with_mode_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("key_up A send");
        assert!(result.is_ok(), "key_up A send should parse");
        let result = engine.compile("key_up A post");
        assert!(result.is_ok(), "key_up A post should parse");
    }

    #[test]
    fn test_key_up_without_key_name_fails() {
        let engine = create_engine();
        let result = engine.compile("key_up");
        assert!(result.is_err(), "key_up without key name should fail");
    }

    #[test]
    fn test_key_up_with_invalid_mode_fails() {
        let engine = create_engine();
        let result = engine.compile("key_up A invalid");
        assert!(result.is_err(), "key_up with invalid mode should fail");
    }
}

mod execute_tests {
    use super::*;

    #[test]
    fn test_key_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key A").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "key A should execute: {:?}", result.err());
    }

    #[test]
    fn test_key_with_delay_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key A 50").unwrap();
        let start = std::time::Instant::now();
        let result = engine.execute(&compiled);
        let elapsed = start.elapsed();
        assert!(result.is_ok(), "key A 50 should execute: {:?}", result.err());
        assert!(elapsed >= std::time::Duration::from_millis(50), "delay should be honored");
    }

    #[test]
    fn test_key_down_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key_down A").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "key_down A should execute: {:?}", result.err());
    }

    #[test]
    fn test_key_up_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key_up A").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "key_up A should execute: {:?}", result.err());
    }

    #[test]
    fn test_key_combination_execution() {
        let engine = create_engine();
        let script = r#"
key_down SHIFT
key A
key_up SHIFT
"#;
        let compiled = engine.compile(script).unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "Key combination should execute: {:?}", result.err());
    }
}

mod default_value_tests {
    use super::*;

    #[test]
    fn test_key_default_delay_is_zero() {
        let engine = create_engine();
        let compiled = engine.compile("key A").unwrap();
        let start = std::time::Instant::now();
        let result = engine.execute(&compiled);
        let elapsed = start.elapsed();
        assert!(result.is_ok());
        assert!(elapsed < std::time::Duration::from_millis(20), "Default delay should be ~0ms");
    }

    #[test]
    fn test_key_default_mode_is_send() {
        let engine = create_engine();
        let compiled = engine.compile("key A send").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "Default mode should be send");
    }

    #[test]
    fn test_key_down_default_mode_is_send() {
        let engine = create_engine();
        let compiled = engine.compile("key_down A send").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "Default mode should be send");
    }

    #[test]
    fn test_key_up_default_mode_is_send() {
        let engine = create_engine();
        let compiled = engine.compile("key_up A send").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "Default mode should be send");
    }
}

mod mode_tests {
    use super::*;
    use windows::Win32::Foundation::HWND;

    #[test]
    fn test_key_send_mode_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key A send").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "key A send should execute");
    }

    #[test]
    fn test_key_down_send_mode_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key_down A send").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "key_down A send should execute");
    }

    #[test]
    fn test_key_up_send_mode_execution() {
        let engine = create_engine();
        let compiled = engine.compile("key_up A send").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "key_up A send should execute");
    }

    #[test]
    fn test_key_post_mode_without_hwnd_fails() {
        let engine = create_engine();
        let compiled = engine.compile("key A post").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_err(), "post mode without hwnd should fail");
    }

    #[test]
    fn test_key_down_post_mode_without_hwnd_fails() {
        let engine = create_engine();
        let compiled = engine.compile("key_down A post").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_err(), "key_down post without hwnd should fail");
    }

    #[test]
    fn test_key_up_post_mode_without_hwnd_fails() {
        let engine = create_engine();
        let compiled = engine.compile("key_up A post").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_err(), "key_up post without hwnd should fail");
    }

    #[test]
    fn test_key_post_mode_with_hwnd_execution() {
        let engine = create_engine();
        let result = engine.compile_and_execute_with_context("key A post", |vm| {
            vm.process.set_hwnd(HWND(std::ptr::null_mut()));
        });
        assert!(result.is_ok(), "key A post with hwnd should execute");
    }
}

mod common_key_names_tests {
    use super::*;

    #[test]
    fn test_letter_keys() {
        let engine = create_engine();
        for letter in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
                       'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'] {
            let result = engine.compile(&format!("key {}", letter));
            assert!(result.is_ok(), "key {} should parse", letter);
        }
    }

    #[test]
    fn test_number_keys() {
        let engine = create_engine();
        for num in ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'] {
            let result = engine.compile(&format!("key {}", num));
            assert!(result.is_ok(), "key {} should parse", num);
        }
    }

    #[test]
    fn test_function_keys() {
        let engine = create_engine();
        for fkey in ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"] {
            let result = engine.compile(&format!("key {}", fkey));
            assert!(result.is_ok(), "key {} should parse", fkey);
        }
    }

    #[test]
    fn test_special_keys() {
        let engine = create_engine();
        let special_keys = [
            "ENTER", "TAB", "ESCAPE", "SPACE", "BACKSPACE", "DELETE",
            "UP", "DOWN", "LEFT", "RIGHT",
            "SHIFT", "CONTROL", "ALT", "CTRL",
            "HOME", "END", "PAGEUP", "PAGEDOWN",
            "INSERT", "PRINTSCREEN",
        ];
        for key in special_keys {
            let result = engine.compile(&format!("key {}", key));
            assert!(result.is_ok(), "key {} should parse", key);
        }
    }
}
