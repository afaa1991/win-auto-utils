//! DLL Injection Example
//!
//! This example demonstrates how to inject a DLL into a target process using the `dll_injector` module.
//! It supports automatic process lookup, injection verification, and optional function calling.
//!
//! # Required Features
//! - `dll_injector`: Core DLL injection functionality
//! - `snapshot` (optional): For automatic process name lookup
//!
//! # Usage
//! ```bash
//! # Interactive mode (no arguments)
//! cargo run --example dll_injection --features "dll_injector snapshot"
//!
//! # Command-line mode
//! cargo run --example dll_injection --features "dll_injector" -- 12345 "C:\path\to\dll.dll"
//!
//! # Compile as x86 for injecting into 32-bit processes
//! rustup target add i686-pc-windows-msvc
//! cargo run --example dll_injection --target i686-pc-windows-msvc --features "dll_injector snapshot"
//! ```
//!
//! # Safety Warning
//! ⚠️  This program performs real DLL injection into running processes.
//! - Run as Administrator (required for process access)
//! - Only test on processes you own (e.g., notepad.exe)
//! - Ensure DLL architecture matches target process (x86 DLL → x86 process)
//!
//! # User Code Area
//! You can modify the `main()` function below to customize:
//! - Target process selection
//! - DLL path configuration
//! - Post-injection actions (e.g., call exported functions)

use std::env;
use std::io::{self, stdin, BufRead, Write};
use std::process;

#[cfg(feature = "dll_injector")]
use win_auto_utils::dll_injector::{
    diagnose_injection, inject_dll, unload_dll,
    get_exported_function_address, call_function_with_raw_bytes, call_function_no_params
};

#[cfg(feature = "snapshot")]
use win_auto_utils::snapshot::list_processes;

/// Check if a module is loaded in the target process
/// This is a local implementation since the internal helper is not public
#[cfg(feature = "snapshot")]
fn is_module_loaded(pid: u32, module_name: &str) -> bool {
    win_auto_utils::snapshot::get_module_base_address(pid, module_name).is_some()
}

/// Fallback when snapshot feature is not enabled
#[cfg(not(feature = "snapshot"))]
fn is_module_loaded(_pid: u32, _module_name: &str) -> bool {
    eprintln!("⚠️  Module detection requires 'snapshot' feature");
    false
}

/// Categorized process list for better user experience
#[derive(Debug, Default)]
struct CategorizedProcesses {
    /// Processes from current directory executables
    local_exes: Vec<(usize, u32, String)>, // (display_number, pid, name)
    /// Other processes
    others: Vec<(usize, u32, String)>,
    /// All processes in display order
    all_ordered: Vec<(usize, u32, String)>,
}

impl CategorizedProcesses {
    /// Check if categorized list is empty
    fn is_empty(&self) -> bool {
        self.all_ordered.is_empty()
    }

    /// Get total count of displayed processes
    fn total_count(&self) -> usize {
        self.all_ordered.len()
    }

    /// Get PID by display index (1-based)
    fn get_pid_by_index(&self, index: usize) -> Option<u32> {
        self.all_ordered
            .iter()
            .find(|(num, _, _)| *num == index)
            .map(|(_, pid, _)| *pid)
    }

    /// Get process name by display index (1-based)
    fn get_name_by_index(&self, index: usize) -> Option<String> {
        self.all_ordered
            .iter()
            .find(|(num, _, _)| *num == index)
            .map(|(_, _, name)| name.clone())
    }

    /// Fuzzy search by process name (case-insensitive)
    fn fuzzy_search(&self, keyword: &str) -> Option<(u32, String)> {
        let keyword_lower = keyword.to_lowercase();
        self.all_ordered
            .iter()
            .find(|(_, _, name)| name.to_lowercase().contains(&keyword_lower))
            .map(|(_, pid, name)| (*pid, name.clone()))
    }

