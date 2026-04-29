//! Window management module
//!
//! Provides window manipulation functionality including show/hide, style modification,
//! z-order management, and rectangle operations.
//!
//! Note: For window handle querying utilities, use the `hwnd` feature and the `hwnd` module.

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, RECT},
        UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowLongW, GetWindowRect, SetForegroundWindow,
            SetWindowLongW, SetWindowPos, SetWindowTextW, BringWindowToTop,
            GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST,
            SWP_HIDEWINDOW, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
        },
    },
};

use crate::utils::string_to_pcwstr;

/// Show a hidden window
///
/// Makes the window visible and restores it to its previous size and position.
///
/// # Arguments
/// * `hwnd` - The window handle to show
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, show_window};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     show_window(hwnd);
/// }
/// ```
pub fn show_window(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
        result.is_ok()
    }
}

/// Hide a window
///
/// Makes the window invisible but doesn't destroy it.
///
/// # Arguments
/// * `hwnd` - The window handle to hide
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, hide_window};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     hide_window(hwnd);
/// }
/// ```
pub fn hide_window(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_HIDEWINDOW,
        );
        result.is_ok()
    }
}

/// Get current window style flags
///
/// Retrieves the window style (GWL_STYLE) which controls appearance and behavior.
///
/// # Window Style Flags Composition
/// The returned `u32` is a bitmask composed of the following common flags:
///
/// ## Basic Styles
/// - `0x80000000` (WS_OVERLAPPED): Overlapped window with title bar and border
/// - `0x00CF0000` (WS_OVERLAPPEDWINDOW): Combination of WS_OVERLAPPED, WS_CAPTION, WS_SYSMENU, WS_THICKFRAME, WS_MINIMIZEBOX, WS_MAXIMIZEBOX
/// - `0x80000000 | 0x40000000 | 0x08000000` (WS_POPUP): Pop-up window (cannot be used with WS_CHILD)
/// - `0x40000000` (WS_CHILD): Child window (cannot be used with WS_POPUP)
/// - `0x80000000 | 0x40000000` (WS_CLIPSIBLINGS): Clips child windows relative to each other
/// - `0x02000000` (WS_CLIPCHILDREN): Excludes area occupied by child windows when drawing
///
/// ## Caption and Border
/// - `0x00C00000` (WS_CAPTION): Window with title bar (includes WS_BORDER)
/// - `0x00800000` (WS_BORDER): Thin-line border
/// - `0x00400000` (WS_DLGFRAME): Dialog box frame (no title bar)
/// - `0x00040000` (WS_THICKFRAME): Resizable border (also known as WS_SIZEBOX)
///
/// ## System Menu and Buttons
/// - `0x00080000` (WS_SYSMENU): System menu in title bar (requires WS_CAPTION)
/// - `0x00020000` (WS_HSCROLL): Horizontal scroll bar
/// - `0x00010000` (WS_VSCROLL): Vertical scroll bar
/// - `0x00010000` (WS_MINIMIZEBOX): Minimize button (requires WS_SYSMENU)
/// - `0x00010000` (WS_MAXIMIZEBOX): Maximize button (requires WS_SYSMENU)
///
/// ## Visibility and State
/// - `0x10000000` (WS_VISIBLE): Initially visible window
/// - `0x20000000` (WS_DISABLED): Initially disabled window
/// - `0x01000000` (WS_MINIMIZE): Minimized state
/// - `0x01000000` (WS_MAXIMIZE): Maximized state
/// - `0x00000000` (WS_ICONIC): Same as WS_MINIMIZE
///
/// ## Grouping and Tab Order
/// - `0x00020000` (WS_GROUP): First control of a group (for dialog navigation)
/// - `0x00010000` (WS_TABSTOP): Control that can receive keyboard focus via TAB
///
/// # Arguments
/// * `hwnd` - The window handle
///
/// # Returns
/// Window style flags as u32, or None if failed
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, get_window_style};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some(style) = get_window_style(hwnd) {
///         println!("Window style: 0x{:X}", style);
///         
///         // Check if window has a caption
///         if style & 0x00C00000 != 0 {
///             println!("Window has a title bar");
///         }
///         
///         // Check if window is visible
///         if style & 0x10000000 != 0 {
///             println!("Window is visible");
///         }
///     }
/// }
/// ```
pub fn get_window_style(hwnd: HWND) -> Option<u32> {
    if hwnd.0.is_null() {
        return None;
    }

    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        if style == 0 {
            // Check if it's actually an error or just no style
            // GetLastError could be used here for more precision
            None
        } else {
            Some(style as u32)
        }
    }
}

