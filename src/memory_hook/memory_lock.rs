//! Memory Lock implementation
//!
//! Provides functionality to continuously monitor and restore memory values.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HANDLE;
use crate::memory::{read_memory_bytes, write_memory_bytes, MemoryError};
use crate::memory_resolver::MemoryAddress;
use super::utils::SendableHandle;

/// Source of the target address for locking
#[derive(Clone)]
pub enum AddressSource {
    /// Static address (fixed value)
    Static(usize),
    /// Dynamic address resolved from MemoryAddress pattern
    Dynamic(MemoryAddress),
}

impl AddressSource {
    /// Resolve the actual address at runtime
    ///
    /// For static addresses, returns the address directly.
    /// For dynamic addresses, resolves through the MemoryAddress resolver.
    pub fn resolve(&self, handle: HANDLE, pid: u32) -> Result<usize, MemoryError> {
        match self {
            AddressSource::Static(addr) => Ok(*addr),
            AddressSource::Dynamic(mem_addr) => {
                mem_addr.resolve_address(handle, pid)
                    .map_err(|e| MemoryError::ReadFailed(
                        format!("Failed to resolve dynamic address: {}", e)
                    ))
            }
        }
    }
}

/// Memory lock controller for continuous value monitoring and restoration
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_hook::MemoryLock;
///
/// let mut lock = MemoryLock::builder()
///     .handle(handle)
///     .address(0x7FF6A1B2C3D4)
///     .value(100u32)
///     .build()?;
///
/// lock.start()?; // Start monitoring
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct MemoryLock {
    handle: Option<SendableHandle>,
    address_source: Option<AddressSource>,
    pid: Option<u32>,
    size: usize,
    locked_value: Vec<u8>,
    scan_interval: Duration,
    stop_flag: Arc<AtomicBool>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl MemoryLock {
    /// Create a new builder with all parameters unset
    pub fn builder() -> MemoryLockBuilder {
        MemoryLockBuilder::new()
    }

    /// Set the scan interval for checking memory changes
    ///
    /// # Arguments
    /// * `interval` - How often to check and restore the value
    ///
    /// # Note
    /// Shorter intervals provide tighter locking but consume more CPU
    pub fn set_scan_interval(&mut self, interval: Duration) {
        self.scan_interval = interval;
    }
    
    /// Lock a specific value at the target address
    ///
    /// # Arguments
    /// * `value` - The value to lock (will be serialized to bytes)
    ///
    /// # Type Support
    /// Supports any type that implements `AsRef<[u8]>` or can be converted to bytes
    ///
    /// # Example
    /// ```no_run
    /// # use win_auto_utils::memory_hook::MemoryLock;
    /// # use windows::Win32::Foundation::HANDLE;
    /// # let handle = HANDLE::default();
    /// let mut lock = MemoryLock::new(handle, 0x1000);
    /// lock.lock_value(100u32).unwrap(); // Lock as u32
    /// lock.lock_value(3.14f32).unwrap(); // Lock as f32
    /// ```
    pub fn lock_value<T: Copy + AsBytes>(&mut self, value: T) -> Result<(), MemoryError> {
        // Validate required parameters
        let handle = self.handle.as_ref().ok_or_else(|| {
            MemoryError::WriteFailed("handle must be set. Call .handle(handle) before lock_value().".to_string())
        })?;
        
        let address_source = self.address_source.as_ref().ok_or_else(|| {
            MemoryError::WriteFailed("address must be set. Call .address(addr) or .address_from_resolver(addr) before lock_value().".to_string())
        })?;
        
        let pid = self.pid.ok_or_else(|| {
            MemoryError::WriteFailed("PID must be set for address resolution. Call .pid(pid) before lock_value().".to_string())
        })?;
        
        // Resolve the actual address (tolerate errors if configured)
        let address = match address_source.resolve(handle.0, pid) {
            Ok(addr) => addr,
            Err(e) => {
                // For dynamic addresses, we can tolerate initial resolution failure
                // The background thread will retry on each cycle
                #[cfg(debug_assertions)]
                println!("[MemoryLock] ⚠ Initial address resolution failed ({}), background thread will retry", e);
                
                // Start monitoring anyway - the thread will handle retries
                let bytes = value.as_bytes();
                self.size = bytes.len();
                self.locked_value = bytes.to_vec();
                return self.start_monitoring(0);  // Use dummy address, thread will re-resolve
            }
        };
        
        let bytes = value.as_bytes();
        self.size = bytes.len();
        self.locked_value = bytes.to_vec();
        
        // Write the initial value
        write_memory_bytes(handle.0, address, &self.locked_value)?;
        
        // Start the monitoring thread
        self.start_monitoring(address)?;
        
        Ok(())
    }
    
    /// Lock raw bytes at the target address
    ///
    /// # Arguments
    /// * `bytes` - Raw byte sequence to lock
    pub fn lock_bytes(&mut self, bytes: &[u8]) -> Result<(), MemoryError> {
        // Validate required parameters
        let handle = self.handle.as_ref().ok_or_else(|| {
            MemoryError::WriteFailed("handle must be set. Call .handle(handle) before lock_bytes().".to_string())
        })?;
        
        let address_source = self.address_source.as_ref().ok_or_else(|| {
            MemoryError::WriteFailed("address must be set. Call .address(addr) or .address_from_resolver(addr) before lock_bytes().".to_string())
        })?;
        
        let pid = self.pid.ok_or_else(|| {
            MemoryError::WriteFailed("PID must be set for address resolution. Call .pid(pid) before lock_bytes().".to_string())
        })?;
        
        // Resolve the actual address (tolerate errors if configured)
        let address = match address_source.resolve(handle.0, pid) {
            Ok(addr) => addr,
            Err(e) => {
                // For dynamic addresses, we can tolerate initial resolution failure
                // The background thread will retry on each cycle
                #[cfg(debug_assertions)]
                println!("[MemoryLock] ⚠ Initial address resolution failed ({}), background thread will retry", e);
                
                // Start monitoring anyway - the thread will handle retries
                self.size = bytes.len();
                self.locked_value = bytes.to_vec();
                return self.start_monitoring(0);  // Use dummy address, thread will re-resolve
            }
        };
        
        self.size = bytes.len();
        self.locked_value = bytes.to_vec();
        
        // Write the initial value
        write_memory_bytes(handle.0, address, &self.locked_value)?;
        
        // Start the monitoring thread
        self.start_monitoring(address)?;
        
        Ok(())
    }
    
    /// Stop locking and restore normal operation
    pub fn unlock(&mut self) -> Result<(), MemoryError> {
        if let Some(thread) = self.worker_thread.take() {
            // Signal the thread to stop
            self.stop_flag.store(true, Ordering::Relaxed);
            
            // Wait for the thread to finish
            if let Err(e) = thread.join() {
                return Err(MemoryError::ReadFailed(
                    format!("Failed to join worker thread: {:?}", e)
                ));
            }
        }
        
        Ok(())
    }

    /// Reset the lock to its initial state (safe for program restart)
    ///
    /// This method safely cleans up all internal state and stops monitoring,
    /// allowing the lock to be reused after the target program restarts.
    ///
    /// # What it does:
    /// 1. If locked, unlocks first (stops monitoring thread)
    /// 2. Clears cached data (locked_value, address, handle)
    /// 3. Resets all flags to initial state
    ///
    /// # Safety
    /// - Safe to call multiple times (idempotent)
    /// - Safe to call even if never locked
    /// - After reset, the object can be used as if newly created
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::MemoryLock;
    ///
    /// // First usage
    /// let mut lock = MemoryLock::builder()
    ///     .handle(handle)
    ///     .address(addr)
    ///     .value(100u32)
    ///     .build()?;
    /// lock.start()?;
    /// lock.stop()?;
    ///
    /// // Program restarted - reset state
    /// lock.reset();
    ///
    /// // Second usage (no side effects)
    /// let mut lock = MemoryLock::builder()
    ///     .handle(new_handle)
    ///     .address(new_addr)
    ///     .value(100u32)
    ///     .build()?;
    /// lock.start()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn reset(&mut self) {
        // If still locked, unlock first (this will stop monitoring thread)
        if self.is_locked() {
            let _ = self.unlock();
        }

        // Clear all cached state
        self.handle = None;
        self.pid = None;
        self.address_source = None;
        self.size = 0;
        self.locked_value.clear();
        self.scan_interval = Duration::from_millis(10); // Reset to default
        self.stop_flag.store(false, Ordering::Relaxed);
        self.worker_thread = None;
    }

    /// Check if the lock is currently active
    pub fn is_locked(&self) -> bool {
        self.worker_thread.is_some()
    }

    /// Get the locked value as bytes
    pub fn get_locked_value(&self) -> &[u8] {
        &self.locked_value
    }
    
    /// Start the background monitoring thread
    fn start_monitoring(&mut self, initial_address: usize) -> Result<(), MemoryError> {
        // If already locked, stop first
        if self.worker_thread.is_some() {
            self.unlock()?;
        }
        
        // Reset stop flag
        self.stop_flag.store(false, Ordering::Relaxed);
        
        let handle = self.handle.as_ref().unwrap();
        let address_source = self.address_source.clone().unwrap();
        let pid = self.pid.unwrap();
        let handle_int = handle.0 .0 as isize;  // Convert HANDLE to integer
        let locked_value = self.locked_value.clone();
        let size = self.size;
        let interval = self.scan_interval;
        let stop_flag: Arc<AtomicBool> = Arc::clone(&self.stop_flag);
        
        // Convert integer back to HANDLE inside the thread
        let thread_handle = thread::spawn(move || {
            let handle = HANDLE(handle_int as *mut std::ffi::c_void);
            let mut current_address = initial_address;
            
            while !stop_flag.load(Ordering::Relaxed) {
                // For dynamic addresses, re-resolve on each cycle
                if let AddressSource::Dynamic(_) = address_source {
                    match address_source.resolve(handle, pid) {
                        Ok(new_addr) => {
                            current_address = new_addr;
                        }
                        Err(_) => {
                            // Resolution failed, wait and retry
                            thread::sleep(interval);
                            continue;
                        }
                    }
                }
                
                // Read current value
                match read_memory_bytes(handle, current_address, size) {
                    Ok(current_value) => {
                        // Compare with locked value
                        if current_value != locked_value {
                            // Restore the locked value
                            let _ = write_memory_bytes(handle, current_address, &locked_value);
                        }
                    }
                    Err(_) => {
                        // If we can't read, wait and try again
                        thread::sleep(interval);
                        continue;
                    }
                }
                
                // Wait before next check
                thread::sleep(interval);
            }
        });
        
        self.worker_thread = Some(thread_handle);
        Ok(())
    }
}

