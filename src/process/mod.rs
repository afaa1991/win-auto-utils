//! Process management module (refactored)
//!
//! Provides a three-layer architecture for process management:
//! - **Config Layer**: Immutable configurations defining how to find processes
//! - **Instance Layer**: Runtime state holding actual system resources
//! - **Manager Layer**: Centralized lifecycle management for multiple processes
//!
//! # Architecture Overview
//! ```text
//! ProcessManager (manages multiple processes)
//!   └─> Process (single process instance)
//!         ├─ Config (immutable settings)
//!         └─ State (runtime resources: PID, handles, DC)
//! ```
//!
//! # Quick Start
//!
//! ## Simple Usage (Quick Initialization)
//! ```no_run
//! use win_auto_utils::process::Process;
//!
//! // Quick way: just specify the process name
//! let mut process = Process::by_name("notepad.exe");
//! if let Ok(_) = process.init() {
//!     println!("PID: {:?}", process.pid());
//! }
//! ```
//!
//! ## Advanced Usage (With Builder)
//! ```no_run
//! use win_auto_utils::process::{Process, ProcessConfig};
//!
//! // Use builder for fluent configuration
//! let config = ProcessConfig::builder("game.exe")
//!     .set_window_client_mode()
//!     .exclude_invisible()
//!     .build();
//! let mut process = Process::new(config);
//! if let Ok(_) = process.init() {
//!     println!("Connected!");
//! }
//! ```
//!
//! ## Custom Initialization Flags
//! ```no_run
//! use win_auto_utils::process::{Process, ProcessConfig, InitFlags};
//!
//! // Control which resources are initialized based on your needs
//! let config = ProcessConfig::builder("app.exe")
//!     .init_flags(InitFlags::memory_only())  // Only PID + HANDLE for memory operations
//!     .build();
//! let mut process = Process::new(config);
//! if let Ok(_) = process.init() {
//!     println!("Ready for memory operations");
//! }
//! ```
//!
//! ## Manager Usage (Multiple Processes)
//! ```no_run
//! use win_auto_utils::process::ProcessManager;
//!
//! let mut manager = ProcessManager::new();
//! manager.register_alias("game", "lf2.exe").ok();
//! manager.init("game").ok();
//! ```

pub mod config;
pub mod state;
pub mod manager;
mod process;

// Re-export main types for convenience
pub use config::{ProcessConfig, ProcessConfigBuilder, DCMode, WindowFilter, FilterRuleType, FilterCriterion, InitFlags};
pub use manager::ProcessManager;
pub use process::{Process, ProcessError, ProcessResult};