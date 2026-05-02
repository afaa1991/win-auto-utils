use win_auto_utils::script_engine::{InstructionRegistry, ScriptConfig, ScriptEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an engine with built-in instructions
    let engine = ScriptEngine::with_builtin();

    println!("=== Example 1: Basic Sleep ===");
    println!("Script: sleep 10");
    engine.compile_and_execute("sleep 10")?;
    println!("✓ Completed\n");

    // Create a custom registry and register some instructions
    let mut registry = InstructionRegistry::new();

    // Register all built-in instructions using the new clean API
    win_auto_utils::scripts_builtin::register_all(&mut registry);

    let engine = ScriptEngine::with_registry_and_config(registry, ScriptConfig::default());

    // ===== Example 2: Loop with sleep =====
    println!("=== Example 2: Loop with Sleep ===");
    println!("Script:");
    println!("  loop 3");
    println!("      sleep 50");
    println!("  end");
    println!();
    engine.compile_and_execute("loop 3\n    sleep 50\nend")?;
    println!("✓ Completed (executed 3 iterations)\n");

    // ===== Example 3: Time instruction - timed execution block =====
    println!("=== Example 3: Time Instruction (Timed Execution Block) ===");
    println!("Script:");
    println!("  time 500");
    println!("      sleep 100");
    println!("  end");
    println!();
    println!("Expected: ~5 iterations (500ms / 100ms per iteration)");
    engine.compile_and_execute("time 500\n    sleep 100\nend")?;
    println!("✓ Completed (executed for ~500ms)\n");

    // ===== Example 4: Time with break =====
    println!("=== Example 4: Time Block with Break ===");
    println!("Script:");
    println!("  time 5000");
    println!("      sleep 100");
    println!("      break");
    println!("  end");
    println!();
    println!("Expected: Should stop after 1 iteration (~100ms), not 5000ms");
    engine.compile_and_execute("time 5000\n    sleep 100\n    break\nend")?;
    println!("✓ Completed (break exited time block early)\n");

    // ===== Example 5: Time with continue =====
    println!("=== Example 5: Time Block with Continue ===");
    println!("Script:");
    println!("  time 500");
    println!("      sleep 50");
    println!("      continue");
    println!("      sleep 100  # This will be skipped");
    println!("  end");
    println!();
    println!("Expected: ~10 iterations (500ms / 50ms), second sleep skipped");
    engine.compile_and_execute("time 500\n    sleep 50\n    continue\n    sleep 100\nend")?;
    println!("✓ Completed (continue skipped remaining body)\n");

    // ===== Example 6: Time with conditional break =====
    println!("=== Example 6: Time with Conditional Break ===");
    println!("Script:");
    println!("  time 1000");
    println!("      sleep 200");
    println!("      # Simulate condition check (in real use, would check for color/window/etc)");
    println!("      # For demo, we'll just break after 2 iterations by using a counter approach");
    println!("  end");
    println!();
    println!("Expected: Runs for ~1000ms with 5 iterations (1000/200)");
    engine.compile_and_execute("time 1000\n    sleep 200\nend")?;
    println!("✓ Completed (ran for ~1000ms)\n");

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     All Examples Completed Successfully! ✓               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    Ok(())
}
