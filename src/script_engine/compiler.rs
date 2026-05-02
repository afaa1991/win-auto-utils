//! Script compiler
//!
//! Converts AST into linear instructions. The Compiler calls each Handler's `parse` method
//! and packages results into `CompiledInstruction`.
//!
//! # Compilation Process
//! 1. Iterate through AST nodes
//! 2. For each node, find the corresponding Handler in registry
//! 3. Call Handler's `parse` method to validate and convert arguments
//! 4. Package parsed data into `CompiledInstruction`
//! 5. Apply automatic block pairing for handlers that declare block structure
//!    (e.g., loop/end for establishing control flow mappings)
//!
//! # Performance Optimization
//! The compiler optimizes post-compilation processing by only calling `declare_block_structure`
//! for handlers that need block pairing. This avoids unnecessary overhead for simple instructions.
//!
//! # Example
//! ```no_run
//! use win_auto_utils::script_engine::compiler::{Compiler, CompilerConfig};
//! use win_auto_utils::script_engine::parser::AstNode;
//! use win_auto_utils::script_engine::instruction::InstructionRegistry;
///
/// let registry = InstructionRegistry::new();
/// let config = CompilerConfig::default();
/// let compiler = Compiler::new(config, &registry);
///
/// // Compile AST nodes (typically from Parser)
/// let ast_nodes = vec![/* ... */];
/// let script = compiler.compile(&ast_nodes).unwrap();
///
/// println!("Compiled {} instructions", script.instructions.len());
/// ```
use super::instruction::InstructionRegistry;
use super::instruction::{CompiledInstruction, InstructionMetadata, ScriptError};
use super::parser::AstNode;

/// Advanced block pairing function that handles multiple block types sharing the same terminator.
///
/// This function properly supports mixed nesting (e.g., time inside loop, loop inside time)
/// by maintaining a single stack that tracks both the start IP and block type.
///
/// # Algorithm
/// Uses a LIFO stack with type information:
/// - Stack stores: (start_ip, block_type_name)
/// - When encountering ANY start instruction, push (ip, name) onto stack
/// - When encountering an end instruction, pop from stack and create metadata for that specific block type
///
/// # Advantages
/// - Correctly handles arbitrary nesting of different block types
/// - Each end is paired with its matching start based on LIFO order
/// - No metadata conflicts between different block types
///
/// # Example
/// ```text
/// time 1000        # Push (0, "time")
///     loop 3       # Push (1, "loop")
///         key A
///     end          # Pop (1, "loop") → pair loop[1] with end[3]
/// end              # Pop (0, "time") → pair time[0] with end[4]
/// ```
pub fn multi_type_block_pairing(
    instructions: &[CompiledInstruction],
    block_types: &[(&str, &str)], // Array of (start_name, end_name) pairs
) -> Vec<Option<InstructionMetadata>> {
    let mut metadata_list = (0..instructions.len()).map(|_| None).collect::<Vec<_>>();

    // Stack stores (start_ip, start_instruction_name)
    let mut stack: Vec<(usize, String)> = Vec::new();

    for (ip, instr) in instructions.iter().enumerate() {
        // Check if this instruction is a block start
        let mut is_start = false;
        let mut block_type_name = String::new();

        for (start_name, _end_name) in block_types {
            if instr.name == *start_name {
                is_start = true;
                block_type_name = start_name.to_string();
                break;
            }
        }

        if is_start {
            // Push start instruction IP and type onto stack
            stack.push((ip, block_type_name));
        } else {
            // Check if this is an end instruction for any block type
            let mut matched_end = false;
            for (_start_name, end_name) in block_types {
                if instr.name == *end_name {
                    // Found an end instruction
                    if let Some((start_ip, start_type)) = stack.pop() {
                        // Create metadata for the start instruction
                        metadata_list[start_ip] =
                            Some(InstructionMetadata::new(GenericBlockMetadata {
                                end_ip: ip,
                                block_type: start_type.clone(),
                            }));

                        // Create metadata for the end instruction (only if not already set)
                        if metadata_list[ip].is_none() {
                            metadata_list[ip] =
                                Some(InstructionMetadata::new(TerminatorMetadata {
                                    start_ip,
                                    block_type: start_type,
                                }));
                        }
                        matched_end = true;
                    } else {
                        eprintln!(
                            "Warning: Unmatched '{}' terminator at IP {}",
                            instr.name, ip
                        );
                    }
                    break; // Only match once
                }
            }

            if !matched_end && !is_start {
                // This is neither a start nor an end we care about
                continue;
            }
        }
    }

    // Validate unclosed blocks
    if !stack.is_empty() {
        eprintln!("Warning: {} unclosed block(s) detected", stack.len());
        for (start_ip, start_type) in &stack {
            eprintln!(
                "  - '{}' block starting at IP {} has no matching 'end'",
                start_type, start_ip
            );
        }
    }

    metadata_list
}

