//! Process management module
//!
//! Provides process enumeration, handle management, and lifecycle control.
//!
//! # Features
//! - Lazy initialization for optimal performance
//! - Thread-safe with fine-grained locking
//! - Automatic resource cleanup via Drop trait
//! - Batch initialization to minimize system calls
//! - Builder pattern for flexible process configuration
//! - Multiple device context modes (DC, Window DC, Desktop DC)
//!
//! # Device Context Modes
//! The library supports three different DC acquisition modes:
//! - **Mode 1 (DC)**: Standard device context from `get_window_dc()`
//! - **Mode 2 (Window DC)**: Client area device context from `get_window_client_dc()`
//! - **Mode 3 (Desktop DC)**: Full desktop device context for screen capture
//!
//! # Builder Pattern Example
//! ```no_run
//! use win_auto_utils::process::{Process, DCMode};
//!
//! // Simple usage with default DC mode
//! let process = Process::builder("notepad.exe").build();
//!
//! // Desktop mode for full-screen capture
//! let desktop_process = Process::builder("game.exe")
//!     .set_dc_mode(DCMode::Desktop)
//!     .build();
//!
//! // Window client DC mode
//! let window_process = Process::builder("app.exe")
//!     .set_dc_mode(DCMode::WindowClient)
//!     .build();
//!
//! // With window filtering
//! let filters = vec![
//!     ("Document".to_string(), 1),
//!     ("Untitled".to_string(), 0),
//! ];
//! let filtered_process = Process::builder("winword.exe")
//!     .hwnd_filter(filters)
//!     .build();
//!
//! // Complex configuration
//! let complex_process = Process::builder("chrome.exe")
//!     .set_dc_mode(DCMode::Standard)
//!     .hwnd_filter(vec![("Google".to_string(), 1)])
//!     .build();
//! ```

use std::sync::{
    atomic::{AtomicU32, Ordering},
    RwLock,
};

use windows::{
    Win32::Foundation::{HANDLE, HWND},
    Win32::Graphics::Gdi::HDC,
};

use crate::{
    get_window_client_dc, get_window_dc,
    handle::{close_handle, open_process_rw_handle},
    hdc::{get_desktop_dc, release_dc, release_desktop_dc},
    snapshot::get_process_pid,
};

/// Device Context acquisition mode
///
/// Specifies how the device context (DC) should be obtained for screen capture operations.
/// Different modes are suitable for different types of applications and capture scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DCMode {
    /// Standard window DC using `GetWindowDC()`
    /// 
    /// Captures the entire window including title bar and borders.
    /// Suitable for most windowed applications.
    /// Corresponds to mode value: 1
    Standard = 1,
    
    /// Window client area DC using `GetDC()` with client area
    /// 
    /// Captures only the client area (content area) of the window,
    /// excluding title bar, borders, and scrollbars.
    /// Ideal for capturing application content without window chrome.
    /// Corresponds to mode value: 2
    WindowClient = 2,
    
    /// Desktop DC for full-screen capture
    /// 
    /// Captures from the entire desktop rather than a specific window.
    /// Useful for full-screen games, overlays, or multi-window scenarios.
    /// Does not require a specific window handle.
    /// Corresponds to mode value: 3
    Desktop = 3,
}

impl DCMode {
    /// Convert DCMode to numeric value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    /// Create DCMode from numeric value
    /// 
    /// # Arguments
    /// * `value` - Numeric mode value (1, 2, or 3)
    /// 
    /// # Returns
    /// * `Some(DCMode)` - If value is valid
    /// * `None` - If value is invalid
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(DCMode::Standard),
            2 => Some(DCMode::WindowClient),
            3 => Some(DCMode::Desktop),
            _ => None,
        }
    }
}

impl Default for DCMode {
    fn default() -> Self {
        DCMode::Standard
    }
}

