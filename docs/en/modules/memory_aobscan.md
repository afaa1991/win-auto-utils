# Memory AOB Scanner (Array of Bytes)

[中文文档](../../zh/modules/memory_aobscan.md) | [Back to Overview](overview.md)

The `memory_aobscan` module provides high-performance pattern scanning in remote process memory using SIMD acceleration, memchr-based heuristic search, and parallel processing. It's ideal for finding dynamic addresses, function signatures, and code patterns without hardcoded offsets.

## Feature Flag

```toml
[dependencies]
win-auto-utils = { version = "0.1.0", features = ["memory_aobscan"] }
```

## Quick Start

### Basic Pattern Scan

```rust
use win_auto_utils::memory_aobscan::AobScanBuilder;
use win_auto_utils::handle::open_process_handle;
use windows::Win32::System::Threading::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 12345;
    
    // Get process handle
    let desired_access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
    let handle = open_process_handle(pid, desired_access)
        .ok_or("Failed to open process")?;
    
    // Scan for pattern with wildcards
    let results = AobScanBuilder::new(handle)
        .pattern_str("48 89 5C 24 ?? 48 89 6C 24 ??")?
        .find_all(false)  // Find first match only (faster)
        .scan()?;
    
    if let Some(addr) = results.first() {
        println!("Found at: 0x{:X}", addr);
    }
    
    Ok(())
}
```

### Advanced Usage with Range Limiting

```rust
use win_auto_utils::memory_aobscan::AobScanBuilder;

// Scan specific memory range
let results = AobScanBuilder::new(handle)
    .pattern_str("E8 ?? ?? ?? ??")?  // Call instruction pattern
    .start_address(0x10000000)
    .length(0x1000000)  // Scan 16MB
    .find_all(true)     // Find all occurrences
    .scan()?;

println!("Found {} matches", results.len());
```

## Key Features

- **Dual SIMD Acceleration**: AVX-512 (64 bytes/cycle) and AVX2 (32 bytes/cycle) with automatic selection
- **memchr Optimization**: Fast byte search for anchor points
- **Parallel Scanning**: Multi-threaded region processing via Rayon
- **Smart Filtering**: Only scans committed, readable memory regions
- **Early Exit**: Stops immediately when first match is found
- **Wildcard Support**: Use `??` or `?` for unknown bytes
- **Region Caching**: Caches memory layout for repeated scans
- **Multi-byte Anchors**: Intelligent anchor sequence selection

## Usage Examples

### Example 1: Finding Function Signatures

```rust
use win_auto_utils::memory_aobscan::AobScanBuilder;

// Find a specific function by its machine code signature
let pattern = "55 48 8B EC 48 83 EC 30";  // Typical function prologue
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .find_all(false)
    .scan()?;

if let Some(func_addr) = results.first() {
    println!("Function found at: 0x{:X}", func_addr);
}
```

### Example 2: Scanning with Multiple Wildcards

```rust
use win_auto_utils::memory_aobscan::AobScanBuilder;

// Pattern with multiple unknown bytes (e.g., relative addresses)
let pattern = "48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B D8";
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .find_all(true)
    .scan()?;

println!("Found {} call sequences", results.len());
for addr in &results {
    println!("  - 0x{:X}", addr);
}
```

### Example 3: Limited Range Scan for Performance

```rust
use win_auto_utils::memory_aobscan::AobScanBuilder;

// Scan only the game module (much faster than full memory)
let module_base = 0x140000000;
let module_size = 0x5000000;  // 80MB

let results = AobScanBuilder::new(handle)
    .pattern_str("48 89 5C 24 ??")?
    .start_address(module_base)
    .length(module_size)
    .find_all(true)
    .scan()?;

println!("Found {} matches in game module", results.len());
```

### Example 4: Repeated Scans with Cache

```rust
use win_auto_utils::memory_aobscan::{AobScanBuilder, clear_region_cache};

let mut scanner = AobScanBuilder::new(handle)
    .pattern_str("48 8B 05 ?? ?? ?? ??")?
    .find_all(false);

// First scan (builds cache)
let result1 = scanner.clone().scan()?;

// Second scan (uses cached regions - much faster)
let result2 = scanner.clone().scan()?;

// Clear cache if process memory layout changed
clear_region_cache(pid);
```

### Example 5: Finding Data Patterns

