//! Process manager module
//!
//! Provides centralized management of multiple process instances.
//! Similar to a script engine: register configurations, then initialize selectively.

use crate::process::config::ProcessConfig;
use crate::process::Process;
use crate::process::ProcessError;
use crate::process::ProcessResult;
use std::collections::HashMap;

/// Process Manager - Centralized process lifecycle management
///
/// # Architecture
/// The manager follows a three-phase workflow:
/// 1. **Register**: Add process configurations (no initialization yet)
/// 2. **Initialize**: Selectively initialize registered processes
/// 3. **Cleanup**: Release resources when done
///
/// # Usage Pattern
/// ```no_run
/// use win_auto_utils::process::ProcessManager;
///
/// let mut manager = ProcessManager::new();
///
/// // Phase 1: Register configurations
/// manager.register("notepad.exe").ok();
/// manager.register_alias("game", "lf2.exe").ok();
///
/// // Phase 2: Initialize selectively
/// manager.init("notepad.exe").ok();
///
/// // Phase 3: Use the process
/// if let Some(proc) = manager.get("notepad.exe") {
///     println!("Notepad PID: {:?}", proc.pid());
/// }
///
/// // Cleanup happens automatically on drop
/// ```
pub struct ProcessManager {
    /// Registered process instances (key -> Process)
    processes: HashMap<String, Process>,
}

impl ProcessManager {
    // ==================== Constructor ====================
    
