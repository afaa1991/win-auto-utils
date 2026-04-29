//! Mouse input example - demonstrates mouse control capabilities
//!
//! This example shows how to use both PostMessage and SendInput methods
//! for mouse automation.
//!
//! # Running this example
//! ```bash
//! cargo run --example mouse_control --features "mouse"
//! ```

#[cfg(feature = "mouse")]
fn main() {
    println!("=== Mouse Control Example ===\n");

    // Note: This is a demonstration of the API structure.
    // Actual mouse operations require a valid window handle or screen coordinates.

    println!("Available mouse operations:");
    println!("- Click (left, right, middle)");
    println!("- Press/Release buttons");
    println!("- Move (absolute and relative)");
    println!("- Scroll (up/down)");
    println!("\nTwo input methods:");
    println!("1. PostMessage: Background input to specific window (requires coordinates)");
    println!("2. SendInput: System-level global input (optional coordinates)");

    // Example 1: SendInputMouse (global operations)
    #[cfg(feature = "mouse")]
    {
        use win_auto_utils::mouse::SendInputMouse;

        println!("\n--- SendInput Example ---");
        let _ = SendInputMouse::new();

        println!("✓ Created SendInputMouse instance");
        println!("  Methods available:");
        println!("  - click_left() / click_left_at(x, y)");
        println!("  - press_left() / release_left()");
        println!("  - move_to(x, y) / move_relative(dx, dy)");
        println!("  - scroll_up(delta) / scroll_down(delta)");

        // Uncomment to test (be careful!):
        // println!("\nMoving mouse to center of screen...");
        // mouse.move_to(960, 540).unwrap();
        // thread::sleep(Duration::from_millis(500));

        // println!("Clicking left button...");
        // mouse.click_left().unwrap();
    }

    // Example 2: PostMessageMouse (window-specific)
    #[cfg(all(feature = "mouse", feature = "hwnd"))]
    {
        use win_auto_utils::mouse::PostMessageMouse;
        use windows::Win32::Foundation::HWND;

        println!("\n--- PostMessage Example ---");

        // You would typically get hwnd from window enumeration
        let hwnd = HWND(std::ptr::null_mut()); // Placeholder
        let _ = PostMessageMouse::new(hwnd);

        println!("✓ Created PostMessageMouse instance");
        println!("  Methods available:");
        println!("  - click_left(x, y) / click_right(x, y) / click_middle(x, y)");
        println!("  - press_left(x, y) / release_left(x, y)");
        println!("  - move_to(x, y)");
        println!("  - scroll_up(x, y, delta) / scroll_down(x, y, delta)");

        println!("\nNote: PostMessage requires a valid window handle.");
        println!("Use hwnd module to find target windows.");
    }

    println!("\n=== Example Completed ===");
}

#[cfg(not(feature = "mouse"))]
fn main() {
    println!("This example requires the 'mouse' feature to be enabled.");
    println!("Run with: cargo run --example mouse_control --features \"mouse\"");
}