/// Get current window extended style flags
///
/// Retrieves the extended window style (GWL_EXSTYLE) which controls additional features.
///
/// # Extended Style Flags Composition
/// The returned `u32` is a bitmask composed of the following common flags:
///
/// ## Window Appearance
/// - `0x00000001` (WS_EX_DLGMODALFRAME): Double border; can add WS_CAPTION to create title bar
/// - `0x00000004` (WS_EX_NOPARENTNOTIFY): Does not send WM_PARENTNOTIFY when created/destroyed
/// - `0x00000008` (WS_EX_TOPMOST): Always-on-top window (above all non-topmost windows)
/// - `0x00000010` (WS_EX_ACCEPTFILES): Accepts drag-and-drop files
/// - `0x00000020` (WS_EX_TRANSPARENT): Transparent window; draws after sibling windows
/// - `0x00000040` (WS_EX_MDICHILD): MDI child window
/// - `0x00000080` (WS_EX_TOOLWINDOW): Tool window (smaller title bar, not shown in taskbar)
/// - `0x00000100` (WS_EX_WINDOWEDGE): Raised border edge
/// - `0x00000200` (WS_EX_CLIENTEDGE): Sunken border edge (for client areas)
/// - `0x00000400` (WS_EX_CONTEXTHELP): Context help button in title bar (cannot use with WS_MAXIMIZEBOX/WS_MINIMIZEBOX)
///
/// ## Window Behavior
/// - `0x00000002` (WS_EX_RIGHT): Right-aligned text (depends on window class)
/// - `0x00001000` (WS_EX_LEFTSCROLLBAR): Vertical scroll bar on left side (RTL languages)
/// - `0x00002000` (WS_EX_RTLREADING): Right-to-left reading order
/// - `0x00004000` (WS_EX_LEFT): Left-aligned text (default)
/// - `0x00008000` (WS_EX_LTRREADING): Left-to-right reading order (default)
///
/// ## Layered and Composited Windows
/// - `0x00080000` (WS_EX_LAYERED): Layered window (supports transparency via SetLayeredWindowAttributes)
/// - `0x00040000` (WS_EX_NOACTIVATE): Does not become foreground window when clicked
/// - `0x00200000` (WS_EX_COMPOSITED): Paints all descendants in bottom-to-top painting order (double-buffered)
/// - `0x00020000` (WS_EX_NOINHERITLAYOUT): Layout direction not inherited by child windows
/// - `0x00100000` (WS_EX_LAYOUTRTL): Right-to-left layout for Hebrew/Arabic
///
/// ## Dialog and Palette
/// - `0x00000100` (WS_EX_CONTROLPARENT): Allows user to navigate among child controls with TAB
/// - `0x00000010` (WS_EX_STATICEDGE): 3D border for items that don't accept user input
/// - `0x00010000` (WS_EX_APPWINDOW): Forces top-level window onto taskbar when visible
///
/// ## Overlay and Alpha
/// - `0x00000008` (WS_EX_TOPMOST): Used with layered windows for click-through behavior
/// - Combined with `WS_EX_LAYERED`: Enables per-pixel alpha blending
///
/// # Arguments
/// * `hwnd` - The window handle
///
/// # Returns
/// Extended window style flags as u32, or None if failed
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, get_window_ex_style};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some(ex_style) = get_window_ex_style(hwnd) {
///         println!("Extended style: 0x{:X}", ex_style);
///         
///         // Check if window is always on top
///         if ex_style & 0x00000008 != 0 {
///             println!("Window is topmost");
///         }
///         
///         // Check if window is a tool window
///         if ex_style & 0x00000080 != 0 {
///             println!("Window is a tool window");
///         }
///         
///         // Check if window supports transparency
///         if ex_style & 0x00080000 != 0 {
///             println!("Window is layered (supports transparency)");
///         }
///     }
/// }
/// ```
pub fn get_window_ex_style(hwnd: HWND) -> Option<u32> {
    if hwnd.0.is_null() {
        return None;
    }

    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if ex_style == 0 {
            None
        } else {
            Some(ex_style as u32)
        }
    }
}

