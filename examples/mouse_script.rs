//! Mouse script engine example - demonstrates mouse automation with scripts
//!
//! This example shows how to use the script engine with mouse instructions,
//! including both foreground (SendInput) and background (PostMessage) modes.
//!
//! # Running this example
//! ```bash
//! cargo run --example mouse_script --features "script_engine,scripts_mouse"
//! ```

use win_auto_utils::script_engine::{ScriptConfig, ScriptEngine};
use win_auto_utils::scripts_builtin::register_all;
use win_auto_utils::script_engine::instruction::InstructionRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mouse Script Engine Example ===\n");

    // Setup: Register all builtin instructions
    let mut registry = InstructionRegistry::new();
    register_all(&mut registry);
    
    println!("✓ All builtin instructions registered");
    println!("Available instructions: {:?}\n", registry.list_instructions());
    
    let config = ScriptConfig::default();
    let engine = ScriptEngine::with_registry_and_config(registry, config);

    // Example 1: Foreground mode (default - SendInput)
    println!("--- Example 1: Foreground Mode (Default) ---");
    let script1 = r#"move 100 200
click 100 200
sleep 50
click
"#;
    
    println!("Script:\n{}\n", script1);
    println!("Note: These operations use SendInput API (global input)\n");
    let compiled = engine.compile(script1)?;
    engine.execute(&compiled)?;
    println!();

    // Example 2: Explicit foreground mode
    println!("--- Example 2: Explicit Foreground Mode ---");
    let script2 = r#"move 300 400 send
click 300 400 send
scrollup 1 send
"#;
    
    println!("Script:\n{}\n", script2);
    let compiled = engine.compile(script2)?;
    engine.execute(&compiled)?;
    println!();

    // Example 3: Background mode (PostMessage)
    println!("--- Example 3: Background Mode (PostMessage) ---");
    let script3 = r#"move 100 200 post
click 100 200 post
scrollup 1 post
scrolldown 2 post
"#;
    
    println!("Script:\n{}\n", script3);
    println!("Note: PostMessage operations require a valid window handle.");
    println!("      Set it with: vm.set_persistent_state(\"target_hwnd\", hwnd)");
    println!("      This persists across multiple execute() calls.\n");
    let _compiled = engine.compile(script3)?;
    
    // For demonstration, we'll skip execution without a real window handle
    println!("(Skipped execution - no target window set)\n");

    // Example 4: Relative movement and scrolling
    println!("--- Example 4: Relative Movement and Scrolling ---");
    let script4 = r#"moverel 50 -30
scrollup 120
sleep 50
scrolldown 240
"#;
    
    println!("Script:\n{}\n", script4);
    let compiled = engine.compile(script4)?;
    engine.execute(&compiled)?;
    println!();

    // Example 5: Press and release (drag operation simulation)
    println!("--- Example 5: Press and Release (Drag Simulation) ---");
    let script5 = r#"move 100 100
press
sleep 50
move 300 300
release
"#;
    
    println!("Script:\n{}\n", script5);
    let compiled = engine.compile(script5)?;
    engine.execute(&compiled)?;
    println!();

    // Example 6: Loop with mixed modes
    println!("--- Example 6: Loop with Mixed Modes ---");
    let script6 = r#"loop 3
    click 100 200
    sleep 50
end
"#;
    
    println!("Script:\n{}\n", script6);
    let compiled = engine.compile(script6)?;
    engine.execute(&compiled)?;
    println!();

    println!("=== Example Completed ===");
    println!("\nKey Differences:");
    println!("• SendInput (send): Global input, works with any application");
    println!("  - Can omit coordinates to use current cursor position");
    println!("  - Requires application to have focus for some operations");
    println!();
    println!("• PostMessage (post): Background input to specific window");
    println!("  - Must specify coordinates (x, y)");
    println!("  - Works even when window is not in focus");
    println!("  - Requires target_hwnd to be set in VM state");
    println!();
    println!("Usage: <instruction> [parameters] [send|post]");

    Ok(())
}