    /// Display categorized processes with formatting
    fn display_with_numbers(&self) {
        let max_display = 50; // Limit total display to avoid overwhelming

        // Show local executables first
        if !self.local_exes.is_empty() {
            println!(
                "\n📁 Current Directory Executables ({}):",
                self.local_exes.len()
            );
            let local_to_show = std::cmp::min(self.local_exes.len(), max_display);
            for &(num, pid, ref name) in self.local_exes.iter().take(local_to_show) {
                println!("   [{}] {:6} - {}", num, pid, name);
            }
        }

        // Calculate how many others we can show
        let shown_count = std::cmp::min(self.local_exes.len(), max_display);
        let remaining = max_display.saturating_sub(shown_count);

        // Show other processes
        if !self.others.is_empty() && remaining > 0 {
            if !self.local_exes.is_empty() {
                println!("\n🔍 Other Processes:");
            } else {
                println!("\n🖥️  Running Processes:");
            }

            let to_show = std::cmp::min(self.others.len(), remaining);

            for &(num, pid, ref name) in self.others.iter().take(to_show) {
                println!("   [{}] {:6} - {}", num, pid, name);
            }

            if self.others.len() > to_show {
                println!("   ... and {} more processes", self.others.len() - to_show);
            }
        }

        println!("\n💡 Selection options:");
        println!("   • Enter number (e.g., \"1\") to select by index");
        println!("   • Enter process name (e.g., \"game\") for fuzzy search");
        println!("   • Enter PID directly (e.g., \"12345\")");
    }
}

/// Configuration for DLL injection
struct InjectionConfig {
    /// Target process ID
    pid: u32,
    /// Full path to the DLL file
    dll_path: String,
    /// Module name (filename without path)
    module_name: String,
    /// Optional: Function to call after injection
    function_name: Option<String>,
    /// Optional: Parameter for the function (f32)
    function_param: Option<f32>,
}

impl InjectionConfig {
    /// Create configuration from command-line arguments or interactive mode
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();

        // Parse command-line arguments: <pid> <dll_path> [function_name] [param]
        if args.len() >= 3 {
            let pid = args[1].parse::<u32>().unwrap_or_else(|_| {
                eprintln!("❌ Invalid PID: {}", args[1]);
                process::exit(1);
            });

            let dll_path = args[2].clone();
            let module_name = std::path::Path::new(&dll_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.dll")
                .to_string();

            let function_name = args.get(3).cloned();
            let function_param = args.get(4).and_then(|s| s.parse::<f32>().ok());

            Self {
                pid,
                dll_path,
                module_name,
                function_name,
                function_param,
            }
        } else {
            // Interactive mode
            Self::interactive_mode()
        }
    }