/// Metadata for generic block-start instructions
#[derive(Debug, Clone)]
pub struct GenericBlockMetadata {
    /// IP of the matching terminator instruction
    pub end_ip: usize,
    /// Type identifier (usually the instruction name)
    pub block_type: String,
}

/// Extended metadata for blocks with optional middle sections (e.g., if-else-end)
#[derive(Debug, Clone)]
pub struct ConditionalBlockMetadata {
    /// IP of the else/middle instruction (if exists)
    pub else_ip: Option<usize>,
    /// IP of the end/terminator instruction
    pub end_ip: usize,
    /// Block type identifier
    pub block_type: String,
}

/// Metadata for terminator instructions (e.g., end, endif, endwhile)
#[derive(Debug, Clone)]
pub struct TerminatorMetadata {
    /// IP of the matching block-start instruction
    pub start_ip: usize,
    /// Type of the block being terminated
    pub block_type: String,
}

/// Utility function for pairing if-else-end structures
///
/// This is a more advanced version of `generic_pair_blocks` that supports
/// three-part block structures like:
/// ```text
/// if condition
///     // true branch
/// else
///     // false branch
/// end
/// ```
///
/// # How It Works
/// - Scans for `if_name`, `else_name`, and `end_name` instructions
/// - Establishes mappings: if else, if end, else end
/// - Stores `ConditionalBlockMetadata` in all three instructions
///
/// # Parameters
/// - `instructions`: The complete instruction sequence
/// - `if_name`: Name of the if instruction (e.g., "if")
/// - `else_name`: Name of the else instruction (e.g., "else")
/// - `end_name`: Name of the terminator (e.g., "end")
///
/// # Returns
/// Metadata vector with conditional block information
pub fn pair_conditional_blocks(
    instructions: &[CompiledInstruction],
    if_name: &str,
    else_name: &str,
    end_name: &str,
) -> Vec<Option<InstructionMetadata>> {
    let mut metadata_list = (0..instructions.len()).map(|_| None).collect::<Vec<_>>();

    // Stack stores (else_ip_or_none, if_start_ip)
    let mut stack: Vec<(Option<usize>, usize)> = Vec::new();

    for (ip, instr) in instructions.iter().enumerate() {
        if instr.name == if_name {
            // Push if instruction with no else yet
            stack.push((None, ip));
        } else if instr.name == else_name {
            // Update the top of stack to record else position
            if let Some((_, if_ip)) = stack.last_mut() {
                *stack.last_mut().unwrap() = (Some(ip), *if_ip);
            } else {
                eprintln!("Warning: 'else' without matching 'if' at IP {}", ip);
            }
        } else if instr.name == end_name {
            // Pop and establish full mapping
            if let Some((else_ip, if_ip)) = stack.pop() {
                // Store metadata for if instruction
                metadata_list[if_ip] = Some(InstructionMetadata::new(ConditionalBlockMetadata {
                    else_ip,
                    end_ip: ip,
                    block_type: if_name.to_string(),
                }));

                // Store metadata for else instruction (if exists)
                if let Some(else_ip_val) = else_ip {
                    metadata_list[else_ip_val] =
                        Some(InstructionMetadata::new(TerminatorMetadata {
                            start_ip: if_ip,
                            block_type: if_name.to_string(),
                        }));
                }

                // Store metadata for end instruction
                metadata_list[ip] = Some(InstructionMetadata::new(TerminatorMetadata {
                    start_ip: if_ip,
                    block_type: if_name.to_string(),
                }));
            } else {
                eprintln!("Warning: Unmatched '{}' terminator at IP {}", end_name, ip);
            }
        }
    }

    // Validate unclosed blocks
    if !stack.is_empty() {
        eprintln!(
            "Warning: {} unclosed '{}' block(s) detected",
            stack.len(),
            if_name
        );
    }

    metadata_list
}

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Optimization level (reserved for future use)
    pub optimization_level: u8,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization_level: 0,
        }
    }
}