impl Drop for MemoryLock {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

/// Trait for converting types to byte slices
pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

// Implement AsBytes for common types
macro_rules! impl_as_bytes {
    ($($t:ty),*) => {
        $(
            impl AsBytes for $t {
                fn as_bytes(&self) -> &[u8] {
                    unsafe {
                        std::slice::from_raw_parts(
                            self as *const $t as *const u8,
                            std::mem::size_of::<$t>()
                        )
                    }
                }
            }
        )*
    };
}

impl_as_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

/// Builder for MemoryLock configuration
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_hook::MemoryLock;
///
/// let mut lock = MemoryLock::builder()
///     .handle(handle)
///     .address(0x7FF6A1B2C3D4)
///     .value(100u32)
///     .build()?;
///
/// lock.start()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct MemoryLockBuilder {
    handle: Option<HANDLE>,
    pid: Option<u32>,
    address_source: Option<AddressSource>,
    locked_value: Option<Vec<u8>>,
    scan_interval: Duration,
}

impl MemoryLockBuilder {
    /// Create a new builder with all parameters unset
    pub fn new() -> Self {
        Self {
            handle: None,
            pid: None,
            address_source: None,
            locked_value: None,
            scan_interval: Duration::from_millis(10), // Default 10ms
        }
    }

    /// Set process handle (required for locking)
    pub fn handle(mut self, handle: HANDLE) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set target process PID (required for dynamic address resolution)
    pub fn pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Set static target memory address (required for locking)
    pub fn address(mut self, addr: usize) -> Self {
        self.address_source = Some(AddressSource::Static(addr));
        self
    }