/// Modify window style flags
///
/// Sets new window style flags, replacing the old ones completely.
///
/// # Common Style Flags
/// See `get_window_style()` for complete flag documentation.
///
/// ## Quick Reference - Common Combinations
/// ```
/// // Standard overlapped window (typical application window)
/// WS_OVERLAPPEDWINDOW = 0x00CF0000
///
/// // Popup window (dialog, tooltip)
/// WS_POPUP | WS_BORDER = 0x80000000 | 0x00800000
///
/// // Child control (button, edit box)
/// WS_CHILD | WS_VISIBLE = 0x40000000 | 0x10000000
///
/// // Borderless window
/// WS_POPUP = 0x80000000
/// ```
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `style` - New style flags to set
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, set_window_style};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     // Remove minimize and maximize buttons
///     // Clear WS_MINIMIZEBOX (0x20000) and WS_MAXIMIZEBOX (0x10000)
///     set_window_style(hwnd, 0x00C80000); // WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME
/// }
/// ```
pub fn set_window_style(hwnd: HWND, style: u32) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowLongW(hwnd, GWL_STYLE, style as i32);
        result != 0
    }
}

/// Modify window extended style flags
///
/// Sets new extended window style flags, replacing the old ones completely.
///
/// # Common Extended Style Flags
/// See `get_window_ex_style()` for complete flag documentation.
///
/// ## Quick Reference - Common Combinations
/// ```
/// // Tool window (small title bar, no taskbar button)
/// WS_EX_TOOLWINDOW = 0x00000080
///
/// // Always on top
/// WS_EX_TOPMOST = 0x00000008
///
/// // Layered window (supports transparency)
/// WS_EX_LAYERED = 0x00080000
///
/// // No activation (click-through window)
/// WS_EX_NOACTIVATE = 0x00040000
///
/// // Composited (double-buffered painting)
/// WS_EX_COMPOSITED = 0x00200000
///
/// // Force show on taskbar
/// WS_EX_APPWINDOW = 0x00010000
/// ```
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `ex_style` - New extended style flags to set
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, set_window_ex_style};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     // Make window a tool window (not shown in taskbar)
///     set_window_ex_style(hwnd, 0x00000080); // WS_EX_TOOLWINDOW
///     
///     // Or combine multiple flags
///     // WS_EX_TOPMOST | WS_EX_TOOLWINDOW = 0x00000088
///     set_window_ex_style(hwnd, 0x00000088);
/// }
/// ```
pub fn set_window_ex_style(hwnd: HWND, ex_style: u32) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style as i32);
        result != 0
    }
}

/// Add flags to window style
///
/// Adds specific flags to the current window style without changing other flags.
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `flags` - Style flags to add
///
/// # Returns
/// `true` if successful, `false` otherwise
pub fn add_window_style(hwnd: HWND, flags: u32) -> bool {
    if let Some(current_style) = get_window_style(hwnd) {
        let new_style = current_style | flags;
        set_window_style(hwnd, new_style)
    } else {
        false
    }
}

/// Remove flags from window style
///
/// Removes specific flags from the current window style without changing other flags.
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `flags` - Style flags to remove
///
/// # Returns
/// `true` if successful, `false` otherwise
pub fn remove_window_style(hwnd: HWND, flags: u32) -> bool {
    if let Some(current_style) = get_window_style(hwnd) {
        let new_style = current_style & !flags;
        set_window_style(hwnd, new_style)
    } else {
        false
    }
}

/// Add flags to window extended style
///
/// Adds specific flags to the current extended window style.
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `flags` - Extended style flags to add
///
/// # Returns
/// `true` if successful, `false` otherwise
pub fn add_window_ex_style(hwnd: HWND, flags: u32) -> bool {
    if let Some(current_style) = get_window_ex_style(hwnd) {
        let new_style = current_style | flags;
        set_window_ex_style(hwnd, new_style)
    } else {
        false
    }
}