/// Error types for process operations
#[derive(Debug, Clone)]
pub enum ProcessError {
    /// Process not found by name
    ProcessNotFound(String),
    /// Failed to open process handle
    HandleOpenFailed(u32),
    /// Failed to get window handle
    WindowNotFound(u32),
    /// Failed to get device context
    DCNotFound(HWND),
    /// Invalid DC mode value
    InvalidDCMode(u8),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::ProcessNotFound(name) => write!(f, "Process '{}' not found", name),
            ProcessError::HandleOpenFailed(pid) => {
                write!(f, "Failed to open handle for PID {}", pid)
            }
            ProcessError::WindowNotFound(pid) => write!(f, "No window found for PID {}", pid),
            ProcessError::DCNotFound(hwnd) => write!(f, "Failed to get DC for window {:?}", hwnd),
            ProcessError::InvalidDCMode(mode) => {
                write!(f, "Invalid DC mode: {} (must be 1, 2, or 3)", mode)
            }
        }
    }
}

impl std::error::Error for ProcessError {}

/// Result type for process operations
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Type alias for window title filter used in Process
/// - String: The pattern to match
/// - u8: Filter mask (1 = must contain, 0 = must NOT contain)
pub type HwndFilter = Vec<(String, u8)>;

/// Builder for creating Process instances with flexible configuration
///
/// Provides a fluent API for configuring process options before creation.
/// All methods return `Self` for method chaining.
///
/// # Example
/// ```no_run
/// use win_auto_utils::process::{Process, DCMode};
///
/// // Simple process with default DC mode
/// let process = Process::builder("notepad.exe").build();
///
/// // Desktop mode for full-screen capture
/// let game = Process::builder("game.exe")
///     .set_dc_mode(DCMode::Desktop)
///     .build();
///
/// // Window client DC mode
/// let app = Process::builder("app.exe")
///     .set_dc_mode(DCMode::WindowClient)
///     .build();
///
/// // With window filtering
/// let filters = vec![
///     ("Document".to_string(), 1),
///     ("Untitled".to_string(), 0),
/// ];
/// let word = Process::builder("winword.exe")
///     .hwnd_filter(filters)
///     .build();
/// ```
#[derive(Debug)]
pub struct ProcessBuilder {
    name: String,
    dc_mode: DCMode,
    hwnd_filter: Option<HwndFilter>,
}

