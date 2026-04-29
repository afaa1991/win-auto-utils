//! Device Context (DC) management module
//! 
//! Provides functions to obtain device contexts for windows and the desktop.
//! Device contexts are used for drawing operations and screen capture.
//! 
//! # Important Notes
//! - Always release DCs when done using them to prevent resource leaks
//! - Use RAII patterns or ensure proper cleanup in error paths
//! - DCs obtained from GetDC must be released with ReleaseDC
//! - DCs obtained from GetWindowDC must be released with ReleaseDC

use windows::{
    Win32::Foundation::HWND,
    Win32::Graphics::Gdi::{GetDC, GetWindowDC, ReleaseDC},
};

/// Get the device context for a window's client area
/// 
/// Retrieves the DC for the client area of the specified window.
/// The client area is the portion of the window excluding title bar,
/// menu, borders, and scroll bars.
/// 
/// # Arguments
/// * `hwnd` - The window handle to get the client DC for
/// 
/// # Returns
/// * `Some(HDC)` - The device context handle if successful
/// * `None` - If the operation failed
/// 
/// # Important
/// **You MUST call `release_dc(hwnd, hdc)` when done with the DC**
/// Failure to do so will cause resource leaks.
/// 
/// # Example
/// ```no_run
/// use win_auto_utils::window::get_hwnd_by_title;
/// use win_auto_utils::hdc::{get_window_client_dc, release_dc};
/// 
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some(hdc) = get_window_client_dc(hwnd) {
///         // Use the DC for drawing or capture
///         // ...
///         
///         // IMPORTANT: Release the DC when done
///         release_dc(hwnd, hdc);
///     }
/// }
/// ```
pub fn get_window_client_dc(hwnd: HWND) -> Option<windows::Win32::Graphics::Gdi::HDC> {
    if hwnd.0.is_null() {
        return None;
    }

    unsafe {
        let hdc = GetDC(Some(hwnd));
        if hdc.0.is_null() {
            None
        } else {
            Some(hdc)
        }
    }
}

/// Get the device context for the entire window (including non-client areas)
/// 
/// Retrieves the DC for the entire window, including:
/// - Title bar
/// - Menu bar
/// - Borders
/// - Scroll bars
/// - Client area
/// 
/// This is useful for capturing the complete window appearance.
/// 
/// # Arguments
/// * `hwnd` - The window handle to get the window DC for
/// 
/// # Returns
/// * `Some(HDC)` - The device context handle if successful
/// * `None` - If the operation failed
/// 
/// # Important
/// **You MUST call `release_dc(hwnd, hdc)` when done with the DC**
/// 
/// # Example
/// ```no_run
/// use win_auto_utils::window::get_hwnd_by_title;
/// use win_auto_utils::hdc::{get_window_dc, release_dc};
/// 
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some(hdc) = get_window_dc(hwnd) {
///         // Capture entire window including title bar and borders
///         // ...
///         
///         // IMPORTANT: Release the DC when done
///         release_dc(hwnd, hdc);
///     }
/// }
/// ```
pub fn get_window_dc(hwnd: HWND) -> Option<windows::Win32::Graphics::Gdi::HDC> {
    if hwnd.0.is_null() {
        return None;
    }

    unsafe {
        let hdc = GetWindowDC(Some(hwnd));
        if hdc.0.is_null() {
            None
        } else {
            Some(hdc)
        }
    }
}

/// Get the device context for the desktop window
/// 
/// Retrieves the DC for the entire desktop screen.
/// This can be used for screen capture or drawing on the desktop.
/// 
/// # Returns
/// * `Some(HDC)` - The desktop device context handle if successful
/// * `None` - If the operation failed
/// 
/// # Important
/// **You MUST call `release_desktop_dc(hdc)` when done with the DC**
/// The desktop window handle is obtained via GetDesktopWindow().
/// 
/// # Example
/// ```no_run
/// use win_auto_utils::hdc::{get_desktop_dc, release_desktop_dc};
/// 
/// if let Some(hdc) = get_desktop_dc() {
///     // Capture entire screen
///     // ...
///     
///     // IMPORTANT: Release the DC when done
///     release_desktop_dc(hdc);
/// }
/// ```
pub fn get_desktop_dc() -> Option<windows::Win32::Graphics::Gdi::HDC> {
    use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

    unsafe {
        let desktop_hwnd = GetDesktopWindow();
        let hdc = GetDC(Some(desktop_hwnd));
        
        if hdc.0.is_null() {
            None
        } else {
            Some(hdc)
        }
    }
}

/// Release a device context obtained from get_window_client_dc or get_window_dc
/// 
/// # Arguments
/// * `hwnd` - The window handle associated with the DC
/// * `hdc` - The device context handle to release
/// 
/// # Returns
/// `true` if successful, `false` otherwise
/// 
/// # Example
/// ```no_run
/// use win_auto_utils::window::get_hwnd_by_title;
/// use win_auto_utils::hdc::{get_window_client_dc, release_dc};
/// 
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some(hdc) = get_window_client_dc(hwnd) {
///         // Use the DC...
///         release_dc(hwnd, hdc);
///     }
/// }
/// ```
pub fn release_dc(hwnd: HWND, hdc: windows::Win32::Graphics::Gdi::HDC) -> bool {
    if hwnd.0.is_null() || hdc.0.is_null() {
        return false;
    }

    unsafe {
        let result = ReleaseDC(Some(hwnd), hdc);
        result == 1
    }
}

/// Release the desktop device context
/// 
/// Convenience function that releases the desktop DC.
/// Internally calls release_dc with the desktop window handle.
/// 
/// # Arguments
/// * `hdc` - The desktop device context handle to release
/// 
/// # Returns
/// `true` if successful, `false` otherwise
/// 
/// # Example
/// ```no_run
/// use win_auto_utils::hdc::{get_desktop_dc, release_desktop_dc};
/// 
/// if let Some(hdc) = get_desktop_dc() {
///     // Use the DC...
///     release_desktop_dc(hdc);
/// }
/// ```
pub fn release_desktop_dc(hdc: windows::Win32::Graphics::Gdi::HDC) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

    if hdc.0.is_null() {
        return false;
    }

    unsafe {
        let desktop_hwnd = GetDesktopWindow();
        let result = ReleaseDC(Some(desktop_hwnd), hdc);
        result == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hwnd::get_hwnd_list;

    #[test]
    fn test_get_window_client_dc() {
        let hwnds = get_hwnd_list();
        
        if let Some(&hwnd) = hwnds.first() {
            if let Some(hdc) = get_window_client_dc(hwnd) {
                println!("Got client DC for window {:?}", hwnd);
                assert!(!hdc.0.is_null(), "DC should not be null");
                
                // Clean up
                let released = release_dc(hwnd, hdc);
                assert!(released, "Should successfully release DC");
            }
        }
    }

    #[test]
    fn test_get_window_dc() {
        let hwnds = get_hwnd_list();
        
        if let Some(&hwnd) = hwnds.first() {
            if let Some(hdc) = get_window_dc(hwnd) {
                println!("Got window DC for window {:?}", hwnd);
                assert!(!hdc.0.is_null(), "DC should not be null");
                
                // Clean up
                let released = release_dc(hwnd, hdc);
                assert!(released, "Should successfully release DC");
            }
        }
    }

    #[test]
    fn test_get_desktop_dc() {
        if let Some(hdc) = get_desktop_dc() {
            println!("Got desktop DC");
            assert!(!hdc.0.is_null(), "Desktop DC should not be null");
            
            // Clean up
            let released = release_desktop_dc(hdc);
            assert!(released, "Should successfully release desktop DC");
        }
    }

    #[test]
    fn test_release_invalid_dc() {
        // Test releasing invalid DCs
        let result = release_dc(HWND::default(), windows::Win32::Graphics::Gdi::HDC::default());
        assert!(!result, "Should fail to release invalid DC");
        
        let result = release_desktop_dc(windows::Win32::Graphics::Gdi::HDC::default());
        assert!(!result, "Should fail to release invalid desktop DC");
    }

    #[test]
    fn test_dc_lifecycle() {
        // Test complete DC lifecycle: get -> use -> release
        let hwnds = get_hwnd_list();
        
        if let Some(&hwnd) = hwnds.first() {
            // Get client DC
            if let Some(hdc) = get_window_client_dc(hwnd) {
                // Simulate using the DC
                println!("Using client DC: {:?}", hdc);
                
                // Release it
                assert!(release_dc(hwnd, hdc));
                
                // Try to use again (should not panic, but DC is invalid)
                println!("DC released successfully");
            }
            
            // Get window DC
            if let Some(hdc) = get_window_dc(hwnd) {
                println!("Using window DC: {:?}", hdc);
                assert!(release_dc(hwnd, hdc));
            }
        }
        
        // Test desktop DC
        if let Some(hdc) = get_desktop_dc() {
            println!("Using desktop DC: {:?}", hdc);
            assert!(release_desktop_dc(hdc));
        }
    }
}
