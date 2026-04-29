//! Memory operations module
//!
//! Provides functions to read and write memory in remote processes.
//! Supports reading/writing primitive types and byte arrays at specific addresses.
//!
//! # Important Notes
//! - Requires a process handle with appropriate access rights (obtained via `handle` module)
//! - Reading/writing invalid addresses will fail
//! - Always ensure the target address is valid before operations
//! - Use `PROCESS_VM_READ` for reading, `PROCESS_VM_WRITE` for writing
//!
//! # Feature Flag
//! Enable with: `--features "memory"`
//!
//! # Quick Start
//!
//! ## Read a Value from Memory
//! ```no_run
//! use win_auto_utils::memory::read_memory_i32;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let address = 0x7FF6A1B2C3D4;
//!     
//!     // Get process handle using handle module
//!     let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
//!     let handle = open_process_handle(pid, desired_access)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Read memory
//!     let value = read_memory_i32(handle, address)?;
//!     println!("Read value: {}", value);
//!     
//!     // Close handle when done
//!     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Write Bytes to Memory
//! ```no_run
//! use win_auto_utils::memory::write_memory_bytes;
//! use win_auto_utils::handle::open_process_handle;
//! use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pid = 12345;
//!     let address = 0x7FF6A1B2C3D4;
//!     let data = vec![0x90, 0x90, 0x90]; // NOP instructions
//!     
//!     // Get process handle
//!     let desired_access = PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
//!     let handle = open_process_handle(pid, desired_access)
//!         .ok_or("Failed to open process")?;
//!     
//!     // Write memory
//!     write_memory_bytes(handle, address, &data)?;
//!     println!("Successfully wrote {} bytes", data.len());
//!     
//!     // Close handle
//!     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
//!     
//!     Ok(())
//! }
//! ```

use windows::{
    Win32::Foundation::HANDLE,
    Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
};

/// Error type for memory operations
#[derive(Debug)]
pub enum MemoryError {
    /// Failed to read memory
    ReadFailed(String),
    /// Failed to write memory
    WriteFailed(String),
    /// Invalid address
    InvalidAddress(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::ReadFailed(msg) => write!(f, "Failed to read memory: {}", msg),
            MemoryError::WriteFailed(msg) => write!(f, "Failed to write memory: {}", msg),
            MemoryError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
        }
    }
}

impl std::error::Error for MemoryError {}

/// Read bytes from a remote process memory
///
/// Reads a specified number of bytes from the target process memory at the given address.
///
/// # Arguments
/// * `handle` - Handle to the target process (must have PROCESS_VM_READ)
/// * `address` - The memory address to read from
/// * `size` - Number of bytes to read
///
/// # Returns
/// * `Ok(Vec<u8>)` - The bytes read from memory
/// * `Err(MemoryError)` - If the read operation failed
///
/// # Safety
/// The address must be valid and accessible in the target process.
/// Reading invalid addresses may cause the operation to fail.
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory::read_memory_bytes;
/// use win_auto_utils::handle::open_process_handle;
/// use windows::Win32::System::Threading::PROCESS_VM_READ;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let handle = open_process_handle(12345, PROCESS_VM_READ).ok_or("Failed to open process")?;
///     let bytes = read_memory_bytes(handle, 0x7FF6A1B2C3D4, 16)?;
///     println!("Read bytes: {:02X?}", bytes);
///     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
///     Ok(())
/// }
/// ```
pub fn read_memory_bytes(handle: HANDLE, address: usize, size: usize) -> Result<Vec<u8>, MemoryError> {
    if address == 0 {
        return Err(MemoryError::InvalidAddress("Address cannot be zero".to_string()));
    }
    
    let mut buffer = vec![0u8; size];
    let mut bytes_read = 0usize;
    
    unsafe {
        match ReadProcessMemory(
            handle,
            address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size,
            Some(&mut bytes_read),
        ) {
            Ok(_) if bytes_read == size => Ok(buffer),
            Ok(_) => Err(MemoryError::ReadFailed(format!(
                "Incomplete read: expected {} bytes, got {}",
                size, bytes_read
            ))),
            Err(e) => Err(MemoryError::ReadFailed(format!(
                "Failed to read {} bytes from address 0x{:X}. Windows error: {:?}",
                size, address, e
            ))),
        }
    }
}

