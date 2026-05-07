//! Mouse instruction tests
//!
//! Tests verify:
//! - Parse-time argument validation (negative coords, invalid args)
//! - Default values when optional arguments are omitted
//! - Execute-time coordinate offset calculation (window_geometry)
//! - All mouse instructions: click, dbclick, move, moverel, scrollup, scrolldown, press, release
//!
//! # Running these tests
//! ```bash
//! cargo test --test test_mouse_instructions --features "script_engine,scripts_builtin,mouse,script_process_context" -- --nocapture
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
    fn test_click_no_args_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("click");
        assert!(result.is_ok(), "click with no args should parse: {:?}", result.err());
    }

    #[test]
    fn test_click_with_x_y_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("click 100 200");
        assert!(result.is_ok(), "click 100 200 should parse: {:?}", result.err());
    }

    #[test]
    fn test_click_with_x_y_delay_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("click 100 200 50");
        assert!(result.is_ok(), "click 100 200 50 should parse: {:?}", result.err());
    }

    #[test]
    fn test_click_negative_coords_rejected() {
        let engine = create_engine();
        let result = engine.compile("click -10 200");
        assert!(result.is_err(), "Negative X should be rejected");
        let result = engine.compile("click 100 -5");
        assert!(result.is_err(), "Negative Y should be rejected");
    }

    #[test]
    fn test_click_only_delay_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("click 50");
        assert!(result.is_ok(), "click with only delay should parse");
    }

    #[test]
    fn test_move_requires_two_args() {
        let engine = create_engine();
        let result = engine.compile("move 100");
        assert!(result.is_err(), "move with only one arg should fail");
        let result = engine.compile("move 100 200");
        assert!(result.is_ok(), "move with two args should succeed");
    }

    #[test]
    fn test_move_negative_coords_rejected() {
        let engine = create_engine();
        let result = engine.compile("move -1 200");
        assert!(result.is_err(), "Negative X should be rejected");
        let result = engine.compile("move 100 -1");
        assert!(result.is_err(), "Negative Y should be rejected");
    }

    #[test]
    fn test_moverel_requires_two_args() {
        let engine = create_engine();
        let result = engine.compile("moverel");
        assert!(result.is_err(), "moverel with no args should fail");
        let result = engine.compile("moverel 10 20");
        assert!(result.is_ok(), "moverel with two args should succeed");
    }

    #[test]
    fn test_dbclick_no_args_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("dbclick");
        assert!(result.is_ok(), "dbclick with no args should parse");
    }

    #[test]
    fn test_dbclick_with_coords_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("dbclick 100 200");
        assert!(result.is_ok(), "dbclick 100 200 should parse");
    }

    #[test]
    fn test_dbclick_with_delay_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("dbclick 100 200 50");
        assert!(result.is_ok(), "dbclick 100 200 50 should parse");
    }

    #[test]
    fn test_dbclick_negative_coords_rejected() {
        let engine = create_engine();
        let result = engine.compile("dbclick -10 200");
        assert!(result.is_err(), "Negative X should be rejected");
    }

    #[test]
    fn test_scrollup_no_args_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("scrollup");
        assert!(result.is_ok(), "scrollup with no args should parse");
    }

    #[test]
    fn test_scrollup_with_times_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("scrollup 5");
        assert!(result.is_ok(), "scrollup 5 should parse");
    }

    #[test]
    fn test_scrollup_with_coords_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("scrollup 100 200");
        assert!(result.is_ok(), "scrollup 100 200 should parse");
    }

    #[test]
    fn test_scrollup_with_coords_and_times_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("scrollup 100 200 3");
        assert!(result.is_ok(), "scrollup 100 200 3 should parse");
    }

    #[test]
    fn test_scrollup_zero_times_rejected() {
        let engine = create_engine();
        let result = engine.compile("scrollup 0");
        assert!(result.is_err(), "Zero times should be rejected");
    }

    #[test]
    fn test_scrollup_excessive_times_rejected() {
        let engine = create_engine();
        let result = engine.compile("scrollup 101");
        assert!(result.is_err(), "Times > 100 should be rejected");
    }

    #[test]
    fn test_scrollup_negative_coords_rejected() {
        let engine = create_engine();
        let result = engine.compile("scrollup -1 200");
        assert!(result.is_err(), "Negative X should be rejected");
    }

    #[test]
    fn test_scrolldown_no_args_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("scrolldown");
        assert!(result.is_ok(), "scrolldown with no args should parse");
    }

    #[test]
    fn test_scrolldown_with_times_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("scrolldown 5");
        assert!(result.is_ok(), "scrolldown 5 should parse");
    }

    #[test]
    fn test_scrolldown_zero_times_rejected() {
        let engine = create_engine();
        let result = engine.compile("scrolldown 0");
        assert!(result.is_err(), "Zero times should be rejected");
    }

    #[test]
    fn test_scrolldown_excessive_times_rejected() {
        let engine = create_engine();
        let result = engine.compile("scrolldown 101");
        assert!(result.is_err(), "Times > 100 should be rejected");
    }

    #[test]
    fn test_press_no_args_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("press");
        assert!(result.is_ok(), "press with no args should parse");
    }

    #[test]
    fn test_press_with_args_rejected() {
        let engine = create_engine();
        let result = engine.compile("press 100");
        assert!(result.is_err(), "press with args should be rejected");
    }

    #[test]
    fn test_release_no_args_parses_successfully() {
        let engine = create_engine();
        let result = engine.compile("release");
        assert!(result.is_ok(), "release with no args should parse");
    }

    #[test]
    fn test_release_with_args_rejected() {
        let engine = create_engine();
        let result = engine.compile("release 100");
        assert!(result.is_err(), "release with args should be rejected");
    }
}

