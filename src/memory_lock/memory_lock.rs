//! Memory Lock implementation
//!
//! Provides functionality to continuously monitor and restore memory values.

use crate::memory::{read_memory_bytes, write_memory_bytes, MemoryError};
use crate::memory_resolver::{MemoryAddress, AddressSource};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HANDLE;

/// Trait to convert types to bytes for memory writing
pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for u8 {
    fn as_bytes(&self) -> &[u8] {
        std::slice::from_ref(self)
    }
}

impl AsBytes for u16 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const u16 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<u16>()) }
    }
}

impl AsBytes for u32 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const u32 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<u32>()) }
    }
}

impl AsBytes for u64 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const u64 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<u64>()) }
    }
}

impl AsBytes for f32 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const f32 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<f32>()) }
    }
}

impl AsBytes for f64 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const f64 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<f64>()) }
    }
}

impl AsBytes for i8 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const i8 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<i8>()) }
    }
}

impl AsBytes for i16 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const i16 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<i16>()) }
    }
}

impl AsBytes for i32 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const i32 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<i32>()) }
    }
}

impl AsBytes for i64 {
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const i64 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<i64>()) }
    }
}

/// Builder for MemoryLock
pub struct MemoryLockBuilder {
    handle: Option<HANDLE>,
    address_source: Option<AddressSource>,
    pid: Option<u32>,
    locked_value: Option<Vec<u8>>,
    scan_interval: Duration,
}

impl MemoryLockBuilder {
    /// Create a new builder with all parameters unset
    pub fn new() -> Self {
        Self {
            handle: None,
            address_source: None,
            pid: None,
            locked_value: None,
            scan_interval: Duration::from_millis(100), // Default interval
        }
    }

    /// Set the process handle
    pub fn handle(mut self, handle: HANDLE) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Set a static address directly
    pub fn address(mut self, address: usize) -> Self {
        // Convert static address to MemoryAddress::Absolute
        let mem_addr = MemoryAddress {
            base: crate::memory_resolver::AddressBase::Absolute(address),
            operations: vec![],
            pointer_size: crate::memory_resolver::PointerSize::default_architecture(),
        };
        self.address_source = Some(AddressSource::from_static(mem_addr));
        self
    }

    /// Set the dynamic address source (MemoryAddress pattern)
    pub fn address_from_resolver(mut self, addr: MemoryAddress) -> Self {
        self.address_source = Some(AddressSource::from_static(addr));
        self
    }

    /// Set an AOB pattern for dynamic address resolution
    pub fn address_from_aob(mut self, pattern: &str) -> Result<Self, String> {
        self.address_source = Some(AddressSource::from_aob_scan(pattern)?);
        Ok(self)
    }

    /// Set the process ID (required for dynamic addresses)
    pub fn pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Set the value to lock (generic type)
    pub fn value<T: AsBytes>(mut self, val: T) -> Self {
        self.locked_value = Some(val.as_bytes().to_vec());
        self
    }

    /// Set the raw bytes to lock
    pub fn bytes(mut self, data: Vec<u8>) -> Self {
        self.locked_value = Some(data);
        self
    }

    /// Set the scan interval
    pub fn scan_interval(mut self, interval: Duration) -> Self {
        self.scan_interval = interval;
        self
    }

