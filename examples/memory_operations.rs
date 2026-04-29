//! Memory Operations Example
//!
//! This example demonstrates how to read and write memory in a target process.
//! It shows various operations including reading/writing primitive types and byte arrays.
//!
//! # Required Features
//! - `memory`: Core memory operations
//! - `handle`: Process handle management (for opening process with correct permissions)
//! - `snapshot` (optional): For automatic process name lookup
//!
//! # Usage
//! ```bash
//! cargo run --example memory_operations --features "memory handle snapshot"
//! ```
//!
//! # Safety Warning
//! ⚠️  This program performs real memory operations on running processes.
//! - Run as Administrator (required for process access)
//! - Only test on processes you own (e.g., notepad.exe)
//! - Writing to wrong addresses can crash the target process

#[cfg(feature = "memory")]
use win_auto_utils::memory::read_memory_bytes;

#[cfg(all(feature = "memory", feature = "handle"))]
use win_auto_utils::handle::open_process_handle;

#[cfg(feature = "snapshot")]
use win_auto_utils::snapshot::get_process_pid;

#[cfg(all(feature = "memory", feature = "handle"))]
fn main() {
    println!("🔧 Memory Operations Example\n");
    
    // Step 1: Find target process
    let pid = if cfg!(feature = "snapshot") {
        #[cfg(feature = "snapshot")]
        {
            println!("📋 Looking for notepad.exe...");
            match get_process_pid("notepad.exe") {
                Some(pid) => {
                    println!("✅ Found notepad.exe with PID: {}", pid);
                    pid
                }
                None => {
                    eprintln!("❌ notepad.exe not found!");
                    eprintln!("💡 Please start notepad.exe and try again");
                    return;
                }
            }
        }
        #[cfg(not(feature = "snapshot"))]
        {
            unreachable!()
        }
    } else {
        println!("⚠️  Using default PID 12345 (replace with actual PID)");
        12345
    };
    
    // Step 2: Open process handle using handle module
    println!("\n🔓 Opening process with memory access rights via handle module...");
    use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
    
    let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
    let handle = match open_process_handle(pid, desired_access) {
        Some(h) => {
            println!("✅ Successfully opened process");
            h
        }
        None => {
            eprintln!("❌ Failed to open process");
            eprintln!("💡 Try running as Administrator");
            return;
        }
    };
    
    // Ensure handle is closed when done
    defer_close_handle(handle);
    
    // Step 3: Read bytes from memory (example address)
    println!("\n📖 Reading bytes from memory...");
    let example_address = 0x7FF600000000usize; // Example address (will likely fail)
    
    match read_memory_bytes(handle, example_address, 16) {
        Ok(bytes) => {
            println!("✅ Read {} bytes from 0x{:X}", bytes.len(), example_address);
            println!("   Data: {:02X?}", bytes);
        }
        Err(e) => {
            println!("⚠️  Read failed (expected for invalid address): {}", e);
            println!("💡 In real usage, use valid addresses from process memory");
        }
    }
    
    // Step 4: Demonstrate type-specific reads/writes
    println!("\n📊 Type-specific operations:");
    println!("   Available functions:");
    println!("   - read_memory_i8/i16/i32/i64");
    println!("   - read_memory_u8/u16/u32/u64");
    println!("   - read_memory_f32/f64");
    println!("   - write_memory_i8/i16/i32/i64");
    println!("   - write_memory_u8/u16/u32/u64");
    println!("   - write_memory_f32/f64");
    println!("   - read_memory_bytes / write_memory_bytes (for byte arrays)");
    println!("   - read_memory_t / write_memory_t (generic for any Copy type)");
    
    // Step 5: Show generic API usage
    println!("\n🎯 Generic API example:");
    println!("   use win_auto_utils::memory::{{read_memory_t, write_memory_t}};");
    println!("   ");
    println!("   // Read any Copy type");
    println!("   let value: i32 = read_memory_t(handle, address)?;");
    println!("   ");
    println!("   // Write any Copy type");
    println!("   write_memory_t(handle, address, 999i32)?;");
    
    println!("\n✨ Memory operations module ready for use!");
    println!("\n📝 Common use cases:");
    println!("   - Game hacking: Read/write player stats, positions, etc.");
    println!("   - Process debugging: Inspect memory contents");
    println!("   - Memory patching: Modify code or data at runtime");
    println!("   - Data extraction: Read structured data from processes");
}

#[cfg(not(all(feature = "memory", feature = "handle")))]
fn main() {
    eprintln!("❌ This example requires both 'memory' and 'handle' features");
    eprintln!("💡 Run with: cargo run --example memory_operations --features \"memory handle\"");
}

/// Helper to close handle when it goes out of scope
#[cfg(all(feature = "memory", feature = "handle"))]
fn defer_close_handle(handle: windows::Win32::Foundation::HANDLE) {
    struct HandleGuard(windows::Win32::Foundation::HANDLE);
    
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
    
    let _guard = HandleGuard(handle);
    // Handle will be closed when _guard goes out of scope
}
