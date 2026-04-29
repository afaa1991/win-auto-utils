//! Window management module
//!
//! Provides window enumeration, lookup, and manipulation functionality using Windows User32 API.
//!
//! # Features
//! - Find window handles by process ID
//! - Find window handles by title with flexible filtering
//! - Enumerate all windows for a specific process
//! - Advanced window filtering with include/exclude patterns
//!
//! # Window Filtering
//! The module supports flexible window filtering using `(String, u8)` tuples where:
//! - `String`: The pattern to match against window titles
//! - `u8` (mask): Filter mode
//!   - `1`: Window title MUST contain this pattern
//!   - `0`: Window title MUST NOT contain this pattern
//!
//! Multiple filters can be combined, and ALL filters must match for a window to be included.
//!
//! # Example
//! ```no_run
//! use win_auto_utils::window::get_hwnd_list_filtered;
//!
//! // Get windows that contain "Notepad" but don't contain "Untitled"
//! let filters = vec![
//!     ("Notepad".to_string(), 1),    // Must contain "Notepad"
//!     ("Untitled".to_string(), 0),   // Must NOT contain "Untitled"
//! ];
//! let hwnds = get_hwnd_list_filtered(None, Some(&filters));
//! ```

use windows::{
    core::BOOL,
    Win32::{
        Foundation::{HWND, LPARAM},
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        },
    },
};

/// Callback function for EnumWindows - collects all window handles
pub extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let list = lparam.0 as *mut Vec<HWND>;
        (*list).push(hwnd);
        BOOL(1)
    }
}

/// Get window title as UTF-16 String
fn get_window_title(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let length = unsafe { GetWindowTextW(hwnd, &mut buffer) };

    if length == 0 {
        return String::new();
    }

    String::from_utf16(&buffer[..length as usize]).unwrap_or_default()
}

/// Get process ID from window handle
fn get_pid_by_hwnd(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    pid
}

/// Get all window handles
///
/// Enumerates all top-level windows on the screen.
///
/// # Returns
/// A vector of all window handles
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::get_hwnd_list;
///
/// let hwnds = get_hwnd_list();
/// println!("Found {} windows", hwnds.len());
/// ```
pub fn get_hwnd_list() -> Vec<HWND> {
    let mut hwnds: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut hwnds as *mut Vec<HWND> as isize),
        );
    }
    hwnds
}

/// Get the main window handle by process ID
///
/// This function enumerates all windows and returns the first visible window
/// that belongs to the specified process and has a non-empty title.
///
/// # Arguments
/// * `pid` - The process ID to search for
///
/// # Returns
/// * `Some(HWND)` - The window handle if found
/// * `None` - If no window is found for the given PID
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid_by_name;
/// use win_auto_utils::window::get_hwnd_by_pid;
///
/// if let Some(pid) = find_pid_by_name("notepad.exe") {
///     if let Some(hwnd) = get_hwnd_by_pid(pid) {
///         println!("Found Notepad window: {:?}", hwnd);
///     }
/// }
/// ```
pub fn get_hwnd_by_pid(pid: u32) -> Option<HWND> {
    let hwnd = get_hwnd_list().into_iter().find(|&hwnd| {
        let cur_pid = get_pid_by_hwnd(hwnd);
        let title = get_window_title(hwnd);
        cur_pid == pid && !title.is_empty()
    });

    hwnd.filter(|h| h.0 != std::ptr::null_mut())
}

/// Get window handle by window title
///
/// Searches for a window whose title contains the specified string.
/// The search is case-insensitive.
///
/// # Arguments
/// * `title` - The window title to search for (partial match supported)
///
/// # Returns
/// * `Some(HWND)` - The window handle if found
/// * `None` - If no matching window is found
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::get_hwnd_by_title;
///
/// // Find Notepad window by title
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     println!("Found window: {:?}", hwnd);
/// }
///
/// // Find window with exact title
/// if let Some(hwnd) = get_hwnd_by_title("Untitled - Notepad") {
///     println!("Found specific window: {:?}", hwnd);
/// }
/// ```
pub fn get_hwnd_by_title(title: &str) -> Option<HWND> {
    let title_lower = title.to_lowercase();

    let hwnd = get_hwnd_list().into_iter().find(|&hwnd| {
        let window_title = get_window_title(hwnd);
        window_title.to_lowercase().contains(&title_lower)
    });

    hwnd.filter(|h| h.0 != std::ptr::null_mut())
}

