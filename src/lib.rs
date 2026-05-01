// === Core Modules (Feature-gated for atomic usage) ===
#[cfg(feature = "process")]
pub mod process;

// Re-export new process architecture components
#[cfg(feature = "process")]
pub use process::{
    Process, ProcessConfig, ProcessManager,
};

#[cfg(feature = "hwnd")]
pub mod hwnd;

#[cfg(feature = "window")]
pub mod window;

#[cfg(feature = "snapshot")]
pub mod snapshot;

#[cfg(feature = "handle")]
pub mod handle;

#[cfg(feature = "hdc")]
pub mod hdc;

// === Input Modules (Independent) ===
#[cfg(feature = "keyboard")]
pub mod keyboard;

#[cfg(feature = "mouse")]
pub mod mouse;

// === Utility Modules ===
pub mod utils;

#[cfg(feature = "color_picker")]
pub mod color_picker;

#[cfg(feature = "color_finder")]
pub mod color_finder;

// === Clipboard Module===
#[cfg(feature = "clipboard")]
pub mod clipboard;

// === Memory Operations Module===
#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "memory_lock")]
pub mod memory_lock;

#[cfg(feature = "memory_hook")]
pub mod memory_hook;

#[cfg(feature = "memory_resolver")]
pub mod memory_resolver;

#[cfg(feature = "memory_aobscan")]
pub mod memory_aobscan;

#[cfg(feature = "memory_manager")]
pub mod memory_manager;

// === Advanced Modules===
#[cfg(feature = "dxgi")]
pub mod dxgi;

#[cfg(feature = "dxgi")]
pub use dxgi::DxgiError;

#[cfg(feature = "template_matcher")]
pub mod template_matcher;

#[cfg(feature = "dll_injector")]
pub mod dll_injector;

#[cfg(feature = "script_engine")]
pub mod script_engine;

// Built-in instruction modules (enabled for testing)
#[cfg(any(
    feature = "scripts_builtin",
    feature = "scripts_control_flow",
    feature = "scripts_keyboard",
    feature = "scripts_mouse",
    feature = "scripts_timing",
    feature = "scripts_mode",
))]
pub mod scripts_builtin;

// === Re-exports for convenience ===
#[cfg(feature = "hwnd")]
pub use hwnd::*;

#[cfg(feature = "window")]
pub use window::*;

#[cfg(feature = "snapshot")]
pub use snapshot::*;

#[cfg(feature = "hdc")]
pub use hdc::*;

#[cfg(feature = "handle")]
pub use handle::*;

// Re-export input modules if enabled
#[cfg(feature = "keyboard")]
pub use keyboard::*;

#[cfg(feature = "mouse")]
pub use mouse::*;
