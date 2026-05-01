//! Memory allocation strategies for TrampolineHook
//!
//! This module provides architecture-aware memory allocation functions:
//! - Detour memory allocation (x86 direct, x64 code cave)
//! - Trampoline memory allocation (optimized for nearby placement)
//!
//! All functions are package-private (pub(super)) and should only be used
//! within the trampoline_hook module.

use crate::memory::MemoryError;
use crate::memory_hook::Architecture;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
    VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};

/// Allocate detour memory near target address
///
/// Allocation strategy:
/// - **x86**: Direct allocation (always within 32-bit address space)
/// - **x64**: Code cave approach (hint allocation ±256MB) → fallback
///
/// # Arguments
/// * `handle` - Process handle with full access rights
/// * `target_address` - Target function address (for proximity calculation)
/// * `size` - Required memory size in bytes
/// * `architecture` - Target architecture (x86 or x64)
///
/// # Returns
/// Allocated memory address
///
/// # Errors
/// Returns `MemoryError` if allocation fails
pub(super) fn allocate_detour_memory(
    handle: HANDLE,
    target_address: usize,
    size: usize,
    architecture: Architecture,
) -> Result<usize, MemoryError> {
    match architecture {
        Architecture::X86 => {
            // For x86, direct allocation is sufficient (always within ±2GB)
            let allocated = unsafe {
                VirtualAllocEx(
                    handle,
                    None,
                    size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                )
            };

            if allocated.is_null() {
                return Err(MemoryError::WriteFailed(
                    "Failed to allocate detour memory (x86)".to_string(),
                ));
            }

            Ok(allocated as usize)
        }
        Architecture::X64 => {
            // For x64, try code cave approach first (allocate near target)
            let hint_distance = 0x1000_0000; // 256MB
            let hint_address = target_address.wrapping_sub(hint_distance);
            
            let allocated = unsafe {
                VirtualAllocEx(
                    handle,
                    Some(hint_address as *mut _),
                    size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                )
            };

            if !allocated.is_null() {
                return Ok(allocated as usize);
            }

            // Fallback: allocate without hint
            let allocated = unsafe {
                VirtualAllocEx(
                    handle,
                    None,
                    size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                )
            };

            if allocated.is_null() {
                return Err(MemoryError::WriteFailed(
                    "Failed to allocate detour memory (x64)".to_string(),
                ));
            }

            Ok(allocated as usize)
        }
    }
}

/// Allocate trampoline memory near the target address
///
/// This function implements an optimal allocation strategy based on architecture:
/// - **x86**: Direct allocation (always within ±2GB, use relative JMP)
/// - **x64**: Two-stage strategy: code cave (hint allocation) → fallback
///
/// # Arguments
/// * `handle` - Process handle with full access rights
/// * `target_address` - Target function address (for proximity calculation)
/// * `size` - Required memory size in bytes
/// * `architecture` - Target architecture (x86 or x64)
///
/// # Returns
/// Allocated memory address
///
/// # Errors
/// Returns `MemoryError` if allocation fails
pub(super) fn allocate_trampoline_nearby(
    handle: HANDLE,
    target_address: usize,
    size: usize,
    architecture: Architecture,
) -> Result<usize, MemoryError> {
    // For x86, direct allocation is sufficient (always within 32-bit address space)
    if architecture == Architecture::X86 {
        return allocate_memory_direct(handle, size);
    }

    // For x64, use code cave strategy (no more scanning PRIVATE memory)
    allocate_trampoline_x64(handle, target_address, size)
}

/// Direct memory allocation for x86 (simpler strategy)
///
/// Since x86 uses 32-bit addresses, any allocated memory is automatically
/// within ±2GB range, allowing relative JMP instructions.
fn allocate_memory_direct(handle: HANDLE, size: usize) -> Result<usize, MemoryError> {
    let allocated = unsafe {
        VirtualAllocEx(
            handle,
            None, // No hint needed for x86
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if allocated.is_null() {
        return Err(MemoryError::WriteFailed(
            "Failed to allocate trampoline memory".to_string(),
        ));
    }

    Ok(allocated as usize)
}

/// Code cave allocation strategy for x64
///
/// Priority:
/// 1. Allocate with hint address (±256MB) for relative JMP possibility
/// 2. Fallback: allocate without hint (may require absolute jump)
///
/// # Returns
/// Allocated memory address
fn allocate_trampoline_x64(
    handle: HANDLE,
    target_address: usize,
    size: usize,
) -> Result<usize, MemoryError> {
    // === Stage 1: Allocate with hint address (±256MB) ===
    let hint_distance = 0x1000_0000; // 256MB
    let hint_address = target_address.wrapping_sub(hint_distance);

    let allocated = unsafe {
        VirtualAllocEx(
            handle,
            Some(hint_address as *mut _),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if !allocated.is_null() {
        return Ok(allocated as usize);
    }

    // === Stage 2: Fallback - allocate without hint ===
    // May be far from target, requiring absolute jump
    let allocated = unsafe {
        VirtualAllocEx(
            handle,
            None,
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if allocated.is_null() {
        return Err(MemoryError::WriteFailed(
            "Failed to allocate trampoline memory".to_string(),
        ));
    }

    Ok(allocated as usize)
}
