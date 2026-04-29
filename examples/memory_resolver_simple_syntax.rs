//! Memory Resolver Simple Syntax Example
//!
//! Demonstrates the clean hexadecimal syntax (no 0x prefix needed)
//! and various number format options.

use win_auto_utils::memory_resolver::MemoryAddress;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Resolver Simple Syntax Demo ===\n");

    // ✅ Clean hex syntax - no 0x prefix needed!
    println!("1. Clean Hex Syntax (Recommended):");
    let _addr1 = MemoryAddress::parse("game.exe+1000->20->30")?;
    println!("   Input:  'game.exe+1000->20->30'");
    println!("   Parsed as hex: 0x1000 -> 0x20 -> 0x30\n");

    // ✅ Also works with 0x prefix after module+offset
    println!("2. Traditional Hex Syntax (Also Works):");
    let _addr2 = MemoryAddress::parse("game.exe+0x1000->0x20->0x30")?;
    println!("   Input:  'game.exe+0x1000->0x20->0x30'");
    println!("   Same result, just more typing!\n");

    // ✅ Decimal with # prefix
    println!("3. Decimal Numbers (Use # Prefix):");
    let _addr3 = MemoryAddress::parse("#1000+#200")?;
    println!("   Input:  '#1000+#200'");
    println!("   Parsed as decimal: 1000 + 200\n");

    // ✅ Mixed syntax
    println!("4. Mixed Hex and Decimal:");
    let _addr4 = MemoryAddress::parse("target.exe+100->#50")?;
    println!("   Input:  'target.exe+100->#50'");
    println!("   First offset: 0x100 (hex), Second: 50 (decimal)\n");

    // ✅ Underscore for readability
    println!("5. Readable Number Formatting:");
    let _addr5 = MemoryAddress::parse("game.exe+1_0000->2FC")?;
    println!("   Input:  'game.exe+1_0000->2FC'");
    println!("   Underscores ignored: 0x10000 -> 0x2FC\n");

    // ✅ Real-world example (LF2 game)
    println!("6. Real-World Example (LF2 Game):");
    let lf2_addr = MemoryAddress::new_x86("lf2.exe+58C94->308")?;
    println!("   Input:  'lf2.exe+58C94->308'");
    println!("   Architecture: x86 (32-bit)");
    println!("   Pointer size: {} bytes", lf2_addr.pointer_size.bytes());
    println!("   Operations: {} steps", lf2_addr.operations.len());

    println!("\n✅ All examples parsed successfully!");
    println!("\nKey Takeaway: Use clean hex syntax without 0x prefix for brevity!");

    Ok(())
}