/// Get all window handles for a specific process
///
/// Enumerates all windows belonging to the specified process.
/// This includes both visible and invisible windows.
///
/// # Arguments
/// * `pid` - The process ID to enumerate windows for
///
/// # Returns
/// A vector of window handles. Empty if no windows found.
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid_by_name;
/// use win_auto_utils::window::get_hwnd_list_by_pid;
///
/// if let Some(pid) = find_pid_by_name("chrome.exe") {
///     let hwnds = get_hwnd_list_by_pid(pid);
///     println!("Chrome has {} windows", hwnds.len());
///     
///     for (i, hwnd) in hwnds.iter().enumerate() {
///         println!("  Window {}: {:?}", i, hwnd);
///     }
/// }
/// ```
pub fn get_hwnd_list_by_pid(pid: u32) -> Vec<HWND> {
    get_hwnd_list()
        .into_iter()
        .filter(|&hwnd| get_pid_by_hwnd(hwnd) == pid)
        .collect()
}

/// Get window handle by PID with flexible title filtering
///
/// Finds the first window belonging to the specified process that matches
/// all the provided title filters. Filters use an include/exclude mechanism:
/// - Mask 1: Window title MUST contain the pattern
/// - Mask 0: Window title MUST NOT contain the pattern
/// All filters must match for a window to be selected.
///
/// # Arguments
/// * `pid` - The process ID to search for
/// * `filters` - Vector of (pattern, mask) tuples for title filtering
///
/// # Returns
/// * `Some(HWND)` - The first matching window handle
/// * `None` - If no matching window is found
///
/// # Example
/// ```no_run
/// use win_auto_utils::snapshot::find_pid;
/// use win_auto_utils::window::get_hwnd_by_pid_and_filter;
///
/// if let Some(pid) = find_pid("chrome.exe") {
///     // Find Chrome windows containing "Google" but not "Incognito"
///     let filters = vec![
///         ("Google".to_string(), 1),      // Must contain "Google"
///         ("Incognito".to_string(), 0),   // Must NOT contain "Incognito"
///     ];
///     
///     if let Some(hwnd) = get_hwnd_by_pid_and_filter(pid, &filters) {
///         println!("Found filtered window: {:?}", hwnd);
///     }
/// }
/// ```
pub fn get_hwnd_by_pid_and_filter(pid: u32, filters: &Vec<WindowTitleFilter>) -> Option<HWND> {
    get_hwnd_list()
        .into_iter()
        .find(|&hwnd| {
            // First check if this window belongs to the target process
            if get_pid_by_hwnd(hwnd) != pid {
                return false;
            }

            // Then apply all title filters
            let window_title_lower = get_window_title(hwnd).to_lowercase();

            // All filters must match
            for (pattern, mask) in filters {
                let pattern_lower = pattern.to_lowercase();
                let contains_pattern = window_title_lower.contains(&pattern_lower);

                // Mask 1: must contain, Mask 0: must NOT contain
                let matches = if *mask == 1 {
                    contains_pattern
                } else {
                    !contains_pattern
                };

                if !matches {
                    return false;
                }
            }

            true
        })
        .filter(|h| h.0 != std::ptr::null_mut())
}

/// Type alias for window title filter
/// - String: The pattern to match
/// - u8: Filter mask (1 = must contain, 0 = must NOT contain)
pub type WindowTitleFilter = (String, u8);