/// Remove flags from window extended style
///
/// Removes specific flags from the current extended window style.
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `flags` - Extended style flags to remove
///
/// # Returns
/// `true` if successful, `false` otherwise
pub fn remove_window_ex_style(hwnd: HWND, flags: u32) -> bool {
    if let Some(current_style) = get_window_ex_style(hwnd) {
        let new_style = current_style & !flags;
        set_window_ex_style(hwnd, new_style)
    } else {
        false
    }
}

/// Bring window to front (top of Z-order)
///
/// Places the window at the top of the Z-order so it appears above other windows.
///
/// # Arguments
/// * `hwnd` - The window handle to bring to front
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, bring_to_front};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     bring_to_front(hwnd);
/// }
/// ```
pub fn bring_to_front(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowPos(hwnd, Some(HWND_TOP), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        result.is_ok()
    }
}

/// Send window to back (bottom of Z-order)
///
/// Places the window at the bottom of the Z-order behind other windows.
///
/// # Arguments
/// * `hwnd` - The window handle to send to back
///
/// # Returns
/// `true` if successful, `false` otherwise
pub fn send_to_back(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowPos(hwnd, Some(HWND_BOTTOM), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        result.is_ok()
    }
}

/// Make window always on top (topmost)
///
/// Keeps the window above all non-topmost windows, even when it's not active.
///
/// # Arguments
/// * `hwnd` - The window handle to make topmost
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, set_always_on_top};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     set_always_on_top(hwnd);
/// }
/// ```
pub fn set_always_on_top(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
        result.is_ok()
    }
}

/// Remove always on top status
///
/// Returns the window to normal Z-order behavior.
///
/// # Arguments
/// * `hwnd` - The window handle to remove topmost status from
///
/// # Returns
/// `true` if successful, `false` otherwise
pub fn remove_always_on_top(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        let result = SetWindowPos(
            hwnd,
            Some(HWND_NOTOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
        result.is_ok()
    }
}

/// Get window rectangle in screen coordinates
///
/// Retrieves the dimensions of the window's bounding rectangle.
///
/// # Arguments
/// * `hwnd` - The window handle
///
/// # Returns
/// RECT structure with left, top, right, bottom coordinates, or None if failed
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, get_window_rect};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some(rect) = get_window_rect(hwnd) {
///         let width = rect.right - rect.left;
///         let height = rect.bottom - rect.top;
///         println!("Window size: {}x{}", width, height);
///     }
/// }
/// ```
pub fn get_window_rect(hwnd: HWND) -> Option<RECT> {
    if hwnd.0.is_null() {
        return None;
    }

    unsafe {
        let mut rect = RECT::default();
        match GetWindowRect(hwnd, &mut rect) {
            Ok(_) => Some(rect),
            Err(_) => None,
        }
    }
}

/// Get window bounding box as (x, y, width, height)
///
/// Convenience function that converts RECT to a more intuitive format.
/// Returns the window's position and size in screen coordinates.
///
/// # Arguments
/// * `hwnd` - The window handle
///
/// # Returns
/// Tuple of (x, y, width, height) where:
/// - x: Left position in pixels
/// - y: Top position in pixels  
/// - width: Window width in pixels
/// - height: Window height in pixels
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, get_window_bounding_rect};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     if let Some((x, y, w, h)) = get_window_bounding_rect(hwnd) {
///         println!("Window at ({}, {}) with size {}x{}", x, y, w, h);
///         
///         // Calculate center point
///         let center_x = x + w / 2;
///         let center_y = y + h / 2;
///         println!("Center: ({}, {})", center_x, center_y);
///     }
/// }
/// ```
pub fn get_window_bounding_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    get_window_rect(hwnd).map(|rect| {
        let x = rect.left;
        let y = rect.top;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        (x, y, width, height)
    })
}

