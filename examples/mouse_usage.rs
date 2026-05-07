//! Mouse Usage Examples
//!
//! This example demonstrates various mouse automation capabilities.
//!
//! Features demonstrated:
//! - Basic mouse clicks (left, right, middle)
//! - Mouse movement (absolute and relative)
//! - Double clicking
//! - Pressing and holding buttons
//! - Scrolling
//! - High-performance atomic operations
//! - Script engine integration

use std::error::Error;
use std::time::Instant;

use win_auto_utils::mouse::{mouse_input, SendInputMouse};
use win_auto_utils::script_engine::ScriptEngine;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Mouse Usage Examples ===\n");

    // Example 1: Basic mouse usage
    println!("1. Basic Mouse Usage");
    basic_mouse_example()?;
    println!();

    // Example 2: Mouse movement
    println!("2. Mouse Movement");
    mouse_movement_example()?;
    println!();

    // Example 3: Drag and drop
    println!("3. Drag and Drop");
    drag_drop_example()?;
    println!();

    // Example 4: Scrolling
    println!("4. Scrolling");
    scrolling_example()?;
    println!();

    // Example 5: High-performance atomic operations
    println!("5. High-Performance Atomic Operations");
    atomic_operations_example()?;
    println!();

    // Example 6: Script engine usage
    println!("6. Script Engine Usage");
    script_engine_example()?;
    println!();

    println!("=== All examples completed! ===");
    Ok(())
}

fn basic_mouse_example() -> Result<(), Box<dyn Error>> {
    let mouse = SendInputMouse::new();

    println!("   Left click at current position");
    mouse.click_left()?;

    println!("   Right click at current position");
    mouse.click_right()?;

    println!("   Middle click at current position");
    mouse.click_middle()?;

    println!("   Left click at (500, 300)");
    mouse.click_left_at(500, 300)?;

    println!("   Double left click");
    mouse.double_click_left()?;

    Ok(())
}

fn mouse_movement_example() -> Result<(), Box<dyn Error>> {
    let mouse = SendInputMouse::new();

    println!("   Move to (100, 200)");
    mouse.move_to(100, 200)?;

    println!("   Move relative: +50, -20");
    mouse.move_relative(50, -20)?;

    println!("   Move to (800, 600) and click");
    mouse.click_left_at(800, 600)?;

    Ok(())
}

fn drag_drop_example() -> Result<(), Box<dyn Error>> {
    let mouse = SendInputMouse::new();

    println!("   Start at (200, 200)");
    mouse.move_to(200, 200)?;

    println!("   Press left button");
    mouse.press_left()?;

    println!("   Move to (500, 500) while holding");
    mouse.move_to(500, 500)?;

    println!("   Release left button");
    mouse.release_left()?;

    Ok(())
}

fn scrolling_example() -> Result<(), Box<dyn Error>> {
    let mouse = SendInputMouse::new();

    println!("   Scroll up 5 notches");
    mouse.scroll_up(5)?;

    println!("   Scroll down 3 notches");
    mouse.scroll_down(3)?;

    Ok(())
}

fn atomic_operations_example() -> Result<(), Box<dyn Error>> {
    // Pre-build INPUT structures
    let click_left = mouse_input::build_click_left();
    let move_to = mouse_input::build_move(500, 300);
    let scroll_up = mouse_input::build_scroll_up(1);
    let scroll_down = mouse_input::build_scroll_down(1);

    println!("   Benchmark: 1000 atomic left clicks");
    let start = Instant::now();
    for _ in 0..1000 {
        mouse_input::execute_inputs(&click_left)?;
    }
    let elapsed = start.elapsed();
    println!("   Completed in: {:?}", elapsed);
    println!("   Average per click: {:?}", elapsed / 1000);

    println!("\n   Atomic move + click");
    mouse_input::execute_single_input(&move_to)?;
    mouse_input::execute_inputs(&click_left)?;

    println!("\n   Atomic scroll");
    mouse_input::execute_single_input(&scroll_up)?;
    mouse_input::execute_single_input(&scroll_up)?;
    mouse_input::execute_single_input(&scroll_up)?;
    mouse_input::execute_single_input(&scroll_down)?;
    mouse_input::execute_single_input(&scroll_down)?;

    Ok(())
}

fn script_engine_example() -> Result<(), Box<dyn Error>> {
    let engine = ScriptEngine::with_builtin();

    println!("   Basic click at position");
    engine.compile_and_execute("click 100 200")?;

    println!("   Click with delay");
    engine.compile_and_execute("click 100 200 50")?;

    println!("   Double-click");
    engine.compile_and_execute("dbclick 300 400")?;

    println!("   Mouse movement");
    engine.compile_and_execute("move 500 300")?;
    engine.compile_and_execute("moverel 50 -20")?;

    println!("   Scroll operations");
    engine.compile_and_execute("scrollup 5")?;
    engine.compile_and_execute("scrolldown 3")?;

    println!("   Drag and drop");
    engine.compile_and_execute(
        r#"
        move 200 200
        press
        sleep 50
        moverel 300 300
        sleep 50
        release
        "#,
    )?;

    println!("   Complete workflow");
    engine.compile_and_execute(
        r#"
        # Move to menu
        move 100 100
        sleep 100
        
        # Click to open
        click 100 100
        sleep 200
        
        # Move to item
        move 100 150
        sleep 100
        
        # Click item
        click 100 150
        "#,
    )?;

    Ok(())
}