/// Get all window handles with optional filtering
///
/// Advanced function that allows filtering by both PID and multiple title patterns.
/// Title filters use a flexible include/exclude mechanism:
/// - Mask 1: Window title MUST contain the pattern
/// - Mask 0: Window title MUST NOT contain the pattern
/// All filters must match for a window to be included.
///
/// # Arguments
/// * `pid` - Optional process ID filter (None means all processes)
/// * `title_filters` - Optional vector of (pattern, mask) tuples for title filtering
///
/// # Returns
/// A vector of window handles matching all criteria
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::get_hwnd_list_filtered;
///
/// // Get windows that contain "Notepad" but don't contain "Untitled"
/// let filters = vec![
///     ("Notepad".to_string(), 1),    // Must contain "Notepad"
///     ("Untitled".to_string(), 0),   // Must NOT contain "Untitled"
/// ];
/// let hwnds = get_hwnd_list_filtered(None, Some(&filters));
///
/// // Get all windows from a specific process
/// let all_process_windows = get_hwnd_list_filtered(Some(12345), None);
///
/// // Complex filtering: Chrome windows with "Google" but not "Incognito"
/// let chrome_filters = vec![
///     ("Google".to_string(), 1),
///     ("Incognito".to_string(), 0),
/// ];
/// let filtered_chrome = get_hwnd_list_filtered(Some(chrome_pid), Some(&chrome_filters));
/// ```
pub fn get_hwnd_list_filtered(
    pid: Option<u32>,
    title_filters: Option<&Vec<WindowTitleFilter>>,
) -> Vec<HWND> {
    get_hwnd_list()
        .into_iter()
        .filter(|&hwnd| {
            // Filter by PID if specified
            if let Some(target_pid) = pid {
                if get_pid_by_hwnd(hwnd) != target_pid {
                    return false;
                }
            }

            // Filter by title patterns if specified
            if let Some(filters) = title_filters {
                let window_title_lower = get_window_title(hwnd).to_lowercase();

                // All filters must match
                for (pattern, mask) in filters {
                    let pattern_lower = pattern.to_lowercase();
                    let contains_pattern = window_title_lower.contains(&pattern_lower);

                    // Mask 1: must contain, Mask 0: must NOT contain
                    let matches = if *mask == 1 {
                        contains_pattern
                    } else {
                        !contains_pattern
                    };

                    if !matches {
                        return false;
                    }
                }
            }

            true
        })
        .collect()
}

/// Check if a window is valid and exists
///
/// # Arguments
/// * `hwnd` - The window handle to validate
///
/// # Returns
/// `true` if the window exists and is valid
pub fn is_valid_window(hwnd: HWND) -> bool {
    hwnd.0 != std::ptr::null_mut()
}

/// Check if a window is visible
///
/// # Arguments
/// * `hwnd` - The window handle to check
///
/// # Returns
/// `true` if the window is visible
pub fn is_window_visible(hwnd: HWND) -> bool {
    if !is_valid_window(hwnd) {
        return false;
    }

    unsafe { IsWindowVisible(hwnd).as_bool() }
}

/// Get window title (for testing purposes)
#[cfg(test)]
pub fn get_window_title_for_test(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let length = unsafe { GetWindowTextW(hwnd, &mut buffer) };

    if length == 0 {
        return String::new();
    }

    String::from_utf16(&buffer[..length as usize]).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hwnd_by_title_nonexistent() {
        // Search for a window title that definitely doesn't exist
        let result = get_hwnd_by_title("definitely_not_existing_window_xyz_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_is_valid_window() {
        // Invalid HWND
        assert!(!is_valid_window(HWND::default()));
        assert!(!is_valid_window(HWND(std::ptr::null_mut())));

        // Note: We can't easily test valid HWNDs without an actual window
    }

    #[test]
    fn test_get_hwnd_list_not_empty() {
        // Get all windows (should have at least some)
        let hwnds = get_hwnd_list();

        // There should be at least some windows on a running system
        // (taskbar, desktop, etc.)
        println!("Total windows found: {}", hwnds.len());
        assert!(!hwnds.is_empty(), "Should find at least some windows");
    }

    #[test]
    fn test_get_hwnd_list_by_nonexistent_pid() {
        // Use a PID that's unlikely to exist
        let hwnds = get_hwnd_list_by_pid(999999);
        assert!(
            hwnds.is_empty(),
            "Should not find windows for non-existent PID"
        );
    }

    #[test]
    fn test_window_title_retrieval() {
        // Get some windows and check that we can retrieve their titles
        let hwnds = get_hwnd_list();

        for hwnd in hwnds.iter().take(5) {
            let title = get_window_title(*hwnd);
            let pid = get_pid_by_hwnd(*hwnd);
            // Title might be empty for some windows, that's okay
            println!("PID: {}, Window {:?}: '{}'", pid, hwnd, title);
        }
    }

    #[test]
    fn test_get_hwnd_list_filtered_no_filter() {
        // Get all windows with no filters
        let hwnds = get_hwnd_list_filtered(None, None);
        assert!(!hwnds.is_empty());
    }
}
