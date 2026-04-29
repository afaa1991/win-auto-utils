//! RegisterExtractor Module
//!
//! Provides automatic CPU register extraction at hook points.
//! This allows you to capture register values (like player pointers) without
//! complex base address lookups.
//!
//! # Quick Start
//! ```no_run
//! use win_auto_utils::memory_hook::register_extractor::{RegisterExtractor, Register};
//!
//! // Create and install extractor
//! let mut extractor = RegisterExtractor::builder()
//!     .handle(process_handle)
//!     .target_address(0x1C17E514F90)
//!     .bytes_to_overwrite(15)
//!     .extract_register(Register::RDI)
//!     .x64()
//!     .build()?;
//!
//! extractor.install()?;
//!
//! // Read captured register value
//! let player_ptr: u64 = extractor.read_register::<u64>(Register::RDI)?;
//!
//! // Read through pointer chain
//! let health: f32 = extractor.read_chain::<f32>(Register::RDI, &[0x34])?;
//!
//! // Modify game state
//! extractor.write_memory_t::<f32>(player_ptr + 0x34, 999.0)?;
//!
//! extractor.uninstall()?;
//! ```
//!
//! # Architecture
//! RegisterExtractor uses TrampolineHook internally to:
//! 1. Allocate private storage memory (PAGE_READWRITE)
//! 2. Generate shellcode that saves registers to storage
//! 3. Install hook at target address
//! 4. Provide API to read captured values
//!
//! # Memory Layout
//! ```text
//! Storage Region (128 bytes for x64):
//! ┌──────┬──────┬──────┬──────┐
//! │ RAX  │ RCX  │ RDX  │ RBX  │  Offset 0-31
//! ├──────┼──────┼──────┼──────┤
//! │ RSP  │ RBP  │ RSI  │ RDI  │  Offset 32-63
//! ├──────┼──────┼──────┼──────┤
//! │ R8   │ R9   │ R10  │ R11  │  Offset 64-95
//! ├──────┼──────┼──────┼──────┤
//! │ R12  │ R13  │ R14  │ R15  │  Offset 96-127
//! └──────┴──────┴──────┴──────┘
//! ```

mod register;
mod builder;
mod extractor;

pub use register::Register;
pub use builder::RegisterExtractorBuilder;
pub use extractor::RegisterExtractor;
