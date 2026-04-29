//! Memory Hook Reset Example
//!
//! Demonstrates how to use the `reset()` method to safely clean up hook state
//! after a target program restarts. This ensures no side effects when reusing
//! hook objects across multiple program sessions.
//!
//! # Key Concepts
//! - **Problem**: After program restart, cached addresses and bytes become invalid
//! - **Solution**: Call `reset()` to clear all internal state before reuse
//! - **Benefit**: Zero side effects, safe to reuse hook objects

use win_auto_utils::memory_hook::{TrampolineHook, InlineHook, BytesSwitch, MemoryLock};
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_ALL_ACCESS, PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Hook Reset Demo ===\n");

    // Note: Replace with actual PID and addresses for your target program
    let pid = 12345; // Example PID
    let target_address = 0x0041FAF2; // Example address
    
    println!("Step 1: First session - using hooks normally");
    first_session(pid, target_address)?;
    
    println!("\nStep 2: Program restarted - simulating cleanup");
    println!("(In real scenario, you would call reset() here)");
    
    println!("\nStep 3: Second session - reusing hooks after reset");
    second_session(pid, target_address)?;
    
    println!("\n✓ All sessions completed successfully!");
    Ok(())
}

/// Simulates the first usage session
fn first_session(pid: u32, target_address: usize) -> Result<(), Box<dyn std::error::Error>> {
    let handle = open_process_handle(pid, PROCESS_ALL_ACCESS)
        .ok_or("Failed to open process")?;
    
    // --- TrampolineHook Example ---
    println!("  [TrampolineHook] Installing hook...");
    let shellcode = vec![
        0x01, 0xD2,                              // add edx,edx
        0x01, 0x91, 0x08, 0x03, 0x00, 0x00,     // add [ecx+0x308],edx
    ];
    
    let mut tramp_hook = TrampolineHook::new_x86(handle, target_address, shellcode.clone());
    
    match tramp_hook.install() {
        Ok(tramp_addr) => {
            println!("    ✓ Installed (trampoline at 0x{:X})", tramp_addr);
            
            // Use the hook...
            println!("    ✓ Hook is working");
            
            // Uninstall when done
            tramp_hook.uninstall()?;
            println!("    ✓ Uninstalled");
        }
        Err(e) => {
            println!("    ⚠ Install failed (expected if address invalid): {}", e);
        }
    }
    
    // --- InlineHook Example ---
    println!("  [InlineHook] Installing hook...");
    let detour_address = 0x00500000; // Example detour address
    let mut inline_hook = InlineHook::new_x86(handle, target_address, detour_address);
    
    match inline_hook.install() {
        Ok(_) => {
            println!("    ✓ Installed");
            inline_hook.uninstall()?;
            println!("    ✓ Uninstalled");
        }
        Err(e) => {
            println!("    ⚠ Install failed: {}", e);
        }
    }
    
    // --- BytesSwitch Example ---
    println!("  [BytesSwitch] Creating switch...");
    match BytesSwitch::new_nop(handle, target_address, 6) {
        Ok(mut switch) => {
            switch.enable()?;
            println!("    ✓ Enabled (NOP patch applied)");
            
            switch.disable()?;
            println!("    ✓ Disabled (original restored)");
        }
        Err(e) => {
            println!("    ⚠ Create failed: {}", e);
        }
    }
    
    // --- MemoryLock Example ---
    println!("  [MemoryLock] Creating lock...");
    let lock_handle = open_process_handle(pid, PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION)
        .ok_or("Failed to open process")?;
    
    match MemoryLock::builder()
        .handle(lock_handle)
        .address(target_address)
        .value(500u32)
        .scan_interval_ms(10)
        .build()
    {
        Ok(mut lock) => {
            lock.lock_value(500u32)?;
            println!("    ✓ Started monitoring");
            
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            lock.unlock()?;
            println!("    ✓ Stopped monitoring");
        }
        Err(e) => {
            println!("    ⚠ Create failed: {}", e);
        }
    }
    
    Ok(())
}

/// Simulates the second usage session after program restart
fn second_session(pid: u32, target_address: usize) -> Result<(), Box<dyn std::error::Error>> {
    let handle = open_process_handle(pid, PROCESS_ALL_ACCESS)
        .ok_or("Failed to open process")?;
    
    // --- TrampolineHook with reset() ---
    println!("  [TrampolineHook] Reusing hook object after reset...");
    let shellcode = vec![
        0x01, 0xD2,
        0x01, 0x91, 0x08, 0x03, 0x00, 0x00,
    ];
    
    // Create new hook (in real scenario, you'd reuse the old one with reset())
    let mut tramp_hook = TrampolineHook::new_x86(handle, target_address, shellcode.clone());
    
    // If reusing: tramp_hook.reset();
    
    match tramp_hook.install() {
        Ok(tramp_addr) => {
            println!("    ✓ Re-installed successfully (trampoline at 0x{:X})", tramp_addr);
            tramp_hook.uninstall()?;
            println!("    ✓ Uninstalled");
        }
        Err(e) => {
            println!("    ⚠ Install failed: {}", e);
        }
    }
    
    // --- Demonstrate reset() explicitly ---
    println!("\n  [Demo] Explicitly calling reset() on hooks:");
    
    let mut hook1 = TrampolineHook::new_x86(handle, target_address, vec![0x90]);
    println!("    Created TrampolineHook");
    
    // Simulate some state
    hook1.original_bytes = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    hook1.detour_address = 0x12345678;
    hook1.trampoline_address = Some(0x87654321);
    println!("    Set fake state (bytes={}, detour=0x{:X}, tramp=0x{:X})", 
             hook1.original_bytes.len(),
             hook1.detour_address,
             hook1.trampoline_address.unwrap());
    
    // Reset!
    hook1.reset();
    println!("    Called reset()");
    println!("    ✓ State cleared (bytes={}, detour=0x{:X}, tramp={:?})", 
             hook1.original_bytes.len(),
             hook1.detour_address,
             hook1.trampoline_address);
    
    let mut hook2 = InlineHook::new_x86(handle, target_address, 0x500000);
    println!("\n    Created InlineHook with fake state");
    
    hook2.reset();
    println!("    Called reset()");
    println!("    ✓ State cleared (is_installed={})", hook2.is_installed());

    let mut switch = BytesSwitch::new_nop(handle, target_address, 6)?;
    switch.enable()?;
    println!("\n    Created BytesSwitch and enabled it");
    
    switch.reset();
    println!("    Called reset()");
    println!("    ✓ State cleared (enabled={}, bytes={})", 
             switch.is_enabled(),
             switch.original_bytes().len());
    
    Ok(())
}