    /// Interactive mode: list DLLs and processes, let user choose
    fn interactive_mode() -> Self {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║         DLL Injector - Interactive Mode                  ║");
        println!("╚═══════════════════════════════════════════════════════════╝\n");

        let stdin = io::stdin();

        // Step 1: List DLL files in current directory
        println!("📁 DLL Files in Current Directory:");
        println!("─────────────────────────────────────────");

        let dll_files = Self::list_dll_files();
        if dll_files.is_empty() {
            println!("   ⚠️  No DLL files found in current directory");
        } else {
            for (i, dll) in dll_files.iter().enumerate() {
                println!("   [{}] {}", i + 1, dll);
            }
        }
        println!();

        // Step 2: Let user select DLL
        let dll_path = loop {
            print!("Enter DLL file number or full path: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.is_empty() {
                println!("❌ Input cannot be empty");
                continue;
            }

            // Try to parse as number
            if let Ok(num) = input.parse::<usize>() {
                if num > 0 && num <= dll_files.len() {
                    break dll_files[num - 1].clone();
                } else {
                    println!("❌ Invalid number. Please enter 1-{}", dll_files.len());
                    continue;
                }
            }

            // Treat as path
            let path = if std::path::Path::new(input).is_absolute() {
                input.to_string()
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(input)
                    .to_string_lossy()
                    .to_string()
            };

            if !std::path::Path::new(&path).exists() {
                println!("❌ File not found: {}", path);
                continue;
            }

            break path;
        };

        let module_name = std::path::Path::new(&dll_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.dll")
            .to_string();

        println!("✅ Selected DLL: {}\n", module_name);

        // Step 3: List running processes with smart categorization
        println!("🖥️  Running Processes:");
        println!("═════════════════════════════════════════════");

        #[cfg(feature = "snapshot")]
        let categorized = Self::categorize_processes();

        #[cfg(not(feature = "snapshot"))]
        let categorized = CategorizedProcesses::default();

        if categorized.is_empty() {
            println!("   ⚠️  Could not retrieve process list");
            println!("   💡 Please enter PID manually\n");
        } else {
            categorized.display_with_numbers();
            println!();
        }

        // Step 4: Let user select process with multiple input methods
        let pid = loop {
            print!("Select process (number/name/PID): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.is_empty() {
                println!("❌ Input cannot be empty");
                continue;
            }

            // Method 1: Try to parse as number (list index)
            if let Ok(index) = input.parse::<usize>() {
                #[cfg(feature = "snapshot")]
                if let Some(pid) = categorized.get_pid_by_index(index) {
                    println!(
                        "✅ Selected: {} (PID: {})",
                        categorized.get_name_by_index(index).unwrap(),
                        pid
                    );
                    break pid;
                }

                println!(
                    "❌ Invalid selection number. Available: 1-{}",
                    categorized.total_count()
                );
                continue;
            }

            // Method 2: Try to parse as PID directly
            if let Ok(pid) = input.parse::<u32>() {
                println!("✅ Using PID: {}", pid);
                break pid;
            }

            // Method 3: Fuzzy match by process name
            #[cfg(feature = "snapshot")]
            if let Some((pid, name)) = categorized.fuzzy_search(input) {
                println!("✅ Found: {} (PID: {})", name, pid);
                break pid;
            }

            println!("❌ Not found. Try:");
            println!("   • Number (1-{})", categorized.total_count());
            println!("   • Process name (e.g., 'game', 'chrome')");
            println!("   • PID directly");
        };

        println!("✅ Target PID: {}\n", pid);

        // Step 5: Optional function call
        println!("💡 Optional: Call exported function after injection?");
        print!("Enter function name (or press Enter to skip): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        let function_name = if input.trim().is_empty() {
            None
        } else {
            Some(input.trim().to_string())
        };

        let function_param = if function_name.is_some() {
            print!("Enter parameter (f32, or press Enter for none): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();
            input.trim().parse::<f32>().ok()
        } else {
            None
        };

        Self {
            pid,
            dll_path,
            module_name,
            function_name,
            function_param,
        }
    }

    /// List DLL files in current directory
    fn list_dll_files() -> Vec<String> {
        let mut dll_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "dll" {
                        if let Some(name) = entry.file_name().to_str() {
                            dll_files.push(name.to_string());
                        }
                    }
                }
            }
        }

        dll_files.sort();
        dll_files
    }

    /// Get executable files (.exe) in the current directory
    fn get_local_executables() -> Vec<String> {
        let mut exe_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "exe" {
                        if let Some(name) = entry.file_name().to_str() {
                            exe_files.push(name.to_string());
                        }
                    }
                }
            }
        }

        exe_files.sort();
        exe_files
    }

    /// Categorize processes for better user experience
    #[cfg(feature = "snapshot")]
    fn categorize_processes() -> CategorizedProcesses {
        // Get all running processes
        let all_processes = list_processes();

        // Get executable files in current directory
        let local_exes: Vec<String> = Self::get_local_executables();
        let has_local = !local_exes.is_empty();

        let mut categorized = CategorizedProcesses::default();

        if has_local {
            // Separate processes into local and others
            let mut local_list: Vec<(u32, String)> = Vec::new();
            let mut other_list: Vec<(u32, String)> = Vec::new();

            for (pid, name) in &all_processes {
                let name_lower = name.to_lowercase();
                let is_local = local_exes.iter().any(|exe| {
                    let exe_lower = exe.to_lowercase();
                    name_lower == exe_lower || name_lower.ends_with(&exe_lower)
                });

                if is_local {
                    local_list.push((*pid, name.clone()));
                } else {
                    other_list.push((*pid, name.clone()));
                }
            }

            // Sort both lists alphabetically
            local_list.sort_by_key(|(_, name)| name.clone());
            other_list.sort_by_key(|(_, name)| name.clone());

            // Assign display numbers and populate categorized structure
            let mut counter = 1;

            for (pid, name) in &local_list {
                categorized.local_exes.push((counter, *pid, name.clone()));
                categorized.all_ordered.push((counter, *pid, name.clone()));
                counter += 1;
            }

            for (pid, name) in &other_list {
                categorized.others.push((counter, *pid, name.clone()));
                categorized.all_ordered.push((counter, *pid, name.clone()));
                counter += 1;
            }
        } else {
            // No local executables, just sort all processes
            let mut sorted = all_processes;
            sorted.sort_by_key(|(_, name)| name.clone());

            for (i, (pid, name)) in sorted.iter().enumerate() {
                let num = i + 1;
                categorized.others.push((num, *pid, name.clone()));
                categorized.all_ordered.push((num, *pid, name.clone()));
            }
        }

        categorized
    }
}

