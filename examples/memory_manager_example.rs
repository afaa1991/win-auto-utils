//! Memory Manager Example
//!
//! Demonstrates using memory_manager with both static addresses and AOB scanning.
//! This example shows the recommended way to manage multiple memory modifications.
//! 
//! Features demonstrated:
//! - Lock memory value at fixed value (static address)
//! - Disable functionality (NOP patch)
//! - Modify function behavior (trampoline hook)
//!
//! Press Enter to toggle each feature on/off individually.

use std::io::{self, BufRead};
use std::time::Duration;
use win_auto_utils::memory_manager::builtin::{
    BytesSwitchHandler, LockHandler, TrampolineHookHandler,
};
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::memory_resolver::AddressSource;
use win_auto_utils::process::ProcessManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Manager Example ===\n");
    println!("This example demonstrates the unified memory management approach.");
    println!("Features:");
    println!("1. Lock Value (Static): target_app.exe+0x1000 = 100");
    println!("2. NOP Patch (Static): target_app.exe+0x2000 → NOP");
    println!("3. Function Hook (Trampoline): Modify function behavior\n");

    // Find target process
    let mut process_mgr = ProcessManager::new();
    process_mgr.register("target_app.exe")?;

    // Try to initialize - if not running, wait for user input
    match process_mgr.init("target_app.exe") {
        Ok(_) => println!("✓ Found target_app.exe"),
        Err(_) => {
            eprintln!("target_app.exe not found. Please start the application and press Enter to continue...");
            let stdin = io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
            process_mgr.reinit("target_app.exe")?;
            println!("✓ Found target_app.exe");
        }
    }

    let proc = process_mgr.get("target_app.exe").ok_or("Failed to get process")?;
    let handle = proc.handle().expect("No handle available");
    let pid = proc.pid().ok_or("No PID available")?;

    println!("  PID: {}\n", pid);

    // Create modifier manager and bind process context
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);

    // ========================================================================
    // Feature 1: Lock Value - Static Address Mode
    // ========================================================================
    println!("[1/3] Registering value lock...");
    let value_lock = LockHandler::new_lock_x86_typed(
        "value_lock",
        "target_app.exe+0x1000",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);
    println!("✓ Value lock registered (locks value to 100)\n");

    // ========================================================================
    // Feature 2: NOP Patch - Static Address Mode
    // ========================================================================
    println!("[2/3] Registering NOP patch...");
    println!("  Address: target_app.exe+0x2000");
    println!("  Original: Some instruction");
    println!("  Modified: NOP (90 90)\n");
    
    let nop_patch = BytesSwitchHandler::new_nop_switch_x86(
        "nop_patch",
        "target_app.exe+0x2000",
        2, // 2 bytes
    )?;
    manager.register("nop_patch", nop_patch);
    println!("✓ NOP patch registered (disables functionality)\n");

    // ========================================================================
    // Feature 3: Function Hook - Trampoline Hook with AOB
    // ========================================================================
    println!("[3/3] Registering function hook...");
    println!("  Original: Some function instruction");
    println!("  Modified: NOP instruction (modify behavior)\n");
    
    // Shellcode: NOP instruction
    let shellcode = vec![0x90, 0x90];
    let func_hook = TrampolineHookHandler::new_x86_skip_trampoline(
        "func_hook",
        AddressSource::from_pattern_x86("target_app.exe+0x3000")?,
        shellcode,
        2, // bytes_to_overwrite
    );
    manager.register("func_hook", func_hook);
    println!("✓ Function hook registered (modifies function behavior)\n");

    // ========================================================================
    // Sequential Testing Loop
    // ========================================================================
    println!("=== Sequential Feature Testing ===");
    println!("Each feature will be tested individually:\n");

    let features = vec![
        ("value_lock", "Value Lock (lock to 100)"),
        ("nop_patch", "NOP Patch (disable at target_app.exe+0x2000)"),
        ("func_hook", "Function Hook (modify behavior)"),
    ];

    for (key, description) in &features {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Testing: {} ({})", key, description);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Activate
        print!("Activating... ");
        io::Write::flush(&mut io::stdout())?;
        match manager.activate(key) {
            Ok(_) => println!("✓ ON"),
            Err(e) => {
                println!("✗ Failed: {}", e);
                continue;
            }
        }

        println!("  → Test the feature now");
        println!("  → Press Enter to deactivate...\n");

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        // Deactivate
        print!("Deactivating... ");
        io::Write::flush(&mut io::stdout())?;
        match manager.deactivate(key) {
            Ok(_) => println!("✓ OFF"),
            Err(e) => println!("✗ Failed: {}", e),
        }

        println!("\nFeature test completed!\n");
    }

    println!("═══════════════════════════════════════");
    println!("All features tested successfully!");
    println!("═══════════════════════════════════════");

    Ok(())
}
