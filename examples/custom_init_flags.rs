//! Example: Custom initialization flags for process resources
//!
//! This example demonstrates how to use InitFlags to control which resources
//! (HWND, HANDLE, HDC) are initialized during process connection.
//! This allows you to choose the right initialization strategy for different use cases.
//!
//! # Initialization Strategies
//!
//! Different tasks require different resources. Use InitFlags to initialize only what you need:
//! - **Minimal**: Only PID + HWND for basic window operations
//! - **Memory-only**: PID + HANDLE for memory read/write operations
//! - **GUI-only**: PID + HWND + HDC for screen capture and GUI automation
//! - **Custom**: Fine-grained control over each resource

use win_auto_utils::process::{InitFlags, Process, ProcessConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Process Initialization Flags Example ===\n");
    println!("Note: This example requires a running target process.\n");

    // Example 1: Default initialization (all resources)
    println!("1. Default initialization (PID + HWND + HANDLE + HDC):");
    println!("   Use case: Full access to process resources");
    let config = ProcessConfig::builder("notepad.exe").build();
    let mut process = Process::new(config);

    match process.init() {
        Ok(_) => {
            println!("   ✓ PID: {:?}", process.pid());
            println!("   ✓ HWND: {:?}", process.hwnd());
            println!("   ✓ HANDLE: {:?}", process.handle());
            println!("   ✓ HDC: {:?}", process.hdc());
        }
        Err(e) => println!("   ✗ Failed: {} (start notepad.exe to test)", e),
    }
    process.cleanup()?;

    // Example 2: Minimal initialization (only PID)
    println!("\n2. Minimal initialization (PID only):");
    println!("   Use case: Process identification without additional resources");
    let config = ProcessConfig::builder("notepad.exe")
        .init_flags(InitFlags::minimal())
        .build();
    let mut process = Process::new(config);

    match process.init() {
        Ok(_) => {
            println!("   ✓ PID: {:?}", process.pid());
            println!("   ✗ HWND: {:?} (not initialized)", process.hwnd());
            println!("   ✗ HANDLE: {:?} (not initialized)", process.handle());
            println!("   ✗ HDC: {:?} (not initialized)", process.hdc());
        }
        Err(e) => println!("   ✗ Failed: {} (start notepad.exe to test)", e),
    }
    process.cleanup()?;

    // Example 3: Memory-only operations (PID + HANDLE, no GUI resources)
    println!("\n3. Memory-only initialization (PID + HANDLE):");
    println!("   Use case: Memory read/write operations without GUI interaction");
    let config = ProcessConfig::builder("notepad.exe")
        .init_flags(InitFlags::memory_only())
        .build();
    let mut process = Process::new(config);

    match process.init() {
        Ok(_) => {
            println!("   ✓ PID: {:?}", process.pid());
            println!("   ✗ HWND: {:?} (not initialized)", process.hwnd());
            println!("   ✓ HANDLE: {:?}", process.handle());
            println!("   ✗ HDC: {:?} (not initialized)", process.hdc());
        }
        Err(e) => println!("   ✗ Failed: {} (start notepad.exe to test)", e),
    }
    process.cleanup()?;

    // Example 4: GUI-only operations (PID + HWND + DC, no process handle)
    println!("\n4. GUI-only initialization (PID + HWND + HDC):");
    println!("   Use case: Screen capture or GUI automation without memory access");
    let config = ProcessConfig::builder("notepad.exe")
        .init_flags(InitFlags::gui_only())
        .build();
    let mut process = Process::new(config);

    match process.init() {
        Ok(_) => {
            println!("   ✓ PID: {:?}", process.pid());
            println!("   ✓ HWND: {:?}", process.hwnd());
            println!("   ✗ HANDLE: {:?} (not initialized)", process.handle());
            println!("   ✓ HDC: {:?}", process.hdc());
        }
        Err(e) => println!("   ✗ Failed: {} (start notepad.exe to test)", e),
    }
    process.cleanup()?;

    // Example 5: Custom flags (fine-grained control)
    println!("\n5. Custom initialization flags:");
    println!("   Use case: Skip specific resources based on your needs");
    let custom_flags = InitFlags::new()
        .with_pid(true)
        .with_hwnd(true)
        .with_handle(false) // Skip process handle
        .with_dc(false); // Skip device context

    let config = ProcessConfig::builder("notepad.exe")
        .init_flags(custom_flags)
        .build();
    let mut process = Process::new(config);

    match process.init() {
        Ok(_) => {
            println!("   ✓ PID: {:?}", process.pid());
            println!("   ✓ HWND: {:?}", process.hwnd());
            println!("   ✗ HANDLE: {:?} (disabled by flags)", process.handle());
            println!("   ✗ HDC: {:?} (disabled by flags)", process.hdc());
        }
        Err(e) => println!("   ✗ Failed: {} (start notepad.exe to test)", e),
    }
    process.cleanup()?;

    println!("\n=== Summary ===");
    println!("Use InitFlags to control resource initialization:");
    println!("  - InitFlags::new()       : All resources (default)");
    println!("  - InitFlags::minimal()   : Only PID (process identification)");
    println!("  - InitFlags::memory_only(): PID + HANDLE (for memory operations)");
    println!("  - InitFlags::gui_only()  : PID + HWND + HDC (for screen capture)");
    println!("  - Custom flags           : Fine-grained control with builder pattern");
    Ok(())
}