/// Compiler for script engine.
///
/// The compiler transforms AST (Abstract Syntax Tree) into compiled instructions
/// through a multi-phase process:
///
/// # Compilation Phases
///
/// ## Phase 1: Parse (AST Instructions)
/// - Traverse AST nodes
/// - Call `handler.parse()` for each instruction
/// - Generate `CompiledInstruction` with name, data, and empty metadata
///
/// ## Phase 2: Post-Compilation Processing
///
/// ### Automatic Block Pairing
/// The compiler automatically pairs block instructions (like `loop...end`, `time...end`)
/// by calling `InstructionHandler::declare_block_structure()`. This eliminates the need
/// for manual pairing logic in each handler.
///
/// # Example
/// ```no_run
/// use win_auto_utils::script_engine::compiler::{Compiler, CompilerConfig};
/// use win_auto_utils::script_engine::parser::AstNode;
/// use win_auto_utils::script_engine::instruction::InstructionRegistry;
///
/// let registry = InstructionRegistry::new();
/// let config = CompilerConfig::default();
/// let compiler = Compiler::new(config, &registry);
///
/// // Compile AST nodes (typically from Parser)
/// let ast_nodes = vec![/* ... */];
/// let script = compiler.compile(&ast_nodes).unwrap();
///
/// println!("Compiled {} instructions", script.instructions.len());
/// ```

/// Script compiler
pub struct Compiler<'a> {
    #[allow(dead_code)]
    config: CompilerConfig,
    registry: &'a InstructionRegistry,
}

impl<'a> Compiler<'a> {
    pub fn new(config: CompilerConfig, registry: &'a InstructionRegistry) -> Self {
        Self { config, registry }
    }

    /// Compile AST node sequence
    pub fn compile(&self, ast_nodes: &[AstNode]) -> Result<CompiledScript, ScriptError> {
        let mut instructions = Vec::new();

        for node in ast_nodes {
            let instr = self.compile_node(node)?;
            instructions.push(instr);
        }

        self.apply_automatic_block_pairing(&mut instructions)?;

        Ok(CompiledScript::new(instructions))
    }

    /// Compile single AST node
    fn compile_node(&self, node: &AstNode) -> Result<CompiledInstruction, ScriptError> {
        // Get corresponding Handler from registry
        let handler = self.registry.get_handler(&node.command).ok_or_else(|| {
            ScriptError::CompilationError(format!("Handler not found for '{}'", node.command))
        })?;

        // Delegate to Handler for argument parsing
        let args: Vec<&str> = node.args.iter().map(|s| s.as_str()).collect();
        let data = handler.parse(&args)?;

        Ok(CompiledInstruction {
            name: node.command.clone(),
            data,
            metadata: None, // Will be filled by on_compilation_complete if needed
        })
    }

