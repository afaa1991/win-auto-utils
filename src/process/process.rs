//! Process implementation module
//!
//! Contains the main Process structure that combines configuration and runtime state.

use crate::process::config::{DCMode, FilterCriterion, FilterRuleType, ProcessConfig, WindowFilter};
use crate::process::state::ProcessState;
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Graphics::Gdi::HDC;

use crate::{
    get_window_client_dc, get_window_dc, handle::open_process_rw_handle, hdc::get_desktop_dc,
    snapshot::get_process_pid,
};

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
    /// Duplicate process name in manager
    DuplicateName(String),
    /// Process not registered in manager
    ProcessNotRegistered(String),
    /// RwLock poisoned (thread panic while holding lock)
    LockPoisoned,
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
            ProcessError::DuplicateName(name) => {
                write!(f, "Process '{}' is already registered", name)
            }
            ProcessError::ProcessNotRegistered(name) => {
                write!(f, "Process '{}' is not registered", name)
            }
            ProcessError::LockPoisoned => {
                write!(
                    f,
                    "RwLock poisoned (a thread panicked while holding the lock)"
                )
            }
        }
    }
}

impl std::error::Error for ProcessError {}

/// Result type for process operations
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Process instance combining configuration and runtime state
///
/// # Architecture
/// - **Configuration**: Immutable settings defined at creation time
/// - **Runtime State**: Dynamically acquired resources (PID, handles, DC)
///
/// # Usage Pattern
/// 1. Create with `Process::new(config)`
/// 2. Initialize with `process.init()` or `process.init_by_pid(pid)`
/// 3. Access state via getter methods
/// 4. Clean up with `process.cleanup()` or let it drop automatically
///
/// # Example
/// ```no_run
/// use win_auto_utils::process::{Process, ProcessConfig};
///
/// // Create configuration using builder
/// let config = ProcessConfig::builder("notepad.exe").build();
///
/// // Create process instance
/// let mut process = Process::new(config);
///
/// // Initialize (finds process by name)
/// if process.init().is_ok() {
///     println!("Connected to PID: {}", process.pid().unwrap());
/// }
/// ```
#[derive(Debug)]
pub struct Process {
    /// Configuration (immutable after creation)
    config: ProcessConfig,

    /// Runtime state (None if not initialized)
    state: Option<ProcessState>,
}

// Safety: HANDLE, HWND, HDC are FFI types safe to send across threads
// Caller is responsible for synchronization (e.g., wrapping in RwLock)
unsafe impl Send for Process {}
unsafe impl Sync for Process {}

impl Process {
    // ==================== Constructors ====================

    /// Create a new process instance with the given configuration
    pub fn new(config: ProcessConfig) -> Self {
        Self {
            config,
            state: None,
        }
    }

    /// Quick constructor by process name only (uses default settings)
    ///
    /// This is a convenience method for simple cases where you only need
    /// to specify the process name. Uses default DC mode (WindowClient).
    ///
    /// # Arguments
    /// * `process_name` - The executable name (e.g., "notepad.exe")
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// // Two-step initialization
    /// let mut process = Process::by_name("notepad.exe");
    /// process.init()?;
    /// println!("PID: {:?}", process.pid());
    /// ```
    pub fn by_name(process_name: &str) -> Self {
        let config = ProcessConfig::new(process_name);
        Self::new(config)
    }

    /// Quick initialization by process name (one-step)
    ///
    /// This is the most convenient way to create and initialize a process in one call.
    /// Creates a new process instance with the given name and immediately initializes it.
    ///
    /// # Arguments
    /// * `process_name` - The executable name (e.g., "notepad.exe")
    ///
    /// # Returns
    /// * `Ok(Process)` - Successfully initialized process
    /// * `Err(ProcessError)` - Initialization failed
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// // One-step initialization (recommended for simple cases)
    /// let mut process = Process::init_by_name("notepad.exe")?;
    /// println!("PID: {}", process.pid().unwrap());
    /// ```
    pub fn init_by_name(process_name: &str) -> ProcessResult<Self> {
        let mut process = Self::by_name(process_name);
        process.init()?;
        Ok(process)
    }

