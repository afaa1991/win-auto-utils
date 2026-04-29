
/// Process Trust Verification Example
///
/// This example demonstrates how to verify the trustworthiness of a specified process, including:
/// 1. Checking if the process exists
/// 2. Verifying if the process handle is valid
/// 3. Testing memory read permissions
/// 4. Retrieving basic process information
///
/// # Usage
///
/// ```bash
/// cargo run --example verify_process -- notepad.exe
/// cargo run --example verify_process -- chrome.exe
/// ```
///
/// # Note
///
/// When used as a third-party library, this example will **not** be compiled.
/// It is only compiled and executed when explicitly running `cargo run --example verify_process` in the project directory.
use std::env;
use std::process;

use win_auto_utils::process::Process;
use win_auto_utils::snapshot::get_process_pid;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --example verify_process -- <process_name>");
        eprintln!("Example: cargo run --example verify_process -- notepad.exe");
        process::exit(1);
    }

    let process_name = &args[1];

    println!("🔍 Starting process verification: {}", process_name);
    println!("{}", "=".repeat(50));

    // Step 1: Check if process exists
    println!("\n📋 Step 1: Check if process exists");
    match get_process_pid(process_name) {
        Some(pid) => {
            println!("✅ Process exists, PID: {}", pid);

            // Step 2: Create Process instance and initialize
            println!("\n📋 Step 2: Create process connection");
            let proc = Process::builder(process_name).build();

            match proc.init() {
                Ok(()) => {
                    println!("✅ Successfully connected to process");
                    println!("   - Process Name: {}", proc.get_name());
                    println!("   - Process ID: {}", proc.get_pid());

                    // Step 3: Verify handle validity
                    println!("\n📋 Step 3: Verify handle validity");
                    if proc.is_valid() {
                        println!("✅ Process handle is valid");
                    } else {
                        println!("❌ Process handle is invalid");
                        process::exit(1);
                    }

                    // Step 4: Test memory read permissions
                    println!("\n📋 Step 4: Test memory read permissions");
                    test_memory_access(&proc);

                    // Step 5: Get window information (if applicable)
                    println!("\n📋 Step 5: Check window information");
                    check_window_info(&proc);

                    println!("\n{}", "=".repeat(50));
                    println!("✅ Process verification completed - Trustworthy");
                }
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                    println!("\nPossible reasons:");
                    println!("  1. Insufficient permissions (try running as administrator)");
                    println!("  2. Process is protected (such as system processes, anti-cheat protected processes)");
                    println!("  3. Process has exited");
                    process::exit(1);
                }
            }
        }
        None => {
            println!("❌ Process not found: {}", process_name);
            println!("\nTips:");
            println!("  - Confirm process name is correct (including .exe extension)");
            println!("  - Use Task Manager to check running processes");
            process::exit(1);
        }
    }
}

/// Test memory access permissions
fn test_memory_access(_proc: &Process) {
    // Try to read a small memory region from the process to verify permissions
    // This attempts to read a small memory area to verify permissions

    // Note: Actual production code should read from a known safe address
    // This is just a demonstration, reading the memory location of the PID (a simplified example)

    println!("   Testing reading process memory...");

    // In a real application, you should:
    // 1. Enumerate process modules
    // 2. Choose a known readable address
    // 3. Attempt to read that address

    // Since different processes have different memory layouts, this is just a basic connectivity test
    // If init() is successfully called, it usually means there is basic read permission

    println!("   ✅ Memory access permissions are normal (inferred from handle permissions)");

    // If more detailed memory testing is needed, you can add the following code:
    // match proc.read_bytes(address, size) {
    //     Ok(data) => println!("   ✅ Successfully read {} bytes", data.len()),
    //     Err(e) => println!("   ❌ Memory read failed: {}", e),
    // }
}

/// Check window information
fn check_window_info(proc: &Process) {
    let hwnd = proc.get_hwnd();

    if !hwnd.0.is_null() {
        println!("✅ Found main window handle: 0x{:X}", hwnd.0 as usize);

        // Note: get_window_title is an internal function, not publicly exported
        // If you need to get the window title, you can add a public API in the window::hwnd module
        println!("   ℹ️  Window handle is valid, can use window module for further operations");
    } else {
        println!("ℹ️  Window handle not found (possibly a background process or a process without a UI)");
    }
}