/// Write bytes to a remote process memory
///
/// Writes the provided byte array to the target process memory at the given address.
///
/// # Arguments
/// * `handle` - Handle to the target process (must have PROCESS_VM_WRITE and PROCESS_VM_OPERATION)
/// * `address` - The memory address to write to
/// * `data` - The bytes to write
///
/// # Returns
/// * `Ok(())` - If the write operation succeeded
/// * `Err(MemoryError)` - If the write operation failed
///
/// # Safety
/// The address must be valid and writable in the target process.
/// Writing to invalid or protected addresses may cause the operation to fail.
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory::write_memory_bytes;
/// use win_auto_utils::handle::open_process_handle;
/// use windows::Win32::System::Threading::{PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let desired_access = PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
///     let handle = open_process_handle(12345, desired_access).ok_or("Failed to open process")?;
///     let data = vec![0x90, 0x90, 0x90]; // NOP sled
///     write_memory_bytes(handle, 0x7FF6A1B2C3D4, &data)?;
///     println!("Successfully wrote {} bytes", data.len());
///     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
///     Ok(())
/// }
/// ```
pub fn write_memory_bytes(handle: HANDLE, address: usize, data: &[u8]) -> Result<(), MemoryError> {
    if address == 0 {
        return Err(MemoryError::InvalidAddress("Address cannot be zero".to_string()));
    }
    
    let mut bytes_written = 0usize;
    
    unsafe {
        match WriteProcessMemory(
            handle,
            address as *mut _,
            data.as_ptr() as *const _,
            data.len(),
            Some(&mut bytes_written),
        ) {
            Ok(_) if bytes_written == data.len() => Ok(()),
            Ok(_) => Err(MemoryError::WriteFailed(format!(
                "Incomplete write: expected {} bytes, wrote {}",
                data.len(), bytes_written
            ))),
            Err(e) => Err(MemoryError::WriteFailed(format!(
                "Failed to write {} bytes to address 0x{:X}. Windows error: {:?}",
                data.len(), address, e
            ))),
        }
    }
}

/// Read a primitive value from remote process memory
///
/// Reads a value of type T from the target process memory at the given address.
/// T must be a primitive type that can be safely read as bytes.
///
/// # Type Parameters
/// * `T` - The type to read (must implement Copy + Sized)
///
/// # Arguments
/// * `handle` - Handle to the target process
/// * `address` - The memory address to read from
///
/// # Returns
/// * `Ok(T)` - The value read from memory
/// * `Err(MemoryError)` - If the read operation failed
///
/// # Supported Types
/// Common primitive types: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`,
/// `f32`, `f64`, `usize`, `isize`
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory::read_memory_t;
/// use win_auto_utils::handle::open_process_handle;
/// use windows::Win32::System::Threading::PROCESS_VM_READ;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let handle = open_process_handle(12345, PROCESS_VM_READ).ok_or("Failed to open process")?;
///     
///     // Read an i32 value
///     let health: i32 = read_memory_t(handle, 0x7FF6A1B2C3D4)?;
///     println!("Health: {}", health);
///     
///     // Read a f32 value
///     let speed: f32 = read_memory_t(handle, 0x7FF6A1B2C3E0)?;
///     println!("Speed: {}", speed);
///     
///     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
///     Ok(())
/// }
/// ```
pub fn read_memory_t<T: Copy + Sized>(handle: HANDLE, address: usize) -> Result<T, MemoryError> {
    let size = std::mem::size_of::<T>();
    let bytes = read_memory_bytes(handle, address, size)?;
    
    // Convert bytes back to T
    unsafe {
        let ptr = bytes.as_ptr() as *const T;
        Ok(ptr.read_unaligned())
    }
}

/// Write a primitive value to remote process memory
///
/// Writes a value of type T to the target process memory at the given address.
/// T must be a primitive type that can be safely written as bytes.
///
/// # Type Parameters
/// * `T` - The type to write (must implement Copy + Sized)
///
/// # Arguments
/// * `handle` - Handle to the target process
/// * `address` - The memory address to write to
/// * `value` - The value to write
///
/// # Returns
/// * `Ok(())` - If the write operation succeeded
/// * `Err(MemoryError)` - If the write operation failed
///
/// # Supported Types
/// Common primitive types: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`,
/// `f32`, `f64`, `usize`, `isize`
///
/// # Example
/// ```no_run
/// use win_auto_utils::memory::write_memory_t;
/// use win_auto_utils::handle::open_process_handle;
/// use windows::Win32::System::Threading::{PROCESS_VM_WRITE, PROCESS_VM_OPERATION};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let desired_access = PROCESS_VM_WRITE | PROCESS_VM_OPERATION;
///     let handle = open_process_handle(12345, desired_access).ok_or("Failed to open process")?;
///     
///     // Write an i32 value
///     write_memory_t(handle, 0x7FF6A1B2C3D4, 999i32)?;
///     
///     // Write a f32 value
///     write_memory_t(handle, 0x7FF6A1B2C3E0, 2.5f32)?;
///     
///     unsafe { windows::Win32::Foundation::CloseHandle(handle); }
///     Ok(())
/// }
/// ```
pub fn write_memory_t<T: Copy + Sized>(handle: HANDLE, address: usize, value: T) -> Result<(), MemoryError> {
    let size = std::mem::size_of::<T>();
    let bytes = unsafe {
        std::slice::from_raw_parts(&value as *const T as *const u8, size)
    };
    
    write_memory_bytes(handle, address, bytes)
}