impl ProcessBuilder {
    /// Create a new ProcessBuilder with the specified process name
    ///
    /// # Arguments
    /// * `name` - The process name (e.g., "notepad.exe")
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let builder = Process::builder("chrome.exe");
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            dc_mode: DCMode::default(),
            hwnd_filter: None,
        }
    }

    /// Set the device context acquisition mode
    ///
    /// Controls how the device context (DC) is obtained for screen capture.
    /// See `DCMode` enum for available options.
    ///
    /// # Arguments
    /// * `mode` - The DC mode to use
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{Process, DCMode};
    ///
    /// // Use desktop DC for full-screen capture
    /// let game = Process::builder("game.exe")
    ///     .set_dc_mode(DCMode::Desktop)
    ///     .build();
    ///
    /// // Use window client DC for content-only capture
    /// let app = Process::builder("app.exe")
    ///     .set_dc_mode(DCMode::WindowClient)
    ///     .build();
    /// ```
    pub fn set_dc_mode(mut self, mode: DCMode) -> Self {
        self.dc_mode = mode;
        self
    }

    /// Set DC mode using numeric value (convenience method)
    ///
    /// # Arguments
    /// * `mode_value` - Mode value: 1=Standard, 2=WindowClient, 3=Desktop
    ///
    /// # Returns
    /// * `Self` if mode is valid
    /// * Panics if mode is invalid (use `try_set_dc_mode_num` for fallible version)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let game = Process::builder("game.exe")
    ///     .set_dc_mode_num(3)  // Desktop mode
    ///     .build();
    /// ```
    pub fn set_dc_mode_num(mut self, mode_value: u8) -> Self {
        self.dc_mode = DCMode::from_u8(mode_value)
            .unwrap_or_else(|| panic!("Invalid DC mode: {}. Must be 1, 2, or 3", mode_value));
        self
    }

    /// Try to set DC mode using numeric value (fallible version)
    ///
    /// # Arguments
    /// * `mode_value` - Mode value: 1=Standard, 2=WindowClient, 3=Desktop
    ///
    /// # Returns
    /// * `Ok(Self)` if mode is valid
    /// * `Err(Self)` if mode is invalid (returns self for further configuration)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let result = Process::builder("game.exe")
    ///     .try_set_dc_mode_num(3);
    ///     
    /// match result {
    ///     Ok(builder) => {
    ///         let process = builder.build();
    ///         println!("Successfully configured");
    ///     }
    ///     Err(_) => eprintln!("Invalid DC mode"),
    /// }
    /// ```
    pub fn try_set_dc_mode_num(mut self, mode_value: u8) -> Result<Self, Self> {
        match DCMode::from_u8(mode_value) {
            Some(mode) => {
                self.dc_mode = mode;
                Ok(self)
            }
            None => Err(self),
        }
    }

    /// Set desktop mode - shorthand for `dc_mode(DCMode::Desktop)`
    ///
    /// This is a convenience method that sets the DC mode to Desktop.
    /// Equivalent to calling `.dc_mode(DCMode::Desktop)` or `.dc_mode_value(3)`.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let game = Process::builder("game.exe")
    ///     .desktop_mode()  // Sets DC mode to Desktop
    ///     .build();
    /// ```
    pub fn desktop_mode(self) -> Self {
        self.set_dc_mode(DCMode::Desktop)
    }

    /// Set window client DC mode - shorthand for `dc_mode(DCMode::WindowClient)`
    ///
    /// This is a convenience method that sets the DC mode to WindowClient.
    /// Captures only the client area of the window.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let app = Process::builder("app.exe")
    ///     .window_client_mode()  // Sets DC mode to WindowClient
    ///     .build();
    /// ```
    pub fn window_client_mode(self) -> Self {
        self.set_dc_mode(DCMode::WindowClient)
    }

    /// Set standard window DC mode - shorthand for `dc_mode(DCMode::Standard)`
    ///
    /// This is a convenience method that sets the DC mode to Standard.
    /// This is also the default mode.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let app = Process::builder("app.exe")
    ///     .standard_dc_mode()  // Explicitly set to Standard (default)
    ///     .build();
    /// ```
    pub fn standard_dc_mode(self) -> Self {
        self.set_dc_mode(DCMode::Standard)
    }

    /// Set window title filters for flexible window selection
    ///
    /// Filters use an include/exclude mechanism:
    /// - Mask 1: Window title MUST contain the pattern
    /// - Mask 0: Window title MUST NOT contain the pattern
    /// All filters must match for a window to be selected.
    ///
    /// # Arguments
    /// * `filter` - Vector of (pattern, mask) tuples
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let filters = vec![
    ///     ("Document".to_string(), 1),   // Must contain
    ///     ("Untitled".to_string(), 0),   // Must NOT contain
    /// ];
    ///
    /// let word = Process::builder("winword.exe")
    ///     .set_hwnd_filter(filters)
    ///     .build();
    /// ```
    pub fn set_hwnd_filter(mut self, filter: HwndFilter) -> Self {
        self.hwnd_filter = Some(filter);
        self
    }

    /// Clear window filter (use auto-detection instead)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let process = Process::builder("app.exe")
    ///     .hwnd_filter(vec![("Test".to_string(), 1)])
    ///     .clear_hwnd_filter()  // Remove the filter
    ///     .build();
    /// ```
    pub fn clear_hwnd_filter(mut self) -> Self {
        self.hwnd_filter = None;
        self
    }

    /// Build the Process instance with current configuration
    ///
    /// Creates a Process with the configured options but does not
    /// initialize the connection. Call `process.init()` to connect.
    ///
    /// # Returns
    /// A configured Process instance
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{Process, DCMode};
    ///
    /// let process = Process::builder("notepad.exe")
    ///     .set_dc_mode(DCMode::Desktop)
    ///     .build();
    ///
    /// // Initialize when ready
    /// if process.init().is_ok() {
    ///     println!("Connected!");
    /// }
    /// ```
    pub fn build(self) -> Process {
        Process {
            name: self.name,
            cached_pid: AtomicU32::new(0),
            dc_mode: self.dc_mode,
            hwnd_filter: self.hwnd_filter,
            info: RwLock::new(None),
        }
    }
}