    /// Quick initialization by process ID (one-step convenience method)
    ///
    /// This is a convenience method to quickly get State data when you already know the PID.
    /// Creates a new process instance and initializes it using the provided PID directly.
    /// Bypasses the configured lookup strategy.
    ///
    /// # Arguments
    /// * `pid` - The process ID to connect to
    ///
    /// # Returns
    /// * `Ok(Process)` - Successfully initialized process with State data
    /// * `Err(ProcessError)` - Initialization failed (e.g., process not found, access denied)
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// // Quick initialization by known PID (e.g., from Task Manager)
    /// let process = Process::init_by_pid(12345)?;
    /// println!("PID: {}", process.pid().unwrap());
    /// println!("HWND: {:?}", process.hwnd().unwrap());
    /// println!("Handle: {:?}", process.handle().unwrap());
    /// ```
    pub fn init_by_pid(pid: u32) -> ProcessResult<Self> {
        // Create a minimal config with empty name (will be bypassed anyway)
        let config = ProcessConfig::new("");
        let mut process = Self::new(config);
        process.init_with_pid(pid)?;
        Ok(process)
    }

    // ==================== Configuration Accessors ====================

    /// Get reference to the configuration
    pub fn config(&self) -> &ProcessConfig {
        &self.config
    }

    /// Get the DC mode from configuration
    pub fn dc_mode(&self) -> DCMode {
        self.config.dc_mode
    }

    // ==================== State Accessors ====================

    /// Get the process ID (if initialized)
    pub fn pid(&self) -> Option<u32> {
        self.state.as_ref().map(|s| s.pid)
    }

    /// Get the window handle (if initialized)
    pub fn hwnd(&self) -> Option<HWND> {
        self.state.as_ref().map(|s| s.hwnd)
    }

    /// Get the process handle (if initialized)
    pub fn handle(&self) -> Option<HANDLE> {
        self.state.as_ref().map(|s| s.handle)
    }

    /// Get the device context (if initialized)
    pub fn hdc(&self) -> Option<HDC> {
        self.state.as_ref().map(|s| s.hdc)
    }

    // ==================== Convenient Accessors with Defaults ====================

    /// Get the window handle, or HWND::default() if not initialized
    ///
    /// This is a convenience method that always returns a valid HWND value.
    /// If the process is not initialized, returns HWND::default() (null handle).
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let mut process = Process::by_name("notepad.exe");
    /// // Before init: returns null HWND
    /// let hwnd = process.hwnd_or_default();
    /// println!("HWND: {:?}", hwnd);
    ///
    /// process.init().ok();
    /// // After init: returns actual HWND
    /// let hwnd = process.hwnd_or_default();
    /// println!("HWND: {:?}", hwnd);
    /// ```
    pub fn hwnd_or_default(&self) -> HWND {
        self.hwnd().unwrap_or_default()
    }

    /// Get the process handle, or HANDLE::default() if not initialized
    ///
    /// This is a convenience method that always returns a valid HANDLE value.
    /// If the process is not initialized, returns HANDLE::default() (invalid handle).
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let mut process = Process::by_name("notepad.exe");
    /// let handle = process.handle_or_default();
    /// println!("Handle valid: {}", !handle.is_invalid());
    /// ```
    pub fn handle_or_default(&self) -> HANDLE {
        self.handle().unwrap_or_default()
    }

    /// Get the device context, or HDC::default() if not initialized
    ///
    /// This is a convenience method that always returns a valid HDC value.
    /// If the process is not initialized, returns HDC::default() (null DC).
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let mut process = Process::by_name("notepad.exe");
    /// let hdc = process.hdc_or_default();
    /// println!("HDC: {:?}", hdc);
    /// ```
    pub fn hdc_or_default(&self) -> HDC {
        self.hdc().unwrap_or_default()
    }