fn main() {
    println!("=== DLL Injection Example ===\n");

    // Step 1: Load configuration
    let config = InjectionConfig::from_args();

    println!("📋 Injection Configuration:");
    println!("   Target PID: {}", config.pid);
    println!("   DLL Path: {}", config.dll_path);
    println!("   Module Name: {}", config.module_name);
    if let Some(ref func) = config.function_name {
        println!("   Function to Call: {}", func);
        if let Some(param) = config.function_param {
            println!("   Function Parameter: {}", param);
        }
    }
    println!();

    // Step 2: Validate DLL file exists
    if !std::path::Path::new(&config.dll_path).exists() {
        eprintln!("❌ DLL file not found at: {}", config.dll_path);
        eprintln!("💡 Tip: Place your DLL at this location or update the path in code");
        eprintln!("   Example paths:");
        eprintln!("   - C:\\temp\\game_mod_x86.dll");
        eprintln!("   - D:\\Games\\GameFolder\\version.dll");
        process::exit(1);
    }
    println!("✅ DLL file exists");

    // Step 3: Check if module is already loaded
    if is_module_loaded(config.pid, &config.module_name) {
        println!(
            "⚠️  Module '{}' is already loaded in process {}",
            config.module_name, config.pid
        );
        println!("💡 Options:");
        println!("   1. Skip injection (module already present)");
        println!("   2. Unload first (may cause instability for proxy DLLs)");

        // Ask user for action (simple y/n)
        print!("   Unload first? (y/n): ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input).ok();

        if input.trim().to_lowercase() == "y" {
            println!("🔄 Attempting to unload...");
            match unload_dll(config.pid, &config.module_name) {
                Ok(()) => println!("✅ Module unloaded"),
                Err(e) => {
                    eprintln!("❌ Failed to unload: {}", e);
                    eprintln!("⚠️  Proxy DLLs cannot be safely unloaded");
                    eprintln!("💡 Consider using configuration switches instead");
                    process::exit(1);
                }
            }
        } else {
            println!("⏭️  Skipping unload, proceeding with existing module");
        }
    } else {
        println!(
            "✅ Module '{}' is not loaded (ready for injection)",
            config.module_name
        );
    }

    // Step 4: Diagnose injection feasibility (optional but recommended)
    println!("\n🔍 Running pre-injection diagnostics...");
    match diagnose_injection(config.pid) {
        Ok(report) => {
            println!("{}", report.summary());

            if !report.is_injection_feasible() {
                eprintln!("\n⚠️  Injection is likely to fail based on diagnostics.");
                eprintln!("💡 Recommendations:");
                eprintln!(
                    "   1. Run this program as Administrator (Right-click → Run as administrator)"
                );
                eprintln!("   2. Verify architecture match:");
                eprintln!("      - Use dll_injector_x86.exe for 32-bit processes");
                eprintln!("      - Use dll_injector_x64.exe for 64-bit processes");
                eprintln!("   3. Try injecting into a simple process first (e.g., notepad.exe)");
                eprintln!();

                print!("Continue anyway? (y/N): ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                stdin().lock().read_line(&mut input).unwrap();

                if input.trim().to_lowercase() != "y" {
                    println!("❌ Injection cancelled by user");
                    process::exit(0);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  Diagnostic check failed: {}", e);
            eprintln!("💡 Continuing with injection attempt...\n");
        }
    }

    // Step 5: Inject the DLL
    println!("\n🚀 Injecting DLL into process {}...", config.pid);
    let start_time = std::time::Instant::now();

    match inject_dll(config.pid, &config.dll_path) {
        Ok(()) => {
            let elapsed = start_time.elapsed();
            println!("✅ DLL injected successfully (took {:?})", elapsed);
        }
        Err(e) => {
            eprintln!("❌ DLL injection failed: {}", e);

            // Provide context-aware troubleshooting
            eprintln!("\n🔍 Troubleshooting based on error:");

            if e.to_string().contains("WriteProcessMemory") || e.to_string().contains("write") {
                eprintln!("   💡 Write failure detected. Common causes:");
                eprintln!("      • Not running as Administrator");
                eprintln!("      • Target process has anti-cheat/anti-injection protection");
                eprintln!("      • Architecture mismatch (x86 injector → x64 target)");
                eprintln!("   🔧 Try:");
                eprintln!("      1. Right-click → 'Run as administrator'");
                eprintln!("      2. Use correct architecture version (x86 vs x64)");
                eprintln!("      3. Test with notepad.exe first");
            } else if e.to_string().contains("OpenProcess") {
                eprintln!("   💡 Cannot open process. Solutions:");
                eprintln!("      • Run as Administrator");
                eprintln!("      • Target process may be protected");
            } else if e.to_string().contains("Architecture") {
                eprintln!("   💡 Architecture mismatch detected!");
                eprintln!("      • x86 DLL must target x86 process");
                eprintln!("      • x64 DLL must target x64 process");
            } else {
                eprintln!("   1. Ensure you're running as Administrator");
                eprintln!("   2. Verify DLL architecture matches target process:");
                eprintln!(
                    "      - x86 DLL → x86 process (compile with --target i686-pc-windows-msvc)"
                );
                eprintln!("      - x64 DLL → x64 process");
                eprintln!("   3. Check if target process has anti-injection protection");
                eprintln!("   4. Verify DLL dependencies are available (use Dependency Walker)");
                eprintln!("   5. Try injecting into a simple process first (e.g., notepad.exe)");
            }

            process::exit(1);
        }
    }

    // Step 6: Verify injection
    println!("\n🔍 Verifying injection...");
    std::thread::sleep(std::time::Duration::from_millis(500)); // Give time for DllMain to execute

    if is_module_loaded(config.pid, &config.module_name) {
        println!(
            "✅ Verification successful: Module '{}' is now loaded",
            config.module_name
        );
    } else {
        eprintln!("❌ Verification failed: Module not found after injection");
        eprintln!("💡 Possible causes:");
        eprintln!("   - DllMain returned FALSE (initialization failed)");
        eprintln!("   - Anti-injection protection blocked loading");
        eprintln!("   - Missing dependencies prevented loading");
        eprintln!("   - Debug version called AllocConsole (crashes protected processes)");
        eprintln!("\n🔧 Diagnostic steps:");
        eprintln!("   1. Check Windows Event Viewer for application errors");
        eprintln!("   2. Use Process Monitor (ProcMon) to trace DLL loading");
        eprintln!("   3. Try injecting into notepad.exe to isolate the issue");
        process::exit(1);
    }

    // Success summary
    println!("\n🎉 DLL injection completed successfully!");
    
    // Step 7: Optional - Call exported function
    if let Some(ref func_name) = config.function_name {
        println!("\n📞 Calling exported function '{}'...", func_name);
        
        match get_exported_function_address(config.pid, &config.module_name, func_name) {
            Ok(func_addr) => {
                println!("   ✅ Function address: 0x{:X}", func_addr);
                
                if let Some(param) = config.function_param {
                    // Call with parameter
                    println!("   📤 Passing parameter: {}", param);
                    let param_bytes = param.to_ne_bytes();
                    
                    match call_function_with_raw_bytes(config.pid, func_addr, Some(&param_bytes)) {
                        Ok(result) => println!("   ✅ Function returned: {}", result),
                        Err(e) => eprintln!("   ❌ Function call failed: {}", e),
                    }
                } else {
                    // Call without parameter
                    match call_function_no_params(config.pid, func_addr) {
                        Ok(result) => println!("   ✅ Function returned: {}", result),
                        Err(e) => eprintln!("   ❌ Function call failed: {}", e),
                    }
                }
            }
            Err(e) => {
                eprintln!("   ❌ Failed to get function address: {}", e);
                eprintln!("   💡 Possible causes:");
                eprintln!("      - Function name is incorrect (check spelling)");
                eprintln!("      - Function is not exported (missing __declspec(dllexport))");
                eprintln!("      - Module not fully loaded yet");
            }
        }
    }
    
    println!("\n📝 Next steps:");
    println!("   - Observe target process for expected behavior");
    println!("   - Use Process Explorer to verify module in memory");
    println!(
        "   - To unload (if safe): call unload_dll({}, \"{}\")",
        config.pid, config.module_name
    );
    println!("   - For proxy DLLs, prefer configuration switches over unloading");

    println!("\n⚠️  Note: The DLL will remain loaded until the process exits.");
}