```rust
use win_auto_utils::memory_aobscan::AobScanBuilder;

// Search for specific data values (e.g., health = 100.0f32)
let health_bytes = 100.0f32.to_le_bytes();
let pattern_hex = format!(
    "{:02X} {:02X} {:02X} {:02X}",
    health_bytes[0], health_bytes[1],
    health_bytes[2], health_bytes[3]
);

let results = AobScanBuilder::new(handle)
    .pattern_str(&pattern_hex)?
    .find_all(true)
    .scan()?;

println!("Found {} instances of 100.0f32", results.len());
```

### Example 6: Error Handling

```rust
use win_auto_utils::memory_aobscan::{AobScanBuilder, AobScanError};

match AobScanBuilder::new(handle)
    .pattern_str("INVALID PATTERN")?
    .scan()
{
    Ok(results) => println!("Found {} matches", results.len()),
    Err(AobScanError::InvalidPattern(msg)) => {
        eprintln!("Invalid pattern: {}", msg);
    }
    Err(AobScanError::MemoryReadFailed(addr)) => {
        eprintln!("Failed to read memory at 0x{:X}", addr);
    }
    Err(e) => eprintln!("Scan error: {}", e),
}
```

## API Reference

### Main Types

#### AobScanBuilder

Builder for configuring and executing AOB scans.

**Constructor**:
- `AobScanBuilder::new(handle: HANDLE)` - Create new scanner

**Configuration Methods**:
- `pattern_str(pattern: &str) -> Result<Self, AobScanError>` - Set pattern from string
- `pattern(pattern: Pattern) -> Self` - Set pre-parsed pattern
- `start_address(addr: usize) -> Self` - Set scan start address
- `length(len: usize) -> Self` - Set scan length in bytes
- `find_all(find_all: bool) -> Self` - Find all matches or just first
- `scan(self) -> Result<Vec<usize>, AobScanError>` - Execute scan

#### Pattern

Represents a parsed AOB pattern with wildcards.

**Constructor**:
- `Pattern::from_str(pattern: &str) -> Result<Self, AobScanError>` - Parse pattern string

**Methods**:
- `len(&self) -> usize` - Get pattern length
- `has_wildcards(&self) -> bool` - Check if pattern contains wildcards

#### AobScanError

Error types for AOB scanning operations.

**Variants**:
- `InvalidPattern(String)` - Pattern syntax error
- `MemoryReadFailed(usize)` - Failed to read memory at address
- `NoReadableRegions` - No valid memory regions found
- `ParseError(String)` - Pattern parsing failed

## Pattern Syntax

### Format

Patterns are space-separated hex bytes with optional wildcards.

| Syntax | Example | Description |
|--------|---------|-------------|
| Hex byte | `48` | Exact byte value |
| Wildcard | `??` or `?` | Any byte value |
| Mixed | `48 ?? 89` | Specific + wildcard |

### Examples

```rust
// Exact pattern (no wildcards)
"48 89 5C 24 10"

// With wildcards (for variable addresses/offsets)
"48 89 5C 24 ?? 48 89 6C 24 ??"

// Call instruction with relative offset
"E8 ?? ?? ?? ??"

// Mixed specificity
"48 8B ?? ?? ?? ?? ?? 48 85 C0"
```

### Case Insensitivity

```rust
// All equivalent
"48 89 5C"
"48 89 5c"
"48 89 5C"
```

## Performance Characteristics

### Scan Speed by Configuration

| Configuration | Typical Speed | Notes |
|---------------|---------------|-------|
| First match only | 10-50ms | Early exit optimization |
| All matches (small range) | 50-200ms | < 100MB scanned |
| All matches (full memory) | 500ms-2s | Depends on RAM size |
| Cached second scan | 5-20ms | Region cache hit |

### Optimization Techniques

1. **Anchor Selection**: Chooses rarest byte sequence for initial memchr search
2. **SIMD Verification**: Uses AVX2 to verify 32 bytes simultaneously
3. **Parallel Regions**: Splits memory into chunks for multi-threaded scanning
4. **Prefetching**: Hides memory latency with software prefetch hints
5. **Region Filtering**: Skips uncommitted/unreadable memory pages

### Benchmark Example

```rust
use std::time::Instant;
use win_auto_utils::memory_aobscan::AobScanBuilder;

let start = Instant::now();
let results = AobScanBuilder::new(handle)
    .pattern_str("48 89 5C 24 ??")?
    .find_all(true)
    .scan()?;
let elapsed = start.elapsed();

println!("Scanned in {:?}", elapsed);
println!("Found {} matches", results.len());
// Typical: 50-200ms for 1GB range on modern CPU
```