    /// Get the process ID, or 0 if not initialized
    ///
    /// This is a convenience method that always returns a valid PID value.
    /// If the process is not initialized, returns 0.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::Process;
    ///
    /// let mut process = Process::by_name("notepad.exe");
    /// let pid = process.pid_or_default();
    /// println!("PID: {}", pid); // Will print 0 before init
    ///
    /// process.init().ok();
    /// let pid = process.pid_or_default();
    /// println!("PID: {}", pid); // Will print actual PID after init
    /// ```
    pub fn pid_or_default(&self) -> u32 {
        self.pid().unwrap_or(0)
    }

    /// Check if the process is initialized and valid (handles are non-null)
    pub fn is_valid(&self) -> bool {
        self.state.as_ref().map_or(false, |s| s.is_valid())
    }

    /// Get reference to the runtime state
    pub fn state(&self) -> Option<&ProcessState> {
        self.state.as_ref()
    }

    // ==================== Initialization Methods ====================

    /// Initialize the process using the configured lookup strategy
    ///
    /// This method searches for the process based on the configuration.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully initialized
    /// * `Err(ProcessError)` - Initialization failed
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{Process, ProcessConfig};
    ///
    /// let config = ProcessConfig::builder("notepad.exe").build();
    /// let mut process = Process::new(config);
    ///
    /// if process.init().is_ok() {
    ///     println!("Found process with PID: {:?}", process.pid());
    /// }
    /// ```
    pub fn init(&mut self) -> ProcessResult<()> {
        // Clean up existing resources first
        if self.state.is_some() {
            self.cleanup()?;
        }

        // Find process based on configuration
        let (pid, hwnd) = self.find_process()?;

        // Open process handle
        let handle =
            open_process_rw_handle(pid).ok_or_else(|| ProcessError::HandleOpenFailed(pid))?;

        // Create device context
        let hdc = self.create_dc(hwnd)?;

        // Store state
        self.state = Some(ProcessState {
            pid,
            hwnd,
            handle,
            hdc,
        });

        Ok(())
    }

    /// Re-initialize the process (same as init, but clearer intent)
    pub fn reinit(&mut self) -> ProcessResult<()> {
        self.init()
    }

    /// Initialize the process by specifying a PID directly
    ///
    /// This bypasses the configured lookup strategy and uses the provided PID directly.
    /// Useful when you know the exact PID (e.g., from task manager).
    ///
    /// # Arguments
    /// * `pid` - The process ID to connect to
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{Process, ProcessConfig};
    ///
    /// let config = ProcessConfig::builder("game.exe").build();
    /// let mut process = Process::new(config);
    ///
    /// // Initialize with known PID
    /// if process.init_with_pid(12345).is_ok() {
    ///     println!("Connected to PID 12345");
    /// }
    /// ```
    pub fn init_with_pid(&mut self, pid: u32) -> ProcessResult<()> {
        // Clean up existing resources first
        if self.state.is_some() {
            self.cleanup()?;
        }

        // Find window for this PID
        let hwnd = self.find_window_for_pid(pid)?;

        // Open process handle
        let handle =
            open_process_rw_handle(pid).ok_or_else(|| ProcessError::HandleOpenFailed(pid))?;

        // Create device context
        let hdc = self.create_dc(hwnd)?;

        // Store state
        self.state = Some(ProcessState {
            pid,
            hwnd,
            handle,
            hdc,
        });

        Ok(())
    }

    // ==================== Cleanup Methods ====================

    /// Clean up all runtime resources
    ///
    /// Closes the process handle and releases the device context.
    /// This is automatically called when the Process is dropped.
    pub fn cleanup(&mut self) -> ProcessResult<()> {
        if let Some(state) = self.state.take() {
            drop(state); // Drop trait will clean up resources
        }
        Ok(())
    }

    /// Reset the process to uninitialized state
    pub fn reset(&mut self) {
        self.state = None;
    }

    // ==================== Internal Helper Methods ====================