/// Internal process information - batch managed with single lock
#[derive(Debug, Clone, Copy)]
struct ProcessInfo {
    pid: u32,
    hwnd: HWND,
    handle: HANDLE,
    hdc: HDC,
}

impl ProcessInfo {
    fn is_valid(&self) -> bool {
        self.pid != 0 && !self.handle.is_invalid()
    }

    /// Clean up all resources
    fn cleanup(&mut self) {
        if !self.handle.is_invalid() {
            close_handle(self.handle);
            self.handle = HANDLE::default();
        }

        if !self.hdc.0.is_null() && !self.hwnd.0.is_null() {
            release_dc(self.hwnd, self.hdc);
            self.hdc = HDC::default();
        }
    }
}

/// Process manager with optimized initialization and resource management
///
/// # Design Goals
/// - Lazy initialization for optimal startup performance
/// - Thread-safe with fine-grained locking (RwLock for reads, exclusive for writes)
/// - Automatic resource cleanup via Drop trait
/// - Batch initialization to minimize system calls
/// - Lock-free PID access via AtomicU32
/// - Support for multiple DC modes and custom window filtering
///
/// # Device Context Modes
/// The `dc_mode` field controls how the device context is acquired:
/// - `DCMode::Standard` (1): Full window DC including title bar and borders
/// - `DCMode::WindowClient` (2): Client area only, excluding window chrome
/// - `DCMode::Desktop` (3): Full desktop DC for screen capture
///
/// # Window Filtering
/// The `hwnd_filter` allows flexible window selection using pattern matching:
/// - Each filter is a `(String, u8)` tuple
/// - Mask 1: Window title MUST contain the pattern
/// - Mask 0: Window title MUST NOT contain the pattern
/// - All filters must match for a window to be selected
///
/// # Example
/// ```no_run
/// use win_auto_utils::process::{Process, DCMode};
///
/// // Standard usage - captures from the first window of the process
/// let process = Process::new("notepad.exe");
///
/// // Desktop mode - captures from entire desktop
/// let desktop_process = Process::builder("game.exe")
///     .set_dc_mode(DCMode::Desktop)
///     .build();
///
/// // Window client mode - captures client area only
/// let client_process = Process::builder("app.exe")
///     .set_dc_mode(DCMode::WindowClient)
///     .build();
///
/// // Custom window filtering - captures from windows containing "Document" but not "Untitled"
/// let filters = vec![
///     ("Document".to_string(), 1),   // Must contain "Document"
///     ("Untitled".to_string(), 0),   // Must NOT contain "Untitled"
/// ];
/// let filtered_process = Process::builder("word.exe")
///     .set_hwnd_filter(filters)
///     .build();
/// ```
#[derive(Debug)]
pub struct Process {
    /// Process name (immutable after creation)
    name: String,

    /// Cached PID - atomic for lock-free reads
    cached_pid: AtomicU32,

    /// Device context acquisition mode
    /// - 1 (Standard): Full window DC
    /// - 2 (WindowClient): Client area DC
    /// - 3 (Desktop): Desktop DC
    dc_mode: DCMode,

    /// Optional HWND filter - if set, uses filtered window selection instead of auto-detection
    /// Format: Vec<(pattern: String, mask: u8)>
    /// - mask 1: window title MUST contain pattern
    /// - mask 0: window title MUST NOT contain pattern
    hwnd_filter: Option<HwndFilter>,

