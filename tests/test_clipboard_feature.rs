//! Clipboard feature integration test
//!
//! This test verifies the clipboard module functionality.
//! 
//! Run with: `cargo test --test test_clipboard_feature --features "clipboard" -- --nocapture`

#[cfg(feature = "clipboard")]
#[test]
fn test_clipboard_basic_operations() {
    use win_auto_utils::clipboard;

    println!("\n=== Testing Basic Clipboard Operations ===\n");

    // Test 1: Set and get text
    println!("Test 1: Set and get text");
    let test_text = "Test message";
    assert!(clipboard::set_text(test_text).is_ok());
    let retrieved = clipboard::get_text().unwrap();
    assert_eq!(retrieved, test_text);
    println!("  ✓ Passed\n");

    // Test 2: Unicode support
    println!("Test 2: Unicode support");
    let unicode = "中文 🌍 日本語";
    assert!(clipboard::set_text(unicode).is_ok());
    let retrieved = clipboard::get_text().unwrap();
    assert_eq!(retrieved, unicode);
    println!("  ✓ Passed\n");

    // Test 3: Empty string
    println!("Test 3: Empty string");
    assert!(clipboard::set_text("").is_ok());
    let retrieved = clipboard::get_text().unwrap();
    assert_eq!(retrieved, "");
    println!("  ✓ Passed\n");

    // Test 4: Long text
    println!("Test 4: Long text (10KB)");
    let long_text = "A".repeat(10000);
    assert!(clipboard::set_text(&long_text).is_ok());
    let retrieved = clipboard::get_text().unwrap();
    assert_eq!(retrieved.len(), long_text.len());
    println!("  ✓ Passed\n");

    // Test 5: Clear clipboard
    println!("Test 5: Clear clipboard");
    assert!(clipboard::clear().is_ok());
    println!("  ✓ Passed\n");

    println!("✅ All clipboard tests passed!\n");
}

#[cfg(feature = "clipboard")]
#[test]
fn test_clipboard_has_text() {
    use win_auto_utils::clipboard;

    println!("\n=== Testing has_text() Function ===\n");

    // Set some text
    clipboard::set_text("Test").ok();
    
    // Check if has_text returns true
    assert!(clipboard::has_text());
    println!("✓ has_text() returns true when clipboard has content\n");

    // Clear clipboard
    clipboard::clear().ok();
    
    // Note: has_text might still return true if other processes set clipboard
    println!("Note: After clearing, has_text() depends on system state\n");

    println!("✅ has_text() test completed\n");
}

#[cfg(not(feature = "clipboard"))]
#[test]
fn test_clipboard_feature_not_enabled() {
    println!("\n⚠️  Clipboard feature is not enabled");
    println!("Run with: cargo test --test test_clipboard_feature --features \"clipboard\"\n");
}