// Convenience functions for common types

/// Read an i8 value from remote process memory
pub fn read_memory_i8(handle: HANDLE, address: usize) -> Result<i8, MemoryError> {
    read_memory_t(handle, address)
}

/// Read a u8 value from remote process memory
pub fn read_memory_u8(handle: HANDLE, address: usize) -> Result<u8, MemoryError> {
    read_memory_t(handle, address)
}

/// Read an i16 value from remote process memory
pub fn read_memory_i16(handle: HANDLE, address: usize) -> Result<i16, MemoryError> {
    read_memory_t(handle, address)
}

/// Read a u16 value from remote process memory
pub fn read_memory_u16(handle: HANDLE, address: usize) -> Result<u16, MemoryError> {
    read_memory_t(handle, address)
}

/// Read an i32 value from remote process memory
pub fn read_memory_i32(handle: HANDLE, address: usize) -> Result<i32, MemoryError> {
    read_memory_t(handle, address)
}

/// Read a u32 value from remote process memory
pub fn read_memory_u32(handle: HANDLE, address: usize) -> Result<u32, MemoryError> {
    read_memory_t(handle, address)
}

/// Read an i64 value from remote process memory
pub fn read_memory_i64(handle: HANDLE, address: usize) -> Result<i64, MemoryError> {
    read_memory_t(handle, address)
}

/// Read a u64 value from remote process memory
pub fn read_memory_u64(handle: HANDLE, address: usize) -> Result<u64, MemoryError> {
    read_memory_t(handle, address)
}

/// Read an f32 value from remote process memory
pub fn read_memory_f32(handle: HANDLE, address: usize) -> Result<f32, MemoryError> {
    read_memory_t(handle, address)
}

/// Read an f64 value from remote process memory
pub fn read_memory_f64(handle: HANDLE, address: usize) -> Result<f64, MemoryError> {
    read_memory_t(handle, address)
}

/// Write an i8 value to remote process memory
pub fn write_memory_i8(handle: HANDLE, address: usize, value: i8) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write a u8 value to remote process memory
pub fn write_memory_u8(handle: HANDLE, address: usize, value: u8) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write an i16 value to remote process memory
pub fn write_memory_i16(handle: HANDLE, address: usize, value: i16) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write a u16 value to remote process memory
pub fn write_memory_u16(handle: HANDLE, address: usize, value: u16) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write an i32 value to remote process memory
pub fn write_memory_i32(handle: HANDLE, address: usize, value: i32) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write a u32 value to remote process memory
pub fn write_memory_u32(handle: HANDLE, address: usize, value: u32) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write an i64 value to remote process memory
pub fn write_memory_i64(handle: HANDLE, address: usize, value: i64) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write a u64 value to remote process memory
pub fn write_memory_u64(handle: HANDLE, address: usize, value: u64) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write an f32 value to remote process memory
pub fn write_memory_f32(handle: HANDLE, address: usize, value: f32) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

/// Write an f64 value to remote process memory
pub fn write_memory_f64(handle: HANDLE, address: usize, value: f64) -> Result<(), MemoryError> {
    write_memory_t(handle, address, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_address_read() {
        // Create a dummy handle for testing (this will fail but tests error handling)
        let handle = HANDLE(std::ptr::null_mut());
        let result = read_memory_bytes(handle, 0, 10);
        assert!(result.is_err());
        match result {
            Err(MemoryError::InvalidAddress(_)) => {},
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_zero_address_write() {
        let handle = HANDLE(std::ptr::null_mut());
        let data = vec![0x90, 0x90];
        let result = write_memory_bytes(handle, 0, &data);
        assert!(result.is_err());
        match result {
            Err(MemoryError::InvalidAddress(_)) => {},
            _ => panic!("Expected InvalidAddress error"),
        }
    }

    #[test]
    fn test_null_handle_read() {
        let handle = HANDLE(std::ptr::null_mut());
        let result = read_memory_u32(handle, 0x1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_null_handle_write() {
        let handle = HANDLE(std::ptr::null_mut());
        let result = write_memory_u32(handle, 0x1000, 42u32);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_memory_t_type_safety() {
        // Test that read_memory_t works with different types
        let handle = HANDLE(std::ptr::null_mut());
        
        // These should all compile and return errors (due to null handle)
        let _: Result<u8, _> = read_memory_t(handle, 0x1000);
        let _: Result<i32, _> = read_memory_t(handle, 0x1000);
        let _: Result<f64, _> = read_memory_t(handle, 0x1000);
    }
}