## Best Practices

1. **Limit Scan Range When Possible**
   ```rust
   // Good: Scan only relevant module
   let results = AobScanBuilder::new(handle)
       .pattern_str(pattern)?
       .start_address(module_base)
       .length(module_size)
       .scan()?;
   
   // Bad: Scan entire process memory (slow)
   let results = AobScanBuilder::new(handle)
       .pattern_str(pattern)?
       .scan()?;
   ```

2. **Use find_all(false) for Single Match**
   ```rust
   // Good: Stop after first match
   .find_all(false)
   
   // Bad: Continue scanning unnecessarily
   .find_all(true)  // When you only need one result
   ```

3. **Cache Region Layout for Repeated Scans**
   ```rust
   // Good: Reuse scanner with same handle
   let mut scanner = AobScanBuilder::new(handle).pattern_str(p1)?;
   let r1 = scanner.clone().scan()?;
   
   scanner = AobScanBuilder::new(handle).pattern_str(p2)?;
   let r2 = scanner.scan()?;  // Uses cached regions
   ```

4. **Choose Specific Patterns**
   ```rust
   // Good: Longer, more specific pattern
   "48 89 5C 24 10 48 89 6C 24 18"
   
   // Bad: Short pattern with many false positives
   "48 89"
   ```

5. **Handle Errors Gracefully**
   ```rust
   match scanner.scan() {
       Ok(results) => use_results(results),
       Err(AobScanError::NoReadableRegions) => {
           eprintln!("Process may have exited");
       }
       Err(e) => log_error(e),
   }
   ```

## Common Pitfalls

### ❌ Using Too Generic Patterns

```rust
// Wrong: Too many matches, slow scan
let results = AobScanBuilder::new(handle)
    .pattern_str("90")?  // NOP - appears millions of times
    .find_all(true)
    .scan()?;

// Correct: More specific pattern
let results = AobScanBuilder::new(handle)
    .pattern_str("48 89 5C 24 ?? 48 89 6C 24 ??")?
    .find_all(true)
    .scan()?;
```

### ❌ Not Limiting Scan Range

```rust
// Wrong: Scans entire 64-bit address space (very slow)
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .scan()?;

// Correct: Limit to known module
let results = AobScanBuilder::new(handle)
    .pattern_str(pattern)?
    .start_address(0x140000000)
    .length(0x5000000)
    .scan()?;
```

### ❌ Ignoring Wildcard Impact

```rust
// Wrong: Too many wildcards reduce performance
"?? ?? ?? ?? ?? ?? ?? ??"  // Almost no filtering

// Correct: Balance specificity and flexibility
"48 89 ?? 24 ?? 48 89"  // Anchor bytes for memchr
```

## Architecture Details

### Three-Phase Scanning

1. **Region Discovery**
   - Uses `VirtualQueryEx` to enumerate memory regions
   - Filters for committed, readable pages
   - Caches results for repeated scans

2. **Anchor Search**
   - Selects optimal byte sequence from pattern
   - Uses `memchr` for fast single-byte search
   - Extends to multi-byte verification

3. **Pattern Verification**
   - Validates full pattern at candidate positions
   - Uses AVX2 SIMD for 32-byte parallel comparison
   - Software prefetching hides memory latency

### SIMD Acceleration

The module implements a three-tier verification strategy:

- **AVX-512 Instructions**: Processes 64 bytes per cycle (fastest, requires AVX-512F support)
- **AVX2 Instructions**: Processes 32 bytes per cycle (fast, widely supported)
- **Scalar Fallback**: Optimized scalar code with software prefetching for older CPUs

**Automatic Selection Logic**:
1. Pattern ≥ 32 bytes + AVX-512F detected → Use AVX-512
2. Pattern ≥ 16 bytes + AVX2 detected → Use AVX2
3. Otherwise → Use optimized scalar implementation

This ensures optimal performance across different hardware generations while maintaining compatibility.

## Related Modules

- [`memory_resolver`](memory_resolver.md): Resolve found addresses to pointers
- [`memory`](memory.md): Read/write values at found addresses
- [`memory_hook`](memory_hook.md): Hook found function addresses
- [`snapshot`](process_window.md): Enumerate modules for scan ranges

---

**Language**: [English](memory_aobscan.md) | [中文](../../zh/modules/memory_aobscan.md)