    /// Process handles - protected by single RwLock for batch updates
    info: RwLock<Option<ProcessInfo>>,
}

// Safety: HANDLE, HWND, HDC are FFI types that are safe to send across threads
// The RwLock ensures proper synchronization
unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    /// Create a ProcessBuilder for flexible process configuration
    ///
    /// This is the recommended way to create Process instances with custom options.
    /// Provides a fluent API for method chaining.
    ///
    /// # Arguments
    /// * `name` - The process name (e.g., "notepad.exe")
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// // Simple usage
    /// let process = Process::builder("notepad.exe").build();
    ///
    /// // Desktop mode
    /// let game = Process::builder("game.exe")
    ///     .desktop_mode()
    ///     .build();
    ///
    /// // With window filtering
    /// let filters = vec![
    ///     ("Document".to_string(), 1),
    ///     ("Untitled".to_string(), 0),
    /// ];
    /// let word = Process::builder("winword.exe")
    ///     .hwnd_filter(filters)
    ///     .build();
    /// ```
    pub fn builder(name: &str) -> ProcessBuilder {
        ProcessBuilder::new(name)
    }

    /// Create a new Process instance without initialization (legacy API)
    ///
    /// This is a lightweight operation that doesn't perform any system calls.
    /// Call `init()` to establish the connection to the actual process.
    ///
    /// For more flexible configuration, consider using `Process::builder()` instead.
    ///
    /// # Arguments
    /// * `name` - The process name (e.g., "notepad.exe")
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let process = Process::new("notepad.exe");
    /// assert_eq!(process.get_name(), "notepad.exe");
    /// assert!(!process.is_valid());  // Not initialized yet
    /// ```
    pub fn new(name: &str) -> Self {
        Self::builder(name).build()
    }

    /// Get process name reference (zero-copy)
    ///
    /// Returns a reference to the process name string.
    /// No allocation or copying occurs.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the device context acquisition mode
    ///
    /// Returns the current DC mode configuration.
    /// 
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{Process, DCMode};
    ///
    /// let process = Process::builder("game.exe")
    ///     .dc_mode(DCMode::Desktop)
    ///     .build();
    ///
    /// assert_eq!(process.get_dc_mode(), DCMode::Desktop);
    /// ```
    pub fn get_dc_mode(&self) -> DCMode {
        self.dc_mode
    }

    /// Get the DC mode as numeric value
    ///
    /// Returns the numeric representation of the DC mode:
    /// - 1: Standard window DC
    /// - 2: Window client DC
    /// - 3: Desktop DC
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let process = Process::builder("game.exe")
    ///     .desktop_mode()
    ///     .build();
    ///
    /// assert_eq!(process.get_dc_mode_value(), 3);
    /// ```
    pub fn get_dc_mode_value(&self) -> u8 {
        self.dc_mode.as_u8()
    }

    /// Get the HWND filter configuration
    ///
    /// Returns the window filter patterns if set,
    /// or None if using auto-detection.
    pub fn get_hwnd_filter(&self) -> Option<&HwndFilter> {
        self.hwnd_filter.as_ref()
    }

    /// Initialize the process connection
    ///
    /// Performs batch initialization of all process resources:
    /// 1. Find PID by process name
    /// 2. Get main window handle (HWND)
    /// 3. Open process handle with read/write access
    /// 4. Get device context (DC) for the window
    ///
    /// All resources are acquired in a single operation and stored atomically.
    /// If any step fails, no partial state is left behind.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully initialized
    /// * `Err(ProcessError)` - Initialization failed with specific error
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let process = Process::new("notepad.exe");
    ///
    /// match process.init() {
    ///     Ok(()) => println!("Connected to PID: {}", process.get_pid()),
    ///     Err(e) => eprintln!("Failed to initialize: {}", e),
    /// }
    /// ```
    pub fn init(&self) -> ProcessResult<()> {
        // Step 1: Find PID by process name
        let pid =
            get_process_pid(&self.name).ok_or_else(|| ProcessError::ProcessNotFound(self.name.clone()))?;

        // Step 2: Get window handle (not needed for Desktop mode)
        let hwnd = match self.dc_mode {
            DCMode::Desktop => {
                // In desktop mode, we don't need a specific window
                // Use a placeholder HWND (will use desktop DC later)
                HWND::default()
            }
            _ => {
                // For Standard and WindowClient modes, we need a window handle
                if let Some(filter) = &self.hwnd_filter {
                    // Use filtered window selection
                    crate::hwnd::get_hwnd_by_pid_and_filter(pid, filter)
                        .ok_or_else(|| ProcessError::WindowNotFound(pid))?
                } else {
                    // Auto-detect the first window with a title
                    crate::hwnd::get_hwnd_by_pid(pid).ok_or_else(|| ProcessError::WindowNotFound(pid))?
                }
            }
        };

        // Step 3: Open process handle with read/write access
        let handle =
            open_process_rw_handle(pid).ok_or_else(|| ProcessError::HandleOpenFailed(pid))?;

        // Step 4: Get device context based on DC mode
        let hdc = match self.dc_mode {
            DCMode::Desktop => {
                // Get desktop DC for full-screen capture
                get_desktop_dc().ok_or_else(|| ProcessError::DCNotFound(HWND::default()))?
            }
            DCMode::Standard => {
                // Get full window DC (including title bar and borders)
                get_window_dc(hwnd).ok_or_else(|| ProcessError::DCNotFound(hwnd))?
            }
            DCMode::WindowClient => {
                // Get client area DC (content area only)
                get_window_client_dc(hwnd).ok_or_else(|| ProcessError::DCNotFound(hwnd))?
            }
        };

        // Create process info
        let new_info = ProcessInfo {
            pid,
            hwnd,
            handle,
            hdc,
        };

        // Atomically update state with write lock
        {
            let mut info_lock = self.info.write().map_err(|_| {
                // Clean up resources if we can't acquire the lock
                close_handle(handle);
                match self.dc_mode {
                    DCMode::Desktop => {
                        release_desktop_dc(hdc);
                    }
                    _ => {
                        release_dc(hwnd, hdc);
                    }
                }
                ProcessError::HandleOpenFailed(pid)
            })?;

            // Clean up old resources if they exist
            if let Some(old_info) = info_lock.as_mut() {
                old_info.cleanup();
            }

            // Store new info
            *info_lock = Some(new_info);
        }

        // Update cached PID atomically (lock-free for readers)
        self.cached_pid.store(pid, Ordering::Release);

        Ok(())
    }

    /// Re-initialize the process connection
    ///
    /// Cleans up existing resources and re-establishes the connection.
    /// Useful when a process has been restarted or the connection was lost.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully re-initialized
    /// * `Err(ProcessError)` - Re-initialization failed
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let process = Process::new("notepad.exe");
    /// process.init().ok();
    ///
    /// // ... process restarts ...
    ///
    /// process.reinit().ok();  // Reconnect to new instance
    /// ```
    pub fn reinit(&self) -> ProcessResult<()> {
        // Clean up existing resources first
        self.cleanup();

        // Re-initialize
        self.init()
    }

    /// Clean up all process resources
    ///
    /// Closes the process handle and releases the device context.
    /// This is automatically called when the Process is dropped,
    /// but can be called manually to free resources early.
    pub fn cleanup(&self) {
        if let Some(info) = self.info.write().ok().and_then(|mut lock| {
            let info = lock.take();
            drop(lock); // Release write lock before cleanup
            info
        }) {
            // Release DC based on mode
            match self.dc_mode {
                DCMode::Desktop => {
                    // For desktop mode, release desktop DC
                    if !info.hdc.0.is_null() {
                        crate::hdc::release_desktop_dc(info.hdc);
                    }
                }
                _ => {
                    // For Standard and WindowClient modes, use standard cleanup
                    if !info.hdc.0.is_null() && !info.hwnd.0.is_null() {
                        release_dc(info.hwnd, info.hdc);
                    }
                }
            }

            // Always close the process handle
            if !info.handle.is_invalid() {
                close_handle(info.handle);
            }
        }

        // Reset cached PID
        self.cached_pid.store(0, Ordering::Release);
    }

    /// Get process handle
    ///
    /// Returns the process handle obtained during initialization.
    /// Returns an invalid handle if not initialized.
    ///
    /// # Note
    /// This does NOT increment any reference count. The handle is valid
    /// until `cleanup()` is called or the Process is dropped.
    pub fn get_handle(&self) -> HANDLE {
        self.info
            .read()
            .ok()
            .and_then(|info| info.map(|i| i.handle))
            .unwrap_or(HANDLE::default())
    }

    /// Get PID (lock-free atomic read)
    ///
    /// Returns the cached process ID. This is a very fast operation
    /// that doesn't require any locks.
    ///
    /// # Returns
    /// The process ID, or 0 if not initialized
    pub fn get_pid(&self) -> u32 {
        self.cached_pid.load(Ordering::Acquire)
    }

    /// Get window handle
    ///
    /// Returns the main window handle for this process.
    /// Returns an invalid HWND if not initialized or no window exists.
    pub fn get_hwnd(&self) -> HWND {
        self.info
            .read()
            .ok()
            .and_then(|info| info.map(|i| i.hwnd))
            .unwrap_or(HWND::default())
    }

    /// Get device context
    ///
    /// Returns the device context for the process's main window.
    /// Returns an invalid HDC if not initialized.
    ///
    /// # Important
    /// The DC is managed by the Process and will be automatically released
    /// when the Process is dropped or cleaned up. Do NOT manually release it.
    pub fn get_hdc(&self) -> HDC {
        self.info
            .read()
            .ok()
            .and_then(|info| info.map(|i| i.hdc))
            .unwrap_or(HDC::default())
    }

    /// Check if process is valid and initialized
    ///
    /// Returns true if the process has been successfully initialized
    /// and has a valid handle.
    pub fn is_valid(&self) -> bool {
        self.info
            .read()
            .ok()
            .and_then(|info| info.as_ref().map(|i| i.is_valid()))
            .unwrap_or(false)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let process = Process::new("test.exe");
        assert_eq!(process.get_name(), "test.exe");
        assert!(!process.is_valid());
        assert_eq!(process.get_pid(), 0);
        assert!(process.get_handle().is_invalid());
        assert_eq!(process.get_hwnd(), HWND::default());
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Process>();
        assert_sync::<Process>();
    }

    #[test]
    fn test_init_nonexistent_process() {
        let process = Process::new("definitely_not_existing_xyz123.exe");
        let result = process.init();

        assert!(result.is_err(), "Should fail for non-existent process");

        if let Err(e) = result {
            match e {
                ProcessError::ProcessNotFound(name) => {
                    assert_eq!(name, "definitely_not_existing_xyz123.exe");
                }
                _ => panic!("Expected ProcessNotFound error"),
            }
        }

        assert!(!process.is_valid());
    }

    #[test]
    fn test_cleanup_without_init() {
        let process = Process::new("test.exe");

        // Should not panic even if not initialized
        process.cleanup();

        assert!(!process.is_valid());
        assert_eq!(process.get_pid(), 0);
    }

    #[test]
    fn test_reinit_without_init() {
        let process = Process::new("test.exe");

        // Should fail gracefully
        let result = process.reinit();
        assert!(result.is_err());
    }

    // ========== Builder Pattern Tests ==========

    #[test]
    fn test_builder_simple() {
        let process = Process::builder("notepad.exe").build();
        assert_eq!(process.get_name(), "notepad.exe");
        assert_eq!(process.get_dc_mode(), DCMode::Standard);
        assert_eq!(process.get_hwnd_filter(), None);
        assert!(!process.is_valid());
    }

    #[test]
    fn test_builder_dc_mode_explicit() {
        // Test desktop mode using enum
        let process = Process::builder("app.exe")
            .set_dc_mode(DCMode::Desktop)
            .build();
        assert_eq!(process.get_dc_mode(), DCMode::Desktop);
        assert_eq!(process.get_dc_mode_value(), 3);

        // Test window client mode
        let process2 = Process::builder("app.exe")
            .set_dc_mode(DCMode::WindowClient)
            .build();
        assert_eq!(process2.get_dc_mode(), DCMode::WindowClient);
        assert_eq!(process2.get_dc_mode_value(), 2);

        // Test standard mode (default)
        let process3 = Process::builder("app.exe").build();
        assert_eq!(process3.get_dc_mode(), DCMode::Standard);
        assert_eq!(process3.get_dc_mode_value(), 1);
    }

    #[test]
    fn test_builder_dc_mode_numeric() {
        // Test using numeric values
        let process1 = Process::builder("app.exe")
            .set_dc_mode_num(3)
            .build();
        assert_eq!(process1.get_dc_mode_value(), 3);

        let process2 = Process::builder("app.exe")
            .set_dc_mode_num(2)
            .build();
        assert_eq!(process2.get_dc_mode_value(), 2);

        let process3 = Process::builder("app.exe")
            .set_dc_mode_num(1)
            .build();
        assert_eq!(process3.get_dc_mode_value(), 1);
    }

    #[test]
    fn test_builder_try_dc_mode_value() {
        // Test fallible version
        let result = Process::builder("app.exe").try_set_dc_mode_num(3);
        assert!(result.is_ok());
        let process = result.unwrap().build();
        assert_eq!(process.get_dc_mode_value(), 3);

        // Test invalid value
        let result = Process::builder("app.exe").try_set_dc_mode_num(99);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_complex_configuration() {
        let filters = vec![("Google".to_string(), 1), ("Incognito".to_string(), 0)];

        let process = Process::builder("chrome.exe")
            .set_dc_mode(DCMode::Standard)
            .set_hwnd_filter(filters.clone())
            .build();

        assert_eq!(process.get_name(), "chrome.exe");
        assert_eq!(process.get_dc_mode(), DCMode::Standard);
        assert_eq!(process.get_hwnd_filter(), Some(&filters));
    }

    #[test]
    fn test_builder_clear_hwnd_filter() {
        let filters = vec![("Test".to_string(), 1)];

        let process = Process::builder("app.exe")
            .set_hwnd_filter(filters)
            .clear_hwnd_filter()
            .build();

        assert_eq!(process.get_hwnd_filter(), None);
    }

    #[test]
    fn test_builder_method_chaining() {
        // Test that all methods return Self for chaining
        let filters = vec![("Pattern".to_string(), 1)];

        let _process = Process::builder("test.exe")
            .desktop_mode()
            .set_hwnd_filter(filters)
            .set_dc_mode(DCMode::Standard)
            .clear_hwnd_filter()
            .window_client_mode()
            .build();

        // Just verify it compiles and builds successfully
    }

    #[test]
    fn test_builder_vs_legacy_api_equivalence() {
        // Verify builder produces same results as legacy API

        // Test 1: Simple creation
        let process1 = Process::new("test.exe");
        let process2 = Process::builder("test.exe").build();
        assert_eq!(process1.get_name(), process2.get_name());
        assert_eq!(process1.get_dc_mode(), process2.get_dc_mode());
        assert_eq!(process1.get_hwnd_filter(), process2.get_hwnd_filter());
    }

    // ========== Legacy API Tests (for backward compatibility) ==========

    #[test]
    fn test_default_process_flags() {
        let process = Process::new("test.exe");
        assert_eq!(process.get_dc_mode(), DCMode::Standard);
        assert_eq!(process.get_hwnd_filter(), None);
    }
}