    /// Set dynamic target address using MemoryAddress pattern (required for locking)
    ///
    /// The address will be resolved on each lock cycle, allowing the lock to adapt
    /// to changing addresses (e.g., when game objects are recreated).
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::memory_hook::MemoryLock;
    /// use win_auto_utils::memory_resolver::MemoryAddress;
    ///
    /// let addr = MemoryAddress::new_x86("lf2.exe+58C94->308")?;
    /// let mut lock = MemoryLock::builder()
    ///     .handle(handle)
    ///     .pid(pid)
    ///     .address_from_resolver(addr)
    ///     .value(500u32)
    ///     .build()?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn address_from_resolver(mut self, mem_addr: MemoryAddress) -> Self {
        self.address_source = Some(AddressSource::Dynamic(mem_addr));
        self
    }

    /// Set the value to lock (supports any type implementing AsBytes)
    pub fn value<T: Copy + AsBytes>(mut self, val: T) -> Self {
        self.locked_value = Some(val.as_bytes().to_vec());
        self
    }

    /// Set raw bytes to lock
    pub fn bytes(mut self, data: Vec<u8>) -> Self {
        self.locked_value = Some(data);
        self
    }

    /// Set scan interval in milliseconds
    pub fn scan_interval_ms(mut self, ms: u64) -> Self {
        self.scan_interval = Duration::from_millis(ms);
        self
    }

    /// Set scan interval as Duration
    pub fn scan_interval(mut self, interval: Duration) -> Self {
        self.scan_interval = interval;
        self
    }

    /// Build the MemoryLock with configured settings
    ///
    /// # Validation
    /// This method validates that all required parameters are set:
    /// - `handle`: Process handle
    /// - `address`: Target memory address
    /// - `locked_value`: Value or bytes to lock
    ///
    /// # Errors
    /// Returns error if any required parameter is missing.
    pub fn build(self) -> Result<MemoryLock, MemoryError> {
        // Validate required parameters
        let handle = self.handle.ok_or_else(|| {
            MemoryError::WriteFailed(
                "handle must be set. Call .handle(handle) before build().".to_string()
            )
        })?;

        let address_source = self.address_source.ok_or_else(|| {
            MemoryError::WriteFailed(
                "address must be set. Call .address(addr) or .address_from_resolver(addr) before build().".to_string()
            )
        })?;

        let locked_value = self.locked_value.ok_or_else(|| {
            MemoryError::WriteFailed(
                "locked_value must be set. Call .value(val) or .bytes(data) before build().".to_string()
            )
        })?;

        let size = locked_value.len();

        // For dynamic addresses, PID is required
        if matches!(address_source, AddressSource::Dynamic(_)) && self.pid.is_none() {
            return Err(MemoryError::WriteFailed(
                "PID must be set when using dynamic address resolution. Call .pid(pid) before build().".to_string()
            ));
        }

        Ok(MemoryLock {
            handle: Some(SendableHandle(handle)),
            pid: self.pid,
            address_source: Some(address_source),
            size,
            locked_value,
            scan_interval: self.scan_interval,
            stop_flag: Arc::new(AtomicBool::new(false)),
            worker_thread: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_as_bytes_u32() {
        let value: u32 = 0x12345678;
        let bytes = value.as_bytes();
        
        assert_eq!(bytes.len(), 4);
        // Little-endian on x86/x64
        assert_eq!(bytes[0], 0x78);
        assert_eq!(bytes[3], 0x12);
    }
    
    #[test]
    fn test_builder_clone_support() {
        let builder = MemoryLock::builder()
            .address(0x1000)
            .value(100u32)
            .scan_interval_ms(5);
        
        // Clone and bind different handles
        let lock1 = builder.clone()
            .handle(HANDLE::default())
            .build();
        
        let lock2 = builder.clone()
            .handle(HANDLE::default())
            .build();
        
        assert!(lock1.is_ok());
        assert!(lock2.is_ok());
    }
    
    #[test]
    fn test_builder_validation_missing_handle() {
        let result = MemoryLock::builder()
            .address(0x1000)
            .value(100u32)
            .build();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("handle must be set"));
        }
    }
    
    #[test]
    fn test_builder_validation_missing_address() {
        let result = MemoryLock::builder()
            .handle(HANDLE::default())
            .value(100u32)
            .build();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("address must be set"));
        }
    }
    
    #[test]
    fn test_builder_validation_missing_value() {
        let result = MemoryLock::builder()
            .handle(HANDLE::default())
            .address(0x1000)
            .build();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("locked_value must be set"));
        }
    }
}