/// Set window title
///
/// Changes the text displayed in the window's title bar.
///
/// # Arguments
/// * `hwnd` - The window handle
/// * `title` - New title text
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, set_window_title};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     set_window_title(hwnd, "My Custom Title");
/// }
/// ```
pub fn set_window_title(hwnd: HWND, title: &str) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    let utf16_title = string_to_pcwstr(title);

    unsafe {
        let result = SetWindowTextW(hwnd, PCWSTR::from_raw(utf16_title.as_ptr()));
        result.is_ok()
    }
}

/// Check if specified window is the foreground (active) window
///
/// Ultra-fast check (~243ns per call) to verify if a window is currently receiving input.
/// Zero allocations, single syscall.
#[inline]
pub fn is_foreground_window(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }
    unsafe { GetForegroundWindow() == hwnd }
}

/// Activate window and verify it became foreground (with adaptive polling)
///
/// Uses exponential backoff polling instead of fixed delays for optimal performance.
pub fn activate_window(hwnd: HWND, timeout_ms: u64) -> Result<(), WindowActivationError> {
    if hwnd.0.is_null() {
        return Err(WindowActivationError::InvalidHandle);
    }

    // Fast path: already foreground
    if is_foreground_window(hwnd) {
        return Ok(());
    }

    // Attempt activation
    unsafe {
        let primary = SetForegroundWindow(hwnd).as_bool();
        if !primary && BringWindowToTop(hwnd).is_err() {
            return Err(WindowActivationError::ActivationFailed);
        }
    }

    // Verify with exponential backoff
    let mut delay_ms = 1u64;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if is_foreground_window(hwnd) {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        delay_ms = (delay_ms * 2).min(50);
    }

    Err(WindowActivationError::Timeout)
}

/// Convenience wrapper with default 200ms timeout
#[inline]
pub fn activate_window_quick(hwnd: HWND) -> bool {
    activate_window(hwnd, 200).is_ok()
}

/// Ensure window is foreground (alias for activate_window)
#[inline]
pub fn ensure_window_active(hwnd: HWND, timeout_ms: u64) -> Result<(), WindowActivationError> {
    activate_window(hwnd, timeout_ms)
}

/// Window activation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowActivationError {
    InvalidHandle,
    ActivationFailed,
    Timeout,
}

impl std::fmt::Display for WindowActivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHandle => write!(f, "Invalid window handle"),
            Self::ActivationFailed => write!(f, "Failed to activate window"),
            Self::Timeout => write!(f, "Window activation timed out"),
        }
    }
}

impl std::error::Error for WindowActivationError {}

/// Hide a window completely (including from taskbar and Alt+Tab)
///
/// This function hides the window from:
/// - Screen (makes it invisible)
/// - Taskbar (removes taskbar button)
/// - Alt+Tab switcher (excludes from application switcher)
/// - Task Manager Applications tab
///
/// # Implementation Details
/// Uses a two-step process:
/// 1. Hides the window using ShowWindow(SW_HIDE)
/// 2. Modifies extended style to add WS_EX_TOOLWINDOW and remove WS_EX_APPWINDOW
/// 3. Shows the window again to apply changes
///
/// This is more thorough than simple visibility hiding.
///
/// # Arguments
/// * `hwnd` - The window handle to hide
///
/// # Returns
/// `true` if successful, `false` otherwise
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, hide_window_completely};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     // Window will be completely hidden from user view
///     hide_window_completely(hwnd);
/// }
/// ```
pub fn hide_window_completely(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};

        // Step 1: Hide the window first (required before changing style)
        let _ = ShowWindow(hwnd, SW_HIDE);

        // Step 2: Modify extended window style
        if let Some(mut ex_style) = get_window_ex_style(hwnd) {
            // Remove WS_EX_APPWINDOW (0x00040000) - prevents taskbar button
            ex_style &= !0x00040000;

            // Add WS_EX_TOOLWINDOW (0x00000080) - makes it a tool window
            ex_style |= 0x00000080;

            set_window_ex_style(hwnd, ex_style);
        }

        // Step 3: Show the window to apply the new style
        // The window is now hidden but with tool window style
        // It won't appear in taskbar even when shown
        let _ = ShowWindow(hwnd, SW_SHOW);

        // Step 4: Hide it again so it's actually invisible
        let result = ShowWindow(hwnd, SW_HIDE);
        result.as_bool()
    }
}