    /// Find process based on configuration
    fn find_process(&self) -> ProcessResult<(u32, HWND)> {
        let process_name = &self.config.process_name;

        // If window filter is specified, use filtered search
        if let Some(filter) = &self.config.window_filter {
            self.find_by_name_with_filter(process_name, filter)
        } else {
            self.find_by_name(process_name)
        }
    }

    /// Find process by name only
    fn find_by_name(&self, process_name: &str) -> ProcessResult<(u32, HWND)> {
        let pid = get_process_pid(process_name)
            .ok_or_else(|| ProcessError::ProcessNotFound(process_name.to_string()))?;

        let hwnd = self.find_window_for_pid(pid)?;

        Ok((pid, hwnd))
    }

    /// Find process by name with window filter
    fn find_by_name_with_filter(
        &self,
        process_name: &str,
        filter: &WindowFilter,
    ) -> ProcessResult<(u32, HWND)> {
        use crate::hwnd::get_hwnd_list_by_pid;
        use crate::snapshot::find_pids_by_name;

        // Optimization: If no filter rules, use simple search
        if filter.rules.is_empty() {
            return self.find_by_name(process_name);
        }

        // Step 1: Get ALL PIDs matching the process name
        let all_pids = find_pids_by_name(process_name);
        
        if all_pids.is_empty() {
            return Err(ProcessError::ProcessNotFound(process_name.to_string()));
        }

        // Step 2: Iterate through all processes and their windows
        for pid in &all_pids {
            let all_windows = get_hwnd_list_by_pid(*pid);
            
            // Step 3: Apply filter rules to find matching window
            for hwnd in &all_windows {
                if self.matches_filter(*hwnd, filter) {
                    return Ok((*pid, *hwnd));
                }
            }
        }

        // No window matched the filter in any process
        Err(ProcessError::WindowNotFound(0))
    }

    /// Check if a window matches all filter rules
    fn matches_filter(&self, hwnd: HWND, filter: &WindowFilter) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextW, IsWindowVisible};

        // If no rules, accept all windows
        if filter.rules.is_empty() {
            return true;
        }

        // Helper function to get window title
        let get_title = |hwnd: HWND| -> String {
            let mut buffer = [0u16; 256];
            let length = unsafe { GetWindowTextW(hwnd, &mut buffer) };
            if length == 0 {
                return String::new();
            }
            String::from_utf16(&buffer[..length as usize]).unwrap_or_default()
        };

        let title = get_title(hwnd);
        let is_visible = unsafe { IsWindowVisible(hwnd).as_bool() };

        // Apply all rules - ALL must match (AND logic)
        for rule in &filter.rules {
            let matches = match &rule.criterion {
                FilterCriterion::ByWindowTitle {
                    title_pattern,
                    case_sensitive,
                } => {
                    if *case_sensitive {
                        title == *title_pattern
                    } else {
                        title.to_lowercase().contains(&title_pattern.to_lowercase())
                    }
                }
                FilterCriterion::ByVisibility(visible) => is_visible == *visible,
            };

            // Check if rule type matches expectation
            match rule.rule_type {
                FilterRuleType::Include => {
                    if !matches {
                        return false; // Include rule not satisfied
                    }
                }
                FilterRuleType::Exclude => {
                    if matches {
                        return false; // Exclude rule triggered
                    }
                }
            }
        }

        true // All rules passed
    }

    /// Find window handle for a given PID
    fn find_window_for_pid(&self, pid: u32) -> ProcessResult<HWND> {
        crate::hwnd::get_hwnd_by_pid(pid).ok_or_else(|| ProcessError::WindowNotFound(pid))
    }

    /// Create device context based on DC mode
    fn create_dc(&self, hwnd: HWND) -> ProcessResult<HDC> {
        match self.config.dc_mode {
            DCMode::Desktop => {
                get_desktop_dc().ok_or_else(|| ProcessError::DCNotFound(HWND::default()))
            }
            DCMode::Standard => get_window_dc(hwnd).ok_or_else(|| ProcessError::DCNotFound(hwnd)),
            DCMode::WindowClient => {
                get_window_client_dc(hwnd).ok_or_else(|| ProcessError::DCNotFound(hwnd))
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
