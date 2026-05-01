//! Memory Lock Module
//!
//! Provides functionality to continuously monitor and restore memory values.
//! This is useful for creating "freeze" effects in game trainers or maintaining
//! constant values in target processes.
//!
//! # Feature Flag
//! Enable with: `--features "memory_lock"`
//!
//! # Quick Start
//!
//! ## Basic Usage
//! ```no_run
//! use win_auto_utils::memory_lock::MemoryLock;
//! use windows::Win32::Foundation::HANDLE;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let handle = HANDLE::default(); // Replace with actual handle
//!     let address = 0x7FF6A1B2C3D4;
//!     
//!     // Lock a value (e.g., health = 100)
//!     let mut lock = MemoryLock::builder()
//!         .handle(handle)
//!         .address(address)
//!         .value(100u32)
//!         .build()?;
//!     
//!     lock.lock_value(100u32)?; // Start locking immediately
//!     
//!     // The value will be continuously restored to 100
//!     // ... your code here ...
//!     
//!     // Lock automatically stops when `lock` goes out of scope
//!     // or you can manually drop it: drop(lock);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Dynamic Address Resolution
//! ```no_run
//! use win_auto_utils::memory_lock::MemoryLock;
//! use win_auto_utils::memory_resolver::MemoryAddress;
//! use windows::Win32::Foundation::HANDLE;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let handle = HANDLE::default(); // Replace with actual handle
//!     
//!     // Use pointer chain that resolves dynamically
//!     let addr = MemoryAddress::new_x86("game.exe+58C94->308")?;
//!     
//!     let mut lock = MemoryLock::builder()
//!         .pid(pid)
//!         .handle(handle)
//!         .address_from_resolver(addr)
//!         .value(500u32)
//!         .build()?;
//!     
//!     lock.lock_value(500u32)?; // Start locking
//!     // Address will be re-resolved on each check cycle
//!     
//!     Ok(())
//! }
//! ```

mod memory_lock;

pub use memory_lock::{MemoryLock, AsBytes};
// Re-export AddressSource from memory_resolver for convenience
pub use crate::memory_resolver::AddressSource;