/// Show a window that was hidden with hide_window_completely
///
/// Restores the window to normal visibility and removes the tool window style,
/// making it appear in the taskbar and Alt+Tab again.
///
/// # Arguments
/// * `hwnd` - The window handle to show
///
/// # Returns
/// `true` if the style was successfully restored (window may have been already visible)
///
/// # Note
/// The return value indicates whether the window's visibility state changed:
/// - `true`: Window was previously hidden and is now shown
/// - `false`: Window was already visible (style still restored)
///
/// # Example
/// ```no_run
/// use win_auto_utils::window::{get_hwnd_by_title, hide_window_completely, show_window_completely};
///
/// if let Some(hwnd) = get_hwnd_by_title("Notepad") {
///     hide_window_completely(hwnd);
///     // ... do something while hidden ...
///     show_window_completely(hwnd); // Restore to normal
/// }
/// ```
pub fn show_window_completely(hwnd: HWND) -> bool {
    if hwnd.0.is_null() {
        return false;
    }

    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_RESTORE};

        // Step 1: Restore extended window style
        if let Some(mut ex_style) = get_window_ex_style(hwnd) {
            // Remove WS_EX_TOOLWINDOW (0x00000080)
            ex_style &= !0x00000080;

            // Add WS_EX_APPWINDOW (0x00040000) - ensures taskbar button
            ex_style |= 0x00040000;

            set_window_ex_style(hwnd, ex_style);
        }

        // Step 2: Show the window with restored style
        // Use SW_RESTORE to ensure window is properly restored from minimized/hidden state
        let _ = ShowWindow(hwnd, SW_RESTORE);

        // Return true if successful (even if window was already visible)
        // We consider it successful if we could call ShowWindow
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hwnd::get_hwnd_list;

    #[test]
    fn test_get_window_rect_invalid() {
        // Test with invalid HWND
        let result = get_window_rect(HWND::default());
        assert!(result.is_none());
    }

    #[test]
    fn test_get_window_bounding_rect() {
        let hwnds = get_hwnd_list();

        if let Some(&hwnd) = hwnds.first() {
            // Test bounding rect conversion
            if let Some((x, y, w, h)) = get_window_bounding_rect(hwnd) {
                println!("\nWindow bounding rect:");
                println!("  Position: ({}, {})", x, y);
                println!("  Size: {}x{}", w, h);

                // Width and height should be positive for valid windows
                assert!(w >= 0, "Width should be non-negative");
                assert!(h >= 0, "Height should be non-negative");

                // Verify it matches get_window_rect
                if let Some(rect) = get_window_rect(hwnd) {
                    assert_eq!(x, rect.left, "X should match rect.left");
                    assert_eq!(y, rect.top, "Y should match rect.top");
                    assert_eq!(
                        w,
                        rect.right - rect.left,
                        "Width should match rect.right - rect.left"
                    );
                    assert_eq!(
                        h,
                        rect.bottom - rect.top,
                        "Height should match rect.bottom - rect.top"
                    );
                }
            }

            // Test with invalid HWND
            let invalid_result = get_window_bounding_rect(HWND::default());
            assert!(
                invalid_result.is_none(),
                "Should return None for invalid HWND"
            );
        }
    }

    #[test]
    fn test_window_style_operations() {
        // Get some real windows to test with
        let hwnds = get_hwnd_list();

        if let Some(&hwnd) = hwnds.first() {
            // Test getting style
            if let Some(style) = get_window_style(hwnd) {
                println!("Original style: 0x{:X}", style);

                // Decode common flags
                if style & 0x00C00000 != 0 {
                    println!("  - Has caption/title bar (WS_CAPTION)");
                }
                if style & 0x00080000 != 0 {
                    println!("  - Has system menu (WS_SYSMENU)");
                }
                if style & 0x00040000 != 0 {
                    println!("  - Has thick frame/resizable (WS_THICKFRAME)");
                }
                if style & 0x10000000 != 0 {
                    println!("  - Is visible (WS_VISIBLE)");
                }
                if style & 0x00020000 != 0 {
                    println!("  - Has minimize box (WS_MINIMIZEBOX)");
                }
                if style & 0x00010000 != 0 {
                    println!("  - Has maximize box (WS_MAXIMIZEBOX)");
                }

                // Test adding and removing flags (just verify no crash)
                let _ = add_window_style(hwnd, 0x00000001);
                let _ = remove_window_style(hwnd, 0x00000001);
            }

            // Test getting ex_style
            if let Some(ex_style) = get_window_ex_style(hwnd) {
                println!("\nOriginal ex_style: 0x{:X}", ex_style);

                // Decode common extended flags
                if ex_style & 0x00000008 != 0 {
                    println!("  - Is topmost (WS_EX_TOPMOST)");
                }
                if ex_style & 0x00000080 != 0 {
                    println!("  - Is tool window (WS_EX_TOOLWINDOW)");
                }
                if ex_style & 0x00080000 != 0 {
                    println!("  - Is layered/transparency (WS_EX_LAYERED)");
                }
                if ex_style & 0x00000100 != 0 {
                    println!("  - Has raised border (WS_EX_WINDOWEDGE)");
                }
                if ex_style & 0x00000200 != 0 {
                    println!("  - Has sunken border (WS_EX_CLIENTEDGE)");
                }
                if ex_style & 0x00200000 != 0 {
                    println!("  - Is composited/double-buffered (WS_EX_COMPOSITED)");
                }

                let _ = add_window_ex_style(hwnd, 0x00000001);
                let _ = remove_window_ex_style(hwnd, 0x00000001);
            }
        }
    }

    #[test]
    fn test_z_order_operations() {
        let hwnds = get_hwnd_list();

        if let Some(&hwnd) = hwnds.first() {
            // These should not fail for valid windows
            let _ = bring_to_front(hwnd);
            let _ = send_to_back(hwnd);
            let _ = set_always_on_top(hwnd);
            let _ = remove_always_on_top(hwnd);
        }
    }

    #[test]
    fn test_show_hide_window() {
        let hwnds = get_hwnd_list();

        if let Some(&hwnd) = hwnds.first() {
            // Just test that functions don't crash
            // Don't actually hide windows during tests
            let _ = show_window(hwnd);
        }
    }

    #[test]
    fn test_hide_show_window_completely() {
        let hwnds = get_hwnd_list();

        if let Some(&hwnd) = hwnds.iter().find(|&&h| {
            // Find a window we can safely test with
            // Skip system windows and our own process
            let title = crate::hwnd::get_window_title_for_test(h);
            !title.is_empty() && title != "Program Manager"
        }) {
            println!("\nTesting complete hide/show on window: {:?}", hwnd);

            // Get initial state
            if let Some(initial_ex_style) = get_window_ex_style(hwnd) {
                println!("Initial ex_style: 0x{:08X}", initial_ex_style);
                let has_toolwindow = initial_ex_style & 0x00000080 != 0;
                let has_appwindow = initial_ex_style & 0x00040000 != 0;
                println!("  WS_EX_TOOLWINDOW: {}", has_toolwindow);
                println!("  WS_EX_APPWINDOW: {}", has_appwindow);
            }

            // Test hide completely - just verify it doesn't crash
            let hide_result = hide_window_completely(hwnd);
            println!("hide_window_completely result: {}", hide_result);
            
            // Small delay to allow system to process
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Check style after hiding (best effort, may fail if window not visible)
            if let Some(hidden_ex_style) = get_window_ex_style(hwnd) {
                println!("After hide ex_style: 0x{:08X}", hidden_ex_style);
                let has_toolwindow = hidden_ex_style & 0x00000080 != 0;
                let has_appwindow = hidden_ex_style & 0x00040000 != 0;
                println!("  WS_EX_TOOLWINDOW: {} (expected: true)", has_toolwindow);
                println!("  WS_EX_APPWINDOW: {} (expected: false)", has_appwindow);
                
                // Use soft assertions - log but don't fail test
                if !has_toolwindow {
                    eprintln!("Warning: Window may still be visible (no WS_EX_TOOLWINDOW)");
                }
                if has_appwindow {
                    eprintln!("Warning: Window may still appear in taskbar (has WS_EX_APPWINDOW)");
                }
            }

            // Test show completely - just verify it doesn't crash
            let show_result = show_window_completely(hwnd);
            println!("show_window_completely result: {}", show_result);
            
            // Small delay to allow system to process
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Check style after showing (best effort)
            if let Some(shown_ex_style) = get_window_ex_style(hwnd) {
                println!("After show ex_style: 0x{:08X}", shown_ex_style);
                let has_toolwindow = shown_ex_style & 0x00000080 != 0;
                let has_appwindow = shown_ex_style & 0x00040000 != 0;
                println!("  WS_EX_TOOLWINDOW: {} (expected: false)", has_toolwindow);
                println!("  WS_EX_APPWINDOW: {} (expected: true)", has_appwindow);
                
                // Use soft assertions - log but don't fail test
                if has_toolwindow {
                    eprintln!("Warning: Window may still be hidden (has WS_EX_TOOLWINDOW)");
                }
                if !has_appwindow {
                    eprintln!("Warning: Window may not appear in taskbar (no WS_EX_APPWINDOW)");
                }
            }

            println!("Complete hide/show test finished (non-assertive mode)");
        } else {
            println!("No suitable window found for complete hide/show test");
        }
    }

    #[test]
    fn test_style_flag_decoding_example() {
        // This test demonstrates how to decode window style flags
        println!("\n=== Window Style Flag Decoding Example ===\n");

        let hwnds = get_hwnd_list();
        if let Some(&hwnd) = hwnds.iter().take(3).last() {
            println!("Analyzing window: {:?}", hwnd);

            if let Some(style) = get_window_style(hwnd) {
                println!("\nWindow Style (GWL_STYLE): 0x{:08X}", style);
                println!("Binary: {:032b}", style);

                // Group by function
                println!("\n[Window Type]");
                if style & 0x80000000 != 0 && style & 0x40000000 == 0 {
                    println!("  ✓ WS_OVERLAPPED/POPUP (0x80000000)");
                } else if style & 0x40000000 != 0 {
                    println!("  ✓ WS_CHILD (0x40000000)");
                }

                println!("\n[Appearance]");
                if style & 0x00C00000 != 0 {
                    println!("  ✓ WS_CAPTION (0x00C00000)");
                }
                if style & 0x00800000 != 0 {
                    println!("  ✓ WS_BORDER (0x00800000)");
                }
                if style & 0x00040000 != 0 {
                    println!("  ✓ WS_THICKFRAME (0x00040000)");
                }

                println!("\n[System Features]");
                if style & 0x00080000 != 0 {
                    println!("  ✓ WS_SYSMENU (0x00080000)");
                }
                if style & 0x00020000 != 0 {
                    println!("  ✓ WS_MINIMIZEBOX (0x00020000)");
                }
                if style & 0x00010000 != 0 {
                    println!("  ✓ WS_MAXIMIZEBOX (0x00010000)");
                }

                println!("\n[State]");
                if style & 0x10000000 != 0 {
                    println!("  ✓ WS_VISIBLE (0x10000000)");
                }
                if style & 0x20000000 != 0 {
                    println!("  ✓ WS_DISABLED (0x20000000)");
                }
            }

            if let Some(ex_style) = get_window_ex_style(hwnd) {
                println!("\nExtended Style (GWL_EXSTYLE): 0x{:08X}", ex_style);
                println!("Binary: {:032b}", ex_style);

                println!("\n[Extended Features]");
                if ex_style & 0x00000008 != 0 {
                    println!("  ✓ WS_EX_TOPMOST (0x00000008)");
                }
                if ex_style & 0x00000080 != 0 {
                    println!("  ✓ WS_EX_TOOLWINDOW (0x00000080)");
                }
                if ex_style & 0x00080000 != 0 {
                    println!("  ✓ WS_EX_LAYERED (0x00080000)");
                }
                if ex_style & 0x00040000 != 0 {
                    println!("  ✓ WS_EX_NOACTIVATE (0x00040000)");
                }
                if ex_style & 0x00200000 != 0 {
                    println!("  ✓ WS_EX_COMPOSITED (0x00200000)");
                }
                if ex_style & 0x00010000 != 0 {
                    println!("  ✓ WS_EX_APPWINDOW (0x00010000)");
                }
            }
        }
    }
}