    /// Automatically pair blocks based on InstructionHandler's declare_block_structure().
    ///
    /// This method scans all registered handlers and automatically establishes
    /// start-end mappings without requiring manual pairing logic in each handler.
    fn apply_automatic_block_pairing(
        &self,
        instructions: &mut Vec<CompiledInstruction>,
    ) -> Result<(), ScriptError> {
        use crate::script_engine::instruction::BlockStructure;

        // Collect all block declarations from handlers
        let mut simple_pairs: Vec<(String, String)> = Vec::new();
        let mut conditional_blocks: Vec<(String, String, String)> = Vec::new();

        // Scan all unique instruction names
        let mut seen_names = std::collections::HashSet::new();
        for instr in instructions.iter() {
            if seen_names.insert(&instr.name) {
                if let Some(handler) = self.registry.get_handler(&instr.name) {
                    // Use InstructionHandler's declare_block_structure method
                    if let Some(structure) = handler.declare_block_structure() {
                        match structure {
                            BlockStructure::SimplePair {
                                start_name,
                                end_name,
                            } => {
                                simple_pairs.push((start_name.to_string(), end_name.to_string()));
                            }
                            BlockStructure::ConditionalTriple {
                                if_name,
                                else_name,
                                end_name,
                            } => {
                                conditional_blocks.push((
                                    if_name.to_string(),
                                    else_name.to_string(),
                                    end_name.to_string(),
                                ));
                            }
                            BlockStructure::Custom { .. } => {
                                eprintln!("Warning: Custom block structure not yet implemented");
                            }
                        }
                    }
                }
            }
        }

        // Apply multi-type pairing for simple pairs (supports mixed nesting)
        if !simple_pairs.is_empty() {
            let block_type_refs: Vec<(&str, &str)> = simple_pairs
                .iter()
                .map(|(s, e)| (s.as_str(), e.as_str()))
                .collect();

            let metadata = multi_type_block_pairing(instructions, &block_type_refs);
            self.merge_metadata(instructions, metadata);
        }

        // Apply conditional block pairing separately (if-else-end structures)
        for (if_name, else_name, end_name) in conditional_blocks {
            let metadata = pair_conditional_blocks(instructions, &if_name, &else_name, &end_name);
            self.merge_metadata(instructions, metadata);
        }

        Ok(())
    }

    /// Merge metadata from automatic pairing into instructions.
    ///
    /// # Strategy: Smart merge for block pairing metadata
    ///
    /// When an instruction has both:
    /// 1. Parse-time metadata (from InstructionHandler::parse)
    /// 2. Block pairing metadata (from automatic pairing)
    ///
    /// The block pairing metadata will **overwrite** the parse-time metadata,
    /// BUT multiple block pairing passes will be **merged** intelligently.
    ///
    /// # Rationale
    /// - Block pairing metadata is compiler-generated and consistent across all block instructions
    /// - Parse-time metadata can be stored in InstructionData::Custom instead
    /// - Keeps the metadata system simple (single slot per instruction)
    /// - For mixed nesting (time/loop/if), we need to preserve metadata from different block types
    ///
    /// # Example
    /// ```rust
    /// // In parse phase:
    /// Ok(InstructionData::custom(MyLoopConfig { count: 5 }))
    ///
    /// // In block pairing phase 1 (loop-end):
    /// // Compiler sets: metadata = GenericBlockMetadata { end_ip: 10 }
    ///
    /// // In block pairing phase 2 (time-end):
    /// // If this instruction is a 'time' start, it gets its own metadata
    /// // If this instruction is an 'end', it keeps existing metadata (from loop or time)
    ///
    /// // In execute phase:
    /// let config = data.get_custom_ref::<MyLoopConfig>()?;  // Read from InstructionData
    /// let end_ip = metadata.get::<GenericBlockMetadata>()?.end_ip;  // Read from metadata
    /// ```
    fn merge_metadata(
        &self,
        instructions: &mut Vec<CompiledInstruction>,
        new_metadata: Vec<Option<InstructionMetadata>>,
    ) {
        for (i, meta_opt) in new_metadata.into_iter().enumerate() {
            if i < instructions.len() {
                // Only set metadata if not already present
                // This allows multiple block pairing passes to coexist
                if instructions[i].metadata.is_none() {
                    instructions[i].metadata = meta_opt;
                }
                // If metadata already exists, skip (preserve existing pairing)
            }
        }
    }
}

/// Compiled script
#[derive(Debug)]
pub struct CompiledScript {
    /// Linear instruction sequence
    pub instructions: Vec<CompiledInstruction>,
    /// Source code lines (for debugging)
    pub source_lines: Vec<String>,
}

impl CompiledScript {
    pub fn new(instructions: Vec<CompiledInstruction>) -> Self {
        Self {
            instructions,
            source_lines: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}
