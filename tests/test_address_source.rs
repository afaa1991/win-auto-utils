//! Integration tests for AddressSource functionality

use win_auto_utils::memory_resolver::{AddressSource, MemoryAddress};

#[test]
fn test_address_source_static_creation() {
    // Test creating static address source
    let addr = MemoryAddress::new_x86("game.exe+1000").unwrap();
    let source = AddressSource::from_static(addr);
    
    // Verify it's a Static variant
    match source {
        AddressSource::Static(_) => {},
        _ => panic!("Expected Static variant"),
    }
}

#[test]
fn test_address_source_aob_creation() {
    // Test creating AOB address source
    let source = AddressSource::from_aob_scan("48 89 5C ?? 48").unwrap();
    
    // Verify it's an AobScan variant
    match source {
        AddressSource::AobScan { pattern, .. } => {
            assert_eq!(pattern, "48 89 5C ?? 48");
        },
        _ => panic!("Expected AobScan variant"),
    }
}

#[test]
fn test_address_source_aob_with_range() {
    // Test creating AOB with custom range
    let source = AddressSource::from_aob_scan_range(
        "E8 ?? ?? ?? ??",
        0x10000000,
        0x1000000
    ).unwrap();
    
    match source {
        AddressSource::AobScan { pattern, start_address, length, find_first } => {
            assert_eq!(pattern, "E8 ?? ?? ?? ??");
            assert_eq!(start_address, 0x10000000);
            assert_eq!(length, 0x1000000);
            assert_eq!(find_first, true);
        },
        _ => panic!("Expected AobScan variant"),
    }
}

#[test]
fn test_address_source_try_from_str() {
    // Test TryFrom<&str> implementation
    use std::convert::TryFrom;
    
    let source = AddressSource::try_from("game.exe+1000").unwrap();
    
    match source {
        AddressSource::Static(addr) => {
            match addr.base {
                win_auto_utils::memory_resolver::AddressBase::Module { name, offset } => {
                    assert_eq!(name, "game.exe");
                    assert_eq!(offset, 0x1000);
                },
                _ => panic!("Expected Module base"),
            }
        },
        _ => panic!("Expected Static variant"),
    }
}

#[test]
fn test_address_source_invalid_aob_pattern() {
    // AOB pattern is validated at scan time, not creation time
    // So creating an AddressSource with any pattern string should succeed
    let result = AddressSource::from_aob_scan("GG HH II");
    
    // Creation should succeed (validation happens during scan)
    assert!(result.is_ok(), "AddressSource creation should succeed even with potentially invalid patterns");
    
    // The pattern is stored as-is
    match result.unwrap() {
        AddressSource::AobScan { pattern, .. } => {
            assert_eq!(pattern, "GG HH II");
        },
        _ => panic!("Expected AobScan variant"),
    }
}

#[test]
fn test_address_source_clone() {
    // Test that AddressSource can be cloned
    let source = AddressSource::from_static(
        MemoryAddress::new_x86("game.exe+1000").unwrap()
    );
    
    let cloned = source.clone();
    
    // Both should be valid
    match (source, cloned) {
        (AddressSource::Static(_), AddressSource::Static(_)) => {},
        _ => panic!("Both should be Static variants"),
    }
}
