//! Clipboard usage example
//!
//! This example demonstrates how to use the clipboard module.
//!
//! Run with: `cargo run --example clipboard_usage --features "clipboard"`

#[cfg(feature = "clipboard")]
fn main() {
    use win_auto_utils::clipboard;

    println!("=== Clipboard Usage Example ===\n");

    // Example 1: Set and get text
    println!("1. Setting text to clipboard...");
    let original_text = "Hello from win-auto-utils!";
    match clipboard::set_text(original_text) {
        Ok(_) => println!("   ✓ Text set successfully"),
        Err(e) => println!("   ✗ Failed to set text: {}", e),
    }

    println!("\n2. Reading text from clipboard...");
    match clipboard::get_text() {
        Ok(text) => {
            println!("   ✓ Retrieved: \"{}\"", text);
            assert_eq!(text, original_text);
        }
        Err(e) => println!("   ✗ Failed to get text: {}", e),
    }

    // Example 2: Check if clipboard has text
    println!("\n3. Checking if clipboard contains text...");
    if clipboard::has_text() {
        println!("   ✓ Clipboard has text content");
    } else {
        println!("   ✗ Clipboard is empty or doesn't contain text");
    }

    // Example 3: Unicode support
    println!("\n4. Testing Unicode support...");
    let unicode_text = "你好世界！🌍 こんにちは мир";
    match clipboard::set_text(unicode_text) {
        Ok(_) => {
            println!("   ✓ Unicode text set successfully");
            if let Ok(retrieved) = clipboard::get_text() {
                println!("   ✓ Retrieved: \"{}\"", retrieved);
            }
        }
        Err(e) => println!("   ✗ Failed: {}", e),
    }

    // Example 4: Clear clipboard
    println!("\n5. Clearing clipboard...");
    match clipboard::clear() {
        Ok(_) => println!("   ✓ Clipboard cleared"),
        Err(e) => println!("   ✗ Failed to clear: {}", e),
    }

    println!("\n=== Example completed ===");
}

#[cfg(not(feature = "clipboard"))]
fn main() {
    println!("This example requires the 'clipboard' feature to be enabled.");
    println!("Run with: cargo run --example clipboard_usage --features \"clipboard\"");
}
