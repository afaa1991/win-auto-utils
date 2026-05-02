//! Memory Hook Module
//!
//! Provides memory manipulation capabilities including:
//! - Inline hooking (redirect function calls)
//! - Trampoline hooking (preserve original function while hooking)
//! - Bytes switching (quick enable/disable of byte sequences)
//! - Register extraction (capture CPU register values at hook points)
//!
//! # Feature Flag
//! Enable with: `--features "memory_hook"`
//!
//! # Quick Start
//!
//! ## Inline Hook Example
//! ```no_run
//! use win_auto_utils::memory_hook::InlineHook;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_ALL_ACCESS};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let target_func = 0x7FF6A1B2C3D4;
//!     let detour_func = 0x7FF6A1B2D4E5;
//!     
//!     let handle = open_process_handle(pid, PROCESS_ALL_ACCESS)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Install inline hook (defaults to x64)
//!     let mut hook = InlineHook::new(handle, target_func, detour_func);
//!     hook.install()?;
//!     
//!     // Now calls to target_func will jump to detour_func
//!     // ... your code here ...
//!     
//!     // Remove hook
//!     hook.uninstall()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Trampoline Hook Example
//! ```no_run
//! use win_auto_utils::memory_hook::TrampolineHook;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_ALL_ACCESS};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let target_addr = 0x0041FAF2;
//!     
//!     let handle = open_process_handle(pid, PROCESS_ALL_ACCESS)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Define custom shellcode
//!     let shellcode = vec![
//!         0x01, 0xD2,                              // add edx,edx
//!         0x01, 0x91, 0x08, 0x03, 0x00, 0x00,     // add [ecx+0x308],edx
//!     ];
//!     
//!     // Create trampoline hook for x86 process
//!     let mut hook = TrampolineHook::new_x86(handle, target_addr, shellcode);
//!     hook.install()?;
//!     
//!     // Use the hook...
//!     
//!     // Remove hook (auto-cleanup)
//!     hook.uninstall()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Bytes Switch Example
//! ```no_run
//! use win_auto_utils::memory_hook::BytesSwitch;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_ALL_ACCESS};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let target_addr = 0x7FF6A1B2C3D4;
//!     
//!     let handle = open_process_handle(pid, PROCESS_ALL_ACCESS)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Create a NOP switch (disable functionality)
//!     let mut switch = BytesSwitch::new_nop(handle, target_addr, 6)?;
//!     
//!     // Enable switch (write NOPs)
//!     switch.enable()?;
//!     
//!     // ... functionality is now disabled ...
//!     
//!     // Disable switch (restore original bytes)
//!     switch.disable()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Register Extractor Example
//! ```no_run
//! use win_auto_utils::memory_hook::register_extractor::{RegisterExtractor, Register};
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_ALL_ACCESS};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let target_addr = 0x1C17E514F90;
//!     
//!     let handle = open_process_handle(pid, PROCESS_ALL_ACCESS)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Create register extractor to capture RDI at hook point
//!     let mut extractor = RegisterExtractor::builder()
//!         .handle(handle)
//!         .target_address(target_addr)
//!         .bytes_to_overwrite(15)
//!         .extract_register(Register::RDI)
//!         .x64()
//!         .build()?;
//!     
//!     extractor.install()?;
//!     
//!     // Read captured register value
//!     let player_ptr: u64 = extractor.read_register::<u64>(Register::RDI)?;
//!     
//!     // Read through pointer chain
//!     let health: f32 = extractor.read_chain::<f32>(Register::RDI, &[0x34])?;
//!     
//!     extractor.uninstall()?;
//!     
//!     Ok(())
//! }
//! ```

mod bytes_switch;
mod inline_hook;
mod shellcode;
mod trampoline_hook;
mod utils;

#[cfg(feature = "memory_register_extractor")]
pub mod register_extractor;

pub use bytes_switch::BytesSwitch;
pub use inline_hook::InlineHook;
pub use shellcode::{Architecture, ShellcodeBuilder};
pub use trampoline_hook::TrampolineHook;
pub use utils::{MemoryProtector, ProtectionGuard};

#[cfg(feature = "memory_register_extractor")]
pub use register_extractor::{Register, RegisterExtractor};
