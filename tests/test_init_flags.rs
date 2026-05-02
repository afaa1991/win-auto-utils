//! Tests for process initialization flags (InitFlags)
//!
//! These tests verify that the InitFlags configuration correctly controls
//! which resources are initialized during process connection.

use win_auto_utils::process::{InitFlags, Process, ProcessConfig};

#[test]
fn test_init_flags_default() {
    // Test that default InitFlags enables all resources
    let flags = InitFlags::default();
    assert!(flags.init_pid);
    assert!(flags.init_hwnd);
    assert!(flags.init_handle);
    assert!(flags.init_dc);
}

#[test]
fn test_init_flags_new() {
    // Test that new() creates flags with all resources enabled
    let flags = InitFlags::new();
    assert!(flags.init_pid);
    assert!(flags.init_hwnd);
    assert!(flags.init_handle);
    assert!(flags.init_dc);
}

#[test]
fn test_init_flags_minimal() {
    // Test that minimal() only enables PID
    let flags = InitFlags::minimal();
    assert!(flags.init_pid);
    assert!(!flags.init_hwnd);
    assert!(!flags.init_handle);
    assert!(!flags.init_dc);
}

#[test]
fn test_init_flags_memory_only() {
    // Test that memory_only() enables PID and HANDLE only
    let flags = InitFlags::memory_only();
    assert!(flags.init_pid);
    assert!(!flags.init_hwnd);
    assert!(flags.init_handle);
    assert!(!flags.init_dc);
}

#[test]
fn test_init_flags_gui_only() {
    // Test that gui_only() enables PID, HWND, and DC
    let flags = InitFlags::gui_only();
    assert!(flags.init_pid);
    assert!(flags.init_hwnd);
    assert!(!flags.init_handle);
    assert!(flags.init_dc);
}

#[test]
fn test_init_flags_builder_pattern() {
    // Test builder pattern for custom flags
    let flags = InitFlags::new()
        .with_pid(true)
        .with_hwnd(true)
        .with_handle(false)
        .with_dc(false);

    assert!(flags.init_pid);
    assert!(flags.init_hwnd);
    assert!(!flags.init_handle);
    assert!(!flags.init_dc);
}

#[test]
fn test_init_flags_all_and_none() {
    // Test convenience methods
    let all = InitFlags::all();
    assert!(all.init_pid && all.init_hwnd && all.init_handle && all.init_dc);

    let none = InitFlags::none();
    assert!(none.init_pid && !none.init_hwnd && !none.init_handle && !none.init_dc);
}

#[test]
fn test_process_config_with_init_flags() {
    // Test that ProcessConfig correctly stores InitFlags
    let config = ProcessConfig::builder("test.exe")
        .init_flags(InitFlags::minimal())
        .build();

    assert_eq!(config.init_flags.init_pid, true);
    assert_eq!(config.init_flags.init_hwnd, false);
    assert_eq!(config.init_flags.init_handle, false);
    assert_eq!(config.init_flags.init_dc, false);
}

#[test]
fn test_process_state_with_optional_resources() {
    // Test that Process can be created with different init flags
    // Note: This test doesn't actually initialize the process, just verifies
    // that the configuration is accepted without errors

    let configs = vec![
        ("default", ProcessConfig::builder("test.exe").build()),
        (
            "minimal",
            ProcessConfig::builder("test.exe")
                .init_flags(InitFlags::minimal())
                .build(),
        ),
        (
            "memory_only",
            ProcessConfig::builder("test.exe")
                .init_flags(InitFlags::memory_only())
                .build(),
        ),
        (
            "gui_only",
            ProcessConfig::builder("test.exe")
                .init_flags(InitFlags::gui_only())
                .build(),
        ),
    ];

    for (name, config) in configs {
        let process = Process::new(config);
        // Verify process was created successfully
        assert_eq!(
            process.pid(),
            None,
            "Process {} should not be initialized yet",
            name
        );
        assert_eq!(
            process.hwnd(),
            None,
            "Process {} should not be initialized yet",
            name
        );
        assert_eq!(
            process.handle(),
            None,
            "Process {} should not be initialized yet",
            name
        );
        assert_eq!(
            process.hdc(),
            None,
            "Process {} should not be initialized yet",
            name
        );
    }
}

#[test]
fn test_init_flags_clone() {
    // Test that InitFlags implements Clone correctly
    let flags1 = InitFlags::memory_only();
    let flags2 = flags1.clone();

    assert_eq!(flags1.init_pid, flags2.init_pid);
    assert_eq!(flags1.init_hwnd, flags2.init_hwnd);
    assert_eq!(flags1.init_handle, flags2.init_handle);
    assert_eq!(flags1.init_dc, flags2.init_dc);
}

#[test]
fn test_init_flags_equality() {
    // Test that InitFlags implements PartialEq correctly
    let flags1 = InitFlags::gui_only();
    let flags2 = InitFlags::gui_only();
    let flags3 = InitFlags::minimal();

    assert_eq!(flags1, flags2);
    assert_ne!(flags1, flags3);
}