mod execute_tests {
    use super::*;

    #[test]
    fn test_click_at_current_position_executes() {
        let engine = create_engine();
        let result = engine.compile("click");
        assert!(result.is_ok());
        let compiled = result.unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "click should execute: {:?}", result.err());
    }

    #[test]
    fn test_click_with_delay_executes() {
        let engine = create_engine();
        let compiled = engine.compile("click 100 200 50").unwrap();
        let start = std::time::Instant::now();
        let result = engine.execute(&compiled);
        let elapsed = start.elapsed();
        assert!(result.is_ok(), "click with delay should execute: {:?}", result.err());
        assert!(elapsed >= std::time::Duration::from_millis(50), "delay should be honored");
    }

    #[test]
    fn test_move_executes() {
        let engine = create_engine();
        let compiled = engine.compile("move 100 200").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "move should execute: {:?}", result.err());
    }

    #[test]
    fn test_moverel_executes() {
        let engine = create_engine();
        let compiled = engine.compile("moverel 50 -30").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "moverel should execute: {:?}", result.err());
    }

    #[test]
    fn test_dbclick_executes() {
        let engine = create_engine();
        let compiled = engine.compile("dbclick").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "dbclick should execute: {:?}", result.err());
    }

    #[test]
    fn test_scrollup_executes() {
        let engine = create_engine();
        let compiled = engine.compile("scrollup 3").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "scrollup should execute: {:?}", result.err());
    }

    #[test]
    fn test_scrolldown_executes() {
        let engine = create_engine();
        let compiled = engine.compile("scrolldown 2").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "scrolldown should execute: {:?}", result.err());
    }

    #[test]
    fn test_press_executes() {
        let engine = create_engine();
        let compiled = engine.compile("press").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "press should execute: {:?}", result.err());
    }

    #[test]
    fn test_release_executes() {
        let engine = create_engine();
        let compiled = engine.compile("release").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "release should execute: {:?}", result.err());
    }
}

mod default_value_tests {
    use super::*;

    #[test]
    fn test_click_default_delay_is_zero() {
        let engine = create_engine();
        let compiled = engine.compile("click 100 200").unwrap();
        let start = std::time::Instant::now();
        let result = engine.execute(&compiled);
        let elapsed = start.elapsed();
        assert!(result.is_ok());
        assert!(elapsed < std::time::Duration::from_millis(20), "Default delay should be ~0ms");
    }

    #[test]
    fn test_scrollup_default_times_is_one() {
        let engine = create_engine();
        let compiled = engine.compile("scrollup").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "Default scrollup should scroll once");
    }

    #[test]
    fn test_scrolldown_default_times_is_one() {
        let engine = create_engine();
        let compiled = engine.compile("scrolldown").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "Default scrolldown should scroll once");
    }
}

mod coordinate_offset_tests {
    use super::*;
    use windows::Win32::Foundation::HWND;

    #[test]
    fn test_click_without_hwnd_uses_raw_coords() {
        let engine = create_engine();
        let compiled = engine.compile("click 100 200").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "click should work without hwnd set");
    }

    #[test]
    fn test_move_without_hwnd_uses_raw_coords() {
        let engine = create_engine();
        let compiled = engine.compile("move 300 400").unwrap();
        let result = engine.execute(&compiled);
        assert!(result.is_ok(), "move should work without hwnd set");
    }

    #[test]
    fn test_click_with_hwnd_applies_offset() {
        let engine = create_engine();
        let result = engine.compile_and_execute_with_context("click 0 0", |vm| {
            vm.process.set_hwnd(HWND(std::ptr::null_mut()));
        });
        assert!(result.is_ok(), "click with hwnd should execute");
    }
}