    /// Create a new empty process manager
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }
    
    // ==================== Registration Methods ====================
    
    /// Register a process by its executable name (process name becomes the key)
    ///
    /// This is the simplest way to register a process. The process name (e.g., "notepad.exe")
    /// will be used as both the search target and the registration key.
    ///
    /// # Arguments
    /// * `process_name` - The executable name to search for (e.g., "notepad.exe")
    ///
    /// # Returns
    /// * `Ok(())` - Successfully registered
    /// * `Err(ProcessError::DuplicateName)` - Name already registered
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessManager;
    ///
    /// let mut manager = ProcessManager::new();
    /// manager.register("notepad.exe").ok();
    /// // Now you can use "notepad.exe" as the key
    /// manager.init("notepad.exe").ok();
    /// ```
    pub fn register(&mut self, process_name: &str) -> ProcessResult<()> {
        if self.processes.contains_key(process_name) {
            return Err(ProcessError::DuplicateName(process_name.to_string()));
        }
        
        let config = ProcessConfig::new(process_name);
        let process = Process::new(config);
        self.processes.insert(process_name.to_string(), process);
        Ok(())
    }
    
    /// Register a process with a custom alias/key
    ///
    /// Use this when you want to use a friendly name instead of the actual process name.
    ///
    /// # Arguments
    /// * `key` - Your custom identifier (e.g., "game", "editor")
    /// * `process_name` - The executable name to search for (e.g., "lf2.exe")
    ///
    /// # Returns
    /// * `Ok(())` - Successfully registered
    /// * `Err(ProcessError::DuplicateName)` - Key already registered
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessManager;
    ///
    /// let mut manager = ProcessManager::new();
    /// manager.register_alias("game", "lf2.exe").ok();
    /// // Now you can use "game" as the key
    /// manager.init("game").ok();
    /// ```
    pub fn register_alias(&mut self, key: &str, process_name: &str) -> ProcessResult<()> {
        if self.processes.contains_key(key) {
            return Err(ProcessError::DuplicateName(key.to_string()));
        }
        
        let config = ProcessConfig::new(process_name);
        let process = Process::new(config);
        self.processes.insert(key.to_string(), process);
        Ok(())
    }
    
    /// Register a process using a full ProcessConfig
    ///
    /// This allows you to register a process with advanced configuration options
    /// (window filters, DC mode, etc.). The process name from the config will be used as the key.
    ///
    /// # Arguments
    /// * `config` - The complete process configuration
    ///
    /// # Returns
    /// * `Ok(())` - Successfully registered
    /// * `Err(ProcessError::DuplicateName)` - Name already registered
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{ProcessManager, ProcessConfig};
    ///
    /// let mut manager = ProcessManager::new();
    ///
    /// // Register with advanced configuration
    /// let config = ProcessConfig::builder("notepad.exe")
    ///     .set_window_client_mode()
    ///     .exclude_invisible()
    ///     .build();
    /// manager.register_config(config).ok();
    ///
    /// // Initialize using the process name from config
    /// manager.init("notepad.exe").ok();
    /// ```
    pub fn register_config(&mut self, config: ProcessConfig) -> ProcessResult<()> {
        let process_name = config.process_name.clone();
        
        if self.processes.contains_key(&process_name) {
            return Err(ProcessError::DuplicateName(process_name));
        }
        
        let process = Process::new(config);
        self.processes.insert(process_name, process);
        Ok(())
    }
    
    /// Register a process using a full ProcessConfig with a custom alias
    ///
    /// Similar to `register_config()`, but allows you to specify a custom key/name
    /// instead of using the process name from the config.
    ///
    /// # Arguments
    /// * `key` - Your custom identifier (e.g., "game_1", "main_app")
    /// * `config` - The complete process configuration
    ///
    /// # Returns
    /// * `Ok(())` - Successfully registered
    /// * `Err(ProcessError::DuplicateName)` - Key already registered
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::{ProcessManager, ProcessConfig};
    ///
    /// let mut manager = ProcessManager::new();
    ///
    /// // Register with advanced configuration and custom alias
    /// let config = ProcessConfig::builder("game.exe")
    ///     .set_window_client_mode()
    ///     .exclude_invisible()
    ///     .build();
    /// manager.register_config_alias("my_game", config).ok();
    ///
    /// // Initialize using the custom alias
    /// manager.init("my_game").ok();
    /// ```
    pub fn register_config_alias(&mut self, key: &str, config: ProcessConfig) -> ProcessResult<()> {
        if self.processes.contains_key(key) {
            return Err(ProcessError::DuplicateName(key.to_string()));
        }
        
        let process = Process::new(config);
        self.processes.insert(key.to_string(), process);
        Ok(())
    }
    
    /// Unregister a process (cleans up resources first)
    ///
    /// # Arguments
    /// * `key` - The registered process key/name
    ///
    /// # Returns
    /// * `Ok(())` - Successfully unregistered
    /// * `Err(ProcessError::ProcessNotRegistered)` - Key not found
    pub fn unregister(&mut self, key: &str) -> ProcessResult<()> {
        if let Some(mut process) = self.processes.remove(key) {
            process.cleanup()?;
        } else {
            return Err(ProcessError::ProcessNotRegistered(key.to_string()));
        }
        Ok(())
    }
    
    /// Check if a process is registered
    pub fn is_registered(&self, key: &str) -> bool {
        self.processes.contains_key(key)
    }

    // ==================== Initialization Methods ====================
    
    /// Initialize a registered process using its configured lookup strategy
    ///
    /// # Arguments
    /// * `key` - The registered process key
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessManager;
    ///
    /// let mut manager = ProcessManager::new();
    /// manager.register("notepad.exe").ok();
    ///
    /// // Later, when notepad is running...
    /// manager.init("notepad.exe").ok();
    /// ```
    pub fn init(&mut self, key: &str) -> ProcessResult<()> {
        if let Some(proc) = self.processes.get_mut(key) {
            proc.init()?;
        } else {
            return Err(ProcessError::ProcessNotRegistered(key.to_string()));
        }
        Ok(())
    }
    
    /// Initialize a registered process by specifying a PID directly
    ///
    /// This bypasses the configured lookup strategy and uses the provided PID.
    /// Useful for distinguishing multiple instances of the same process.
    ///
    /// # Arguments
    /// * `key` - The registered process key
    /// * `pid` - The process ID to connect to
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessManager;
    ///
    /// let mut manager = ProcessManager::new();
    /// manager.register_alias("notepad_2", "notepad.exe").ok();
    ///
    /// // Initialize with specific PID (e.g., from Task Manager)
    /// manager.init_with_pid("notepad_2", 20908).ok();
    /// ```
    pub fn init_with_pid(&mut self, key: &str, pid: u32) -> ProcessResult<()> {
        if let Some(proc) = self.processes.get_mut(key) {
            proc.init_with_pid(pid)?;
        } else {
            return Err(ProcessError::ProcessNotRegistered(key.to_string()));
        }
        Ok(())
    }
    
    /// Re-initialize a registered process
    pub fn reinit(&mut self, key: &str) -> ProcessResult<()> {
        if let Some(proc) = self.processes.get_mut(key) {
            proc.reinit()?;
        } else {
            return Err(ProcessError::ProcessNotRegistered(key.to_string()));
        }
        Ok(())
    }
    
    // ==================== Cleanup Methods ====================
    
    /// Clean up a specific process's runtime resources
    pub fn cleanup(&mut self, key: &str) -> ProcessResult<()> {
        if let Some(proc) = self.processes.get_mut(key) {
            proc.cleanup()?;
        } else {
            return Err(ProcessError::ProcessNotRegistered(key.to_string()));
        }
        Ok(())
    }
    
    /// Clean up all registered processes
    pub fn cleanup_all(&mut self) -> ProcessResult<()> {
        for proc in self.processes.values_mut() {
            proc.cleanup()?;
        }
        Ok(())
    }
    
    // ==================== Query Methods ====================
    
    /// Get an immutable reference to a registered process
    pub fn get(&self, key: &str) -> Option<&Process> {
        self.processes.get(key)
    }
    
    /// List all registered process keys
    pub fn list_processes(&self) -> Vec<String> {
        self.processes.keys().cloned().collect()
    }
    
    /// Get all valid (initialized) processes
    pub fn get_all_valid(&self) -> Vec<(String, bool)> {
        self.processes.iter()
            .filter(|(_, p)| p.is_valid())
            .map(|(name, proc)| (name.clone(), proc.is_valid()))
            .collect()
    }

    /// Get all registered process names (alias for list_processes)
    pub fn get_all_registered(&self) -> Vec<String> {
        self.list_processes()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        let _ = self.cleanup_all();
    }
}
