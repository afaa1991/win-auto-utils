//! Example demonstrating the new Process management architecture
//!
//! This example shows three usage patterns:
//! 1. Quick initialization (simple case)
//! 2. Configuration-based initialization with builder (advanced case)
//! 3. Manager-based initialization (multi-process case)

use std::io;
use win_auto_utils::process::{Process, ProcessConfig, ProcessManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Process Management Examples ===\n");

    // ==================== Pattern 1: Quick Initialization ====================
    println!("Pattern 1: Quick Initialization (Two-Step)");
    println!("-------------------------------------------");
    println!("Usage: Process::by_name(\"notepad.exe\") + init()");
    println!();

    let mut quick_process = Process::by_name("notepad.exe");

    match quick_process.init() {
        Ok(_) => {
            println!("✓ Initialized successfully!");
            println!("After initialization:");
            println!("  PID: {}", quick_process.pid_or_default());
            println!("  HWND: {:?}", quick_process.hwnd_or_default());
            println!("  HANDLE: {:?}", quick_process.handle_or_default());
            println!("  HDC: {:?}", quick_process.hdc_or_default());
            println!("  Valid: {}", quick_process.is_valid());
        }
        Err(e) => {
            println!("✗ Failed to initialize: {}", e);
            println!("  (Make sure notepad.exe is running)");
        }
    }
    println!();

    // ==================== Pattern 1.5: One-Step Initialization ====================
    println!("Pattern 1.5: One-Step Initialization (NEW!)");
    println!("--------------------------------------------");
    println!("Usage: Process::init_by_name(\"notepad.exe\") - Creates and initializes in one call");
    println!();

    println!("Press Enter to demonstrate one-step initialization...");
    io::stdin().read_line(&mut String::new())?;

    match Process::init_by_name("notepad.exe") {
        Ok(mut one_step_process) => {
            println!("✓ One-step initialization successful!");
            println!("  PID: {}", one_step_process.pid_or_default());
            println!("  HWND: {:?}", one_step_process.hwnd_or_default());
            println!("  Valid: {}", one_step_process.is_valid());

            // Clean up
            one_step_process.cleanup()?;
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
            println!("  (Make sure notepad.exe is running)");
        }
    }
    println!();

    // ==================== Pattern 3: Manager-Based (Multi-Process) ====================
    println!("Pattern 3: Manager-Based (Multi-Process)");
    println!("-----------------------------------------");
    println!("Usage: ProcessManager for managing multiple processes");
    println!("Demonstrating 4 registration methods:");
    println!("  1. register(name)");
    println!("  2. register_alias(alias, name)");
    println!("  3. register_config(config) - KEY FEATURE");
    println!("  4. register_config_alias(alias, config)");
    println!();

    let mut manager = ProcessManager::new(); // Need mut for modifications

    // Method 1: Simple registration (process name becomes the key)
    manager.register("notepad.exe")?;
    println!("✓ Method 1: Registered 'notepad.exe'");

    // Method 2: Registration with custom alias
    manager.register_alias("notepad_2", "notepad.exe")?;
    println!("✓ Method 2: Registered alias 'notepad_2' -> 'notepad.exe'");

    // Method 3: Registration with full config (advanced features)
    // This is the key feature: registering a process with specific window/DC configurations
    let config = ProcessConfig::builder("notepad.exe")
        .set_window_client_mode() // Use client area DC
        .exclude_invisible() // Skip hidden windows
        .build();
    manager.register_config_alias("notepad_config", config)?; // Use alias to avoid conflict
    println!("✓ Method 3: Registered 'notepad_config' with WindowClient config");

    // Method 4: Registration with config and custom alias
    let config2 = ProcessConfig::builder("notepad.exe")
        .set_desktop_mode() // Use desktop DC
        .build();
    manager.register_config_alias("notepad_desktop", config2)?;
    println!("✓ Method 4: Registered alias 'notepad_desktop' with Desktop config");

    println!();
    println!("Registered processes: {:?}", manager.list_processes());
    println!();

    println!("Press Enter to initialize all...");
    io::stdin().read_line(&mut String::new())?;

    // Initialize first process (simple registration)
    match manager.init("notepad.exe") {
        Ok(_) => println!("✓ notepad.exe (simple) initialized"),
        Err(e) => println!("✗ notepad.exe (simple) failed: {}", e),
    }

    // Initialize second process with specific PID (for distinguishing multiple instances)
    match manager.init_with_pid("notepad_2", 20908) {
        Ok(_) => println!("✓ notepad_2 initialized with PID 20908"),
        Err(e) => println!(
            "✗ notepad_2 failed: {} (make sure to replace 20908 with actual PID)",
            e
        ),
    }

    // Initialize third process (config-based, uses window filter)
    match manager.init("notepad_config") {
        Ok(_) => println!("✓ notepad_config initialized with window filter"),
        Err(e) => println!("✗ notepad_config failed: {}", e),
    }

    // Initialize fourth process (desktop mode)
    match manager.init("notepad_desktop") {
        Ok(_) => println!("✓ notepad_desktop initialized with desktop mode"),
        Err(e) => println!("✗ notepad_desktop failed: {}", e),
    }
    println!();

    // Query status using convenient accessors
    println!("Process Status (using *_or_default methods):");
    for name in manager.list_processes() {
        if let Some(proc) = manager.get(&name) {
            println!(
                "  {}: pid={}, hwnd={:?}, valid={}",
                name,
                proc.pid_or_default(),
                proc.hwnd_or_default(),
                proc.is_valid()
            );
        }
    }
    println!();

    // ==================== Pattern 3.5: Manager with Config Registration (Alternative Approach) ====================
    println!("Pattern 3.5: Manager with Config Registration (Alternative)");
    println!("------------------------------------------------------------");
    println!("Using a separate manager to demonstrate config-based registration works identically");
    println!();

    let mut manager2 = ProcessManager::new(); // New manager instance

    // Use ONLY config-based registration methods
    let config1 = ProcessConfig::builder("notepad.exe")
        .set_window_client_mode()
        .exclude_invisible()
        .build();
    manager2.register_config(config1)?;
    println!("✓ Registered 'notepad.exe' using register_config");

    let config2 = ProcessConfig::builder("notepad.exe")
        .set_desktop_mode()
        .build();
    manager2.register_config_alias("notepad_alt", config2)?;
    println!("✓ Registered alias 'notepad_alt' using register_config_alias");

    println!();
    println!("Registered processes: {:?}", manager2.list_processes());
    println!();

    println!("Press Enter to initialize both...");
    io::stdin().read_line(&mut String::new())?;

    // Initialize using config-registered names
    match manager2.init("notepad.exe") {
        Ok(_) => println!("✓ notepad.exe (config) initialized successfully"),
        Err(e) => println!("✗ notepad.exe (config) failed: {}", e),
    }

    match manager2.init_with_pid("notepad_alt", 20908) {
        Ok(_) => println!("✓ notepad_alt initialized with PID 20908"),
        Err(e) => println!(
            "✗ notepad_alt failed: {} (replace 20908 with actual PID)",
            e
        ),
    }
    println!();

    // Verify both managers work the same way
    println!("Verification - Both managers should show similar results:");
    for name in manager2.list_processes() {
        if let Some(proc) = manager2.get(&name) {
            println!(
                "  {}: pid={}, hwnd={:?}, valid={}",
                name,
                proc.pid_or_default(),
                proc.hwnd_or_default(),
                proc.is_valid()
            );
        }
    }
    println!();

    // Cleanup second manager
    manager2.cleanup_all()?;
    println!("✓ manager2 cleaned up");
    println!();

    // ==================== Cleanup ====================
    println!("Cleanup");
    println!("-------");
    println!("All resources will be automatically cleaned up when dropped.");
    println!("You can also manually cleanup:");

    quick_process.cleanup()?;
    println!("✓ quick_process cleaned up");

    manager.cleanup_all()?;
    println!("✓ All manager processes cleaned up");
    println!();

    println!("=== Example Complete ===");

    Ok(())
}
