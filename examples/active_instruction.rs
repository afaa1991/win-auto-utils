//! Active Instruction Example
//!
//! Demonstrates how to use the `active` instruction to ensure a window is in the foreground
//! before executing input operations.
//!
//! # Feature Requirements
//! This example requires:
//! - `scripts_window` feature (provides the active instruction)
//! - `script_process_context` feature (provides HWND support)
//! - `hwnd` feature (for window enumeration)
//!
//! # Build and Run
//! ```bash
//! cargo run --example active_instruction --features "scripts_window,hwnd"
//! ```

use win_auto_utils::hwnd::get_hwnd_by_title;
use win_auto_utils::script_engine::{InterruptController, ScriptEngine};

fn main() {
    println!("=== Active Instruction Demo ===\n");

    // ========================================================================
    // Step 1: Create script engine with built-in instructions
    // ========================================================================
    println!("1. Setting up script engine with window activation support...");

    let engine = ScriptEngine::with_builtin();
    println!("   ✓ Engine ready with all built-in instructions (including 'active')");

    // ========================================================================
    // Step 2: Find target window
    // ========================================================================
    println!("\n2. Finding target window...");

    let window_title = "Notepad";
    if let Some(hwnd) = get_hwnd_by_title(window_title) {
        println!("   ✓ Found {}: {:?}", window_title, hwnd);

        // Note: The 'active' instruction will automatically retrieve HWND from VM context
        // No need to manually set it - the engine handles this internally
    } else {
        println!(
            "   ✗ {} not found. Please open Notepad and try again.",
            window_title
        );
        return;
    }

    // ========================================================================
    // Step 3: Execute script with 'active' instruction
    // ========================================================================
    println!("\n3. Executing script with 'active' instruction...");

    let script = r#"
        # Ensure window is in foreground before sending input
        active
        
        # Now safe to send keyboard input
        key H
        key E
        key L
        key L
        key O
    "#;

    println!("   Script content:");
    for line in script.lines() {
        println!("     {}", line);
    }

    println!("\n   Executing...");

    // Compile once for better performance
    match engine.compile(script) {
        Ok(compiled_script) => {
            // Execute with interrupt control for safety
            let interrupt = InterruptController::new();

            match engine.execute_with_interrupt(&compiled_script, &interrupt) {
                Ok(_) => println!("   ✓ Script executed successfully!"),
                Err(e) => {
                    println!("   ✗ Execution failed: {}", e);
                    println!("\n   Note: If you see 'activation timed out', it means");
                    println!("   Windows blocked the activation (foreground lock).");
                    println!("   Try clicking on the Notepad window manually first.");
                }
            }
        }
        Err(e) => {
            println!("   ✗ Compilation failed: {}", e);
        }
    }

    // ========================================================================
    // Step 4: Demonstrate error handling
    // ========================================================================
    println!("\n4. Error Handling Demo");
    println!("   Testing what happens when 'active' fails...");

    // Create a simple test without proper window setup
    let test_script = "active";

    match engine.compile_and_execute(test_script) {
        Ok(_) => println!("   Unexpected success (window was already active)"),
        Err(e) => {
            println!("   ✓ Expected behavior: {}", e);
            println!("   This protects against sending input to wrong window!");
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n=== Key Takeaways ===");
    println!("✓ The 'active' instruction ensures window is in foreground");
    println!("✓ Automatically retrieves HWND from VM context");
    println!("✓ Uses adaptive polling (not fixed delay) for verification");
    println!("✓ Fails safely if activation is blocked by Windows");
    println!("✓ Essential for reliable keyboard/mouse automation");

    println!("\n=== Best Practices ===");
    println!("1. Always call 'active' before input operations");
    println!("2. Check execution errors for activation failures");
    println!("3. Consider user interaction if activation is blocked");
    println!("4. Use reasonable timeouts (default 300ms works well)");
}
