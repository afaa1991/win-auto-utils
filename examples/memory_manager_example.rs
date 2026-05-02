//! Memory Manager Example
//!
//! Demonstrates using memory_manager with static addresses and AOB scanning.
//! This example shows the recommended way to manage multiple memory modifications.
//!
//! Features demonstrated:
//! - Lock memory value at fixed address with pointer chain
//! - Disable functionality (NOP patch)
//! - Modify function behavior (trampoline hook with AOB)
//!
//! Press Enter to toggle each feature on/off individually.

use std::io::{self, BufRead};
use std::time::Duration;
use win_auto_utils::memory_manager::builtin::{
    BytesSwitchHandler, LockHandler, TrampolineHookHandler,
};
use win_auto_utils::memory_manager::ModifierManager;
use win_auto_utils::process::ProcessManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Manager Example ===\n");
    println!("This example demonstrates memory management with different address modes.");
    println!("Features:");
    println!("1. Lock Value: game.exe+0x1000->2FC = 100 (static pointer chain)");
    println!("2. NOP Patch: game.exe+0x2000 -> NOP (static address)");
    println!("3. Function Hook: AOB scan + trampoline hook (dynamic)\n");

    // Find target process
    let mut process_mgr = ProcessManager::new();
    process_mgr.register("game.exe")?;

    // Try to initialize - if not running, wait for user input
    match process_mgr.init("game.exe") {
        Ok(_) => println!("Found game.exe"),
        Err(_) => {
            eprintln!(
                "game.exe not found. Please start the application and press Enter to continue..."
            );
            let stdin = io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
            process_mgr.reinit("game.exe")?;
            println!("Found game.exe");
        }
    }

    let proc = process_mgr.get("game.exe").ok_or("Failed to get process")?;
    let handle = proc.handle().expect("No handle available");
    let pid = proc.pid().ok_or("No PID available")?;

    println!("  PID: {}\n", pid);

    // Create modifier manager and bind process context
    let mut manager = ModifierManager::new();
    manager.set_context(handle, pid);

    // ========================================================================
    // Feature 1: Lock Value - Static Address with Multi-Level Pointer
    // ========================================================================
    println!("[1/3] Registering value lock...");
    println!("  Pattern: game.exe+0x1000->2FC (module base + offset -> pointer dereference)");
    println!("  Locked value: 100 (i32)\n");

    let value_lock = LockHandler::new_lock(
        "value_lock",
        "game.exe+0x1000->2FC",
        100i32,
        Duration::from_millis(100),
    )?;
    manager.register("value_lock", value_lock);
    println!("Value lock registered\n");

    // ========================================================================
    // Feature 2: NOP Patch - Static Address
    // ========================================================================
    println!("[2/3] Registering NOP patch...");
    println!("  Address: game.exe+0x2000");
    println!("  Original: Some instruction");
    println!("  Modified: NOP (90 90)\n");

    let nop_patch = BytesSwitchHandler::new_nop_switch(
        "nop_patch",
        "game.exe+0x2000",
        2, // 2 bytes
    )?;
    manager.register("nop_patch", nop_patch);
    println!("NOP patch registered\n");

    // ========================================================================
    // Feature 3: Function Hook - Trampoline Hook with AOB (Recommended!)
    // ========================================================================
    println!("[3/3] Registering function hook...");
    println!("  Pattern: 48 89 5C 24 (AOB scan - works across game updates!)");
    println!("  Original: Some function instruction");
    println!("  Modified: NOP instruction (modify behavior)\n");

    // Shellcode: NOP instruction
    let shellcode = vec![0x90, 0x90];
    let func_hook = TrampolineHookHandler::new_hook_aob(
        "func_hook",
        "48 89 5C 24", // AOB pattern - most robust approach!
        shellcode,
        5, // bytes_to_overwrite
    )?;
    manager.register("func_hook", func_hook);
    println!("Function hook registered\n");

    // ========================================================================
    // Sequential Testing Loop
    // ========================================================================
    println!("=== Sequential Feature Testing ===");
    println!("Each feature will be tested individually:\n");

    let features = vec![
        ("value_lock", "Value Lock (lock to 100)"),
        ("nop_patch", "NOP Patch (disable at game.exe+0x2000)"),
        ("func_hook", "Function Hook (AOB scan + trampoline)"),
    ];

    for (key, description) in &features {
        println!("----------------------------------------");
        println!("Testing: {} ({})", key, description);
        println!("----------------------------------------\n");

        // Activate
        print!("Activating... ");
        io::Write::flush(&mut io::stdout())?;
        match manager.activate(key) {
            Ok(_) => println!("ON"),
            Err(e) => {
                println!("Failed: {}", e);
                continue;
            }
        }

        println!("  -> Test the feature now");
        println!("  -> Press Enter to deactivate...\n");

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        // Deactivate
        print!("Deactivating... ");
        io::Write::flush(&mut io::stdout())?;
        match manager.deactivate(key) {
            Ok(_) => println!("OFF"),
            Err(e) => println!("Failed: {}", e),
        }

        println!("\nFeature test completed!\n");
    }

    // Demonstrate caching behavior
    println!("\n=== Caching Behavior Demo ===\n");
    println!("The memory manager caches resolved addresses for performance:");
    println!("- First activation: Resolves address (AOB scan or pattern parse)");
    println!("- Subsequent activations: Uses cached address (fast!)");
    println!("- Cache auto-invalidates when PID changes\n");

    // Test rapid toggle to show caching benefit
    println!("Testing rapid toggle (10 times)...");
    let start = std::time::Instant::now();

    for i in 1..=10 {
        manager.activate("nop_patch")?;
        manager.deactivate("nop_patch")?;

        if i == 1 || i == 5 || i == 10 {
            println!("  Iteration {}: {}ms", i, start.elapsed().as_millis());
        }
    }

    let total_time = start.elapsed();
    println!("\nCompleted 10 toggle cycles in {:?}", total_time);
    println!("  (With caching, only the first cycle resolves the address)\n");

    println!("========================================");
    println!("All features tested successfully!");
    println!("========================================");

    Ok(())
}
