//! Keyboard Usage Examples
//!
//! This example demonstrates various keyboard automation capabilities.
//!
//! Features demonstrated:
//! - Basic key clicks
//! - Keyboard shortcuts (Ctrl+C, Alt+Tab, etc.)
//! - Text input
//! - Background window input (PostMessage)
//! - High-performance atomic operations
//! - Script engine integration

use std::error::Error;
use std::time::Instant;

use win_auto_utils::keyboard::{
    keyboard_input, keyboard_message, PostMessageKeyboard, SendInputKeyboard,
};
use win_auto_utils::script_engine::ScriptEngine;
use windows::Win32::Foundation::HWND;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Keyboard Usage Examples ===\n");

    // Example 1: Basic keyboard usage with SendInput
    println!("1. Basic Keyboard Usage (SendInput)");
    basic_keyboard_example()?;
    println!();

    // Example 2: Keyboard shortcuts
    println!("2. Keyboard Shortcuts");
    keyboard_shortcuts_example()?;
    println!();

    // Example 3: Text input
    println!("3. Text Input");
    text_input_example()?;
    println!();

    // Example 4: High-performance atomic operations
    println!("4. High-Performance Atomic Operations");
    atomic_operations_example()?;
    println!();

    // Example 5: Script engine usage
    println!("5. Script Engine Usage");
    script_engine_example()?;
    println!();

    // Example 6: Background input (PostMessage) - requires valid window
    println!("6. Background Input (PostMessage)");
    println!("   Note: Requires a valid window handle to demonstrate");
    println!("   See the documentation for hwnd::find_window_by_title()");
    println!();

    println!("=== All examples completed! ===");
    Ok(())
}

fn basic_keyboard_example() -> Result<(), Box<dyn Error>> {
    let kb = SendInputKeyboard::new();

    println!("   Clicking 'A' key");
    kb.click("a")?;

    println!("   Pressing Shift key");
    kb.press("shift")?;

    println!("   Clicking 'B' key (with Shift held)");
    kb.click("b")?;

    println!("   Releasing Shift key");
    kb.release("shift")?;

    Ok(())
}

fn keyboard_shortcuts_example() -> Result<(), Box<dyn Error>> {
    let kb = SendInputKeyboard::new();

    println!("   Ctrl+C (Copy)");
    kb.press("ctrl")?;
    kb.click("c")?;
    kb.release("ctrl")?;

    println!("   Ctrl+V (Paste)");
    kb.press("ctrl")?;
    kb.click("v")?;
    kb.release("ctrl")?;

    println!("   Alt+Tab (Switch Window)");
    kb.press("alt")?;
    kb.click("tab")?;
    kb.release("alt")?;

    println!("   Win+R (Run Dialog)");
    kb.press("win")?;
    kb.click("r")?;
    kb.release("win")?;

    Ok(())
}

fn text_input_example() -> Result<(), Box<dyn Error>> {
    let kb = SendInputKeyboard::new();
    let text = "Hello, World!";

    println!("   Typing: '{}'", text);
    for ch in text.chars() {
        kb.click(&ch.to_string())?;
    }

    println!("   Pressing Enter");
    kb.click("enter")?;

    Ok(())
}

fn atomic_operations_example() -> Result<(), Box<dyn Error>> {
    // Pre-build INPUT structures at parse time (zero overhead)
    let inputs = keyboard_input::build_key_click_inputs(0x41, false); // 'A' key
    let key_down = keyboard_input::build_key_down_input(0x41, false);
    let key_up = keyboard_input::build_key_up_input(0x41, false);

    println!("   Benchmark: 1000 atomic clicks");
    let start = Instant::now();
    for _ in 0..1000 {
        keyboard_input::execute_inputs(&inputs)?;
    }
    let elapsed = start.elapsed();
    println!("   Completed in: {:?}", elapsed);
    println!("   Average per click: {:?}", elapsed / 1000);

    println!("\n   Individual key down/up with delay");
    keyboard_input::execute_single_input(&key_down)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    keyboard_input::execute_single_input(&key_up)?;

    Ok(())
}

fn script_engine_example() -> Result<(), Box<dyn Error>> {
    let engine = ScriptEngine::with_builtin();

    println!("   Basic key click script");
    engine.compile_and_execute("key a")?;

    println!("   Key with delay");
    engine.compile_and_execute("key enter 50")?;

    println!("   Key combination");
    engine.compile_and_execute(
        r#"
        key_down ctrl
        sleep 50
        key c
        key_up ctrl
        "#,
    )?;

    println!("   Complete workflow script");
    engine.compile_and_execute(
        r#"
        # Open Run dialog
        key_down win
        sleep 100
        key r
        key_up win
        sleep 500
        
        # Type notepad
        key n
        sleep 50
        key o
        sleep 50
        key t
        sleep 50
        key e
        sleep 50
        key p
        sleep 50
        key a
        sleep 50
        key d
        sleep 300
        
        # Launch
        key enter
        "#,
    )?;

    Ok(())
}

/// Example of using PostMessage for background input
/// This function is a template - you need to provide a valid HWND
#[allow(dead_code)]
fn background_input_example(hwnd: HWND) -> Result<(), Box<dyn Error>> {
    // High-level API
    let kb = PostMessageKeyboard::new(hwnd);
    kb.click("a")?;
    kb.press("ctrl")?;
    kb.click("c")?;
    kb.release("ctrl")?;

    // Low-level atomic API
    let vk_code = 0x41; // 'A'
    let scan_code = win_auto_utils::keyboard::get_scan_code(vk_code);

    keyboard_message::post_key_click_atomic(hwnd, vk_code, scan_code)?;
    keyboard_message::post_key_down_atomic(hwnd, vk_code, scan_code)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    keyboard_message::post_key_up_atomic(hwnd, vk_code, scan_code)?;

    Ok(())
}