    /// Build the MemoryLock instance
    pub fn build(self) -> Result<MemoryLock, MemoryError> {
        // Validate required parameters
        let handle = self.handle.ok_or_else(|| {
            MemoryError::WriteFailed(
                "handle must be set. Call .handle(handle) before build().".to_string(),
            )
        })?;

        let address_source = self.address_source.ok_or_else(|| {
            MemoryError::WriteFailed(
                "address must be set. Call .address(addr), .address_from_resolver(addr), or .address_from_aob(pattern) before build().".to_string()
            )
        })?;

        let locked_value = self.locked_value.ok_or_else(|| {
            MemoryError::WriteFailed(
                "locked_value must be set. Call .value(val) or .bytes(data) before build()."
                    .to_string(),
            )
        })?;

        let size = locked_value.len();

        // For module-based or AOB addresses, PID is recommended
        if self.pid.is_none() {
            // Check if we need PID
            let needs_pid = match &address_source {
                AddressSource::Static(addr) => {
                    matches!(addr.base, crate::memory_resolver::AddressBase::Module { .. })
                }
                AddressSource::AobScan { .. } => true,
            };
            
            if needs_pid {
                return Err(MemoryError::WriteFailed(
                    "PID must be set when using module-relative or AOB addresses. Call .pid(pid) before build().".to_string()
                ));
            }
        }

        Ok(MemoryLock {
            handle: Some(handle),  // Store HANDLE directly
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

/// Memory lock controller for continuous value monitoring and restoration
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory_lock::MemoryLock;
/// use windows::Win32::Foundation::HANDLE;
///
/// let handle = HANDLE::default(); // Replace with actual handle
/// let mut lock = MemoryLock::builder()
///     .handle(handle)
///     .address(0x7FF6A1B2C3D4)
///     .value(100u32)
///     .build()?;
///
/// lock.lock_value(100u32)?; // Start locking
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct MemoryLock {
    handle: Option<HANDLE>,  // Store HANDLE directly for type consistency
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
    /// # use win_auto_utils::memory_lock::MemoryLock;
    /// # use windows::Win32::Foundation::HANDLE;
    /// # let handle = HANDLE::default();
    /// let mut lock = MemoryLock::builder()
    ///     .handle(handle)
    ///     .address(0x1000)
    ///     .value(100u32)
    ///     .build()?;
    /// lock.lock_value(100u32)?; // Start locking with the value
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn lock_value<T: Copy + AsBytes>(&mut self, value: T) -> Result<(), MemoryError> {
        // Validate required parameters
        let handle = self.handle.ok_or_else(|| {
            MemoryError::WriteFailed(
                "handle must be set. Call .handle(handle) before lock_value().".to_string(),
            )
        })?;

        let address_source = self.address_source.as_ref().ok_or_else(|| {
            MemoryError::WriteFailed("address must be set. Call .address(addr), .address_from_resolver(addr), or .address_from_aob(pattern) before lock_value().".to_string())
        })?;

        let pid = self.pid.unwrap_or(0);

        // Resolve the actual address (tolerate errors if configured)
        let address = match address_source.resolve(handle, pid) {
            Ok(addr) => addr,
            Err(_) => {
                // For dynamic addresses, we can tolerate initial resolution failure
                // The background thread will retry on each cycle

                // Start monitoring anyway - the thread will handle retries
                let bytes = value.as_bytes();
                self.size = bytes.len();
                self.locked_value = bytes.to_vec();
                return self.start_monitoring(); // Thread will re-resolve address
            }
        };

        let bytes = value.as_bytes();
        self.size = bytes.len();
        self.locked_value = bytes.to_vec();

        // Write the initial value
        write_memory_bytes(handle, address, &self.locked_value)?;

        // Start the monitoring thread
        self.start_monitoring()?;

        Ok(())
    }

    /// Lock raw bytes at the target address
    ///
    /// # Arguments
    /// * `bytes` - Raw byte sequence to lock
    pub fn lock_bytes(&mut self, bytes: &[u8]) -> Result<(), MemoryError> {
        // Validate required parameters
        let handle = self.handle.ok_or_else(|| {
            MemoryError::WriteFailed(
                "handle must be set. Call .handle(handle) before lock_bytes().".to_string(),
            )
        })?;

        let address_source = self.address_source.as_ref().ok_or_else(|| {
            MemoryError::WriteFailed("address must be set. Call .address(addr), .address_from_resolver(addr), or .address_from_aob(pattern) before lock_bytes().".to_string())
        })?;

        let pid = self.pid.unwrap_or(0);

        // Resolve the actual address (tolerate errors if configured)
        let address = match address_source.resolve(handle, pid) {
            Ok(addr) => addr,
            Err(_) => {
                // For dynamic addresses, we can tolerate initial resolution failure
                // The background thread will retry on each cycle

                // Start monitoring anyway - the thread will handle retries
                self.size = bytes.len();
                self.locked_value = bytes.to_vec();
                return self.start_monitoring(); // Thread will re-resolve address
            }
        };

        self.size = bytes.len();
        self.locked_value = bytes.to_vec();

        // Write the initial value
        write_memory_bytes(handle, address, &self.locked_value)?;

        // Start the monitoring thread
        self.start_monitoring()?;

        Ok(())
    }

    /// Start the monitoring thread
    fn start_monitoring(&mut self) -> Result<(), MemoryError> {
        let handle = self.handle.unwrap();
        let address_source = self.address_source.clone().unwrap();
        let pid = self.pid.unwrap_or(0);
        let size = self.size;
        let locked_value = self.locked_value.clone();
        let scan_interval = self.scan_interval;
        let stop_flag = self.stop_flag.clone();

        // Convert HANDLE to usize for thread-safe passing
        let handle_usize = handle.0 as usize;

        self.worker_thread = Some(thread::spawn(move || {
            // Convert usize back to HANDLE inside the thread
            let handle = HANDLE(handle_usize as *mut std::ffi::c_void);

            while !stop_flag.load(Ordering::SeqCst) {
                thread::sleep(scan_interval);

                // Resolve the actual address (tolerate errors if configured)
                let address = match address_source.resolve(handle, pid) {
                    Ok(addr) => addr,
                    Err(_) => continue,
                };

                // Read the current value
                let current_value = match read_memory_bytes(handle, address, size) {
                    Ok(val) => val,
                    Err(_) => continue,
                };

                // Compare and write if necessary
                if current_value != locked_value {
                    if let Err(_) = write_memory_bytes(handle, address, &locked_value) {
                        continue;
                    }
                }
            }
        }));

        Ok(())
    }

    /// Stop the monitoring thread
    ///
    /// This method gracefully stops the background monitoring thread.
    /// After calling this, you can call `lock_value()` or `lock_bytes()` again
    /// to restart monitoring with new parameters.
    ///
    /// # Example
    /// ```no_run
    /// # use win_auto_utils::memory_lock::MemoryLock;
    /// # use windows::Win32::Foundation::HANDLE;
    /// # let handle = HANDLE::default();
    /// let mut lock = MemoryLock::builder()
    ///     .handle(handle)
    ///     .address(0x1000)
    ///     .value(100u32)
    ///     .build()?;
    /// lock.lock_value(100u32)?; // Start locking
    /// 
    /// // ... some time later ...
    /// lock.stop(); // Stop the monitoring thread
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn stop(&mut self) {
        // Signal the worker thread to stop
        self.stop_flag.store(true, Ordering::SeqCst);
        
        // Wait for the thread to finish (with timeout to avoid hanging)
        if let Some(handle) = self.worker_thread.take() {
            // Give the thread up to 1 second to finish
            let _ = handle.join();
        }
    }
}

impl Drop for MemoryLock {
    fn drop(&mut self) {
        self.stop();
    }
}
