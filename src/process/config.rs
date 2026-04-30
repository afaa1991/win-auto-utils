//! Process configuration module
//!
//! Defines the configuration layer for process management.
//! Configurations are immutable once created and can be cloned for reuse.

use std::fmt;

/// Device Context acquisition mode
///
/// Specifies how the device context (DC) should be obtained for screen capture operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DCMode {
    /// Standard window DC using `GetWindowDC()`
    Standard = 1,
    
    /// Window client area DC using `GetDC()` with client area
    WindowClient = 2,
    
    /// Desktop DC for full-screen capture
    Desktop = 3,
}

impl fmt::Display for DCMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DCMode::Standard => write!(f, "Standard"),
            DCMode::WindowClient => write!(f, "WindowClient"),
            DCMode::Desktop => write!(f, "Desktop"),
        }
    }
}

impl Default for DCMode {
    fn default() -> Self {
        DCMode::Standard
    }
}

/// Window filter rule type
///
/// Determines whether a window should be included or excluded based on criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterRuleType {
    /// Include windows that match the criteria
    Include,
    /// Exclude windows that match the criteria
    Exclude,
}

/// Window filter criterion
///
/// Represents a single filtering condition for windows.
/// Can be used to include or exclude windows based on various properties.
///
/// # Example
/// ```no_run
/// use win_auto_utils::process::config::WindowFilter;
///
/// // Simple builder-style usage
/// let filter = WindowFilter::new()
///     .exclude_invisible()
///     .include_by_title("Main Window");
///
/// // Or use convenience constructors
/// let filter = WindowFilter::exclude_invisible();
/// ```
#[derive(Debug, Clone)]
pub struct WindowFilter {
    /// List of filter rules (applied in order)
    pub rules: Vec<FilterRule>,
}

/// A single filter rule
#[derive(Debug, Clone)]
pub struct FilterRule {
    /// Whether to include or exclude matching windows
    pub rule_type: FilterRuleType,
    /// The filter criterion
    pub criterion: FilterCriterion,
}

/// Filter criterion types
#[derive(Debug, Clone)]
pub enum FilterCriterion {
    /// Match by window title (partial match, case-insensitive)
    ByWindowTitle {
        title_pattern: String,
        case_sensitive: bool,
    },
    /// Match by visibility (visible only or invisible only)
    ByVisibility(bool), // true = visible, false = invisible
}

impl WindowFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    // ==================== Convenience Constructors ====================
    
    /// Create a filter that excludes invisible/hidden windows
    /// 
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::config::WindowFilter;
    /// let filter = WindowFilter::exclude_invisible();
    /// ```
    pub fn new_exclude_invisible() -> Self {
        Self::new().exclude_invisible()
    }
    
    /// Create a filter that includes only visible windows
    pub fn new_include_visible_only() -> Self {
        Self::new().include_visible_only()
    }
    
    /// Create a filter that includes windows by title pattern
    pub fn new_include_by_title(title_pattern: &str) -> Self {
        Self::new().include_by_title(title_pattern)
    }
    
    // ==================== Include Rules ====================
    
    /// Add an include rule by window title (case-insensitive partial match)
    pub fn include_by_title(mut self, title_pattern: &str) -> Self {
        self.rules.push(FilterRule {
            rule_type: FilterRuleType::Include,
            criterion: FilterCriterion::ByWindowTitle {
                title_pattern: title_pattern.to_string(),
                case_sensitive: false,
            },
        });
        self
    }
    
    /// Add an include rule by exact window title (case-sensitive)
    pub fn include_by_exact_title(mut self, title: &str) -> Self {
        self.rules.push(FilterRule {
            rule_type: FilterRuleType::Include,
            criterion: FilterCriterion::ByWindowTitle {
                title_pattern: title.to_string(),
                case_sensitive: true,
            },
        });
        self
    }
    
    /// Add an include rule for visible windows only
    pub fn include_visible_only(mut self) -> Self {
        self.rules.push(FilterRule {
            rule_type: FilterRuleType::Include,
            criterion: FilterCriterion::ByVisibility(true),
        });
        self
    }
    
    // ==================== Exclude Rules ====================
    
    /// Add an exclude rule by window title pattern
    pub fn exclude_by_title(mut self, title_pattern: &str) -> Self {
        self.rules.push(FilterRule {
            rule_type: FilterRuleType::Exclude,
            criterion: FilterCriterion::ByWindowTitle {
                title_pattern: title_pattern.to_string(),
                case_sensitive: false,
            },
        });
        self
    }
    
    /// Add an exclude rule for invisible/hidden windows
    pub fn exclude_invisible(mut self) -> Self {
        self.rules.push(FilterRule {
            rule_type: FilterRuleType::Exclude,
            criterion: FilterCriterion::ByVisibility(false),
        });
        self
    }
    
    /// Check if this filter has any rules
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for WindowFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Process configuration builder
///
/// Provides a fluent API for building process configurations.
/// This is the recommended way to create `ProcessConfig` instances.
///
/// # Example
/// ```no_run
/// use win_auto_utils::process::config::{ProcessConfig, DCMode};
///
/// // Simple config with just process name
/// let config = ProcessConfig::builder("notepad.exe").build();
///
/// // Complex config with multiple settings
/// let config = ProcessConfig::builder("game.exe")
///     .dc_mode(DCMode::WindowClient)
///     .exclude_invisible()
///     .include_by_title("Game Window")
///     .build();
/// ```
pub struct ProcessConfigBuilder {
    process_name: String,
    dc_mode: DCMode,
    window_filter: WindowFilter,
}

impl ProcessConfigBuilder {
    /// Create a new builder with the target process name
    pub fn new(process_name: &str) -> Self {
        Self {
            process_name: process_name.to_string(),
            dc_mode: DCMode::default(),
            window_filter: WindowFilter::new(),
        }
    }
    
    // ==================== DC Mode Setters (Convenience Methods) ====================
    
    /// Set DC mode to Standard (GetWindowDC)
    /// 
    /// This is the default mode. Uses `GetWindowDC()` to get the device context.
    /// 
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessConfig;
    /// let config = ProcessConfig::builder("app.exe")
    ///     .set_window_mode()
    ///     .build();
    /// ```
    pub fn set_window_mode(mut self) -> Self {
        self.dc_mode = DCMode::Standard;
        self
    }
    
    /// Set DC mode to Window Client (GetDC with client area)
    /// 
    /// Uses `GetDC()` to get the client area device context.
    /// This is often more suitable for game capture and overlay operations.
    /// 
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessConfig;
    /// let config = ProcessConfig::builder("game.exe")
    ///     .set_window_client_mode()
    ///     .build();
    /// ```
    pub fn set_window_client_mode(mut self) -> Self {
        self.dc_mode = DCMode::WindowClient;
        self
    }
    
    /// Set DC mode to Desktop (full-screen capture)
    /// 
    /// Uses desktop DC for full-screen capture operations.
    /// Useful when you need to capture the entire screen.
    /// 
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessConfig;
    /// let config = ProcessConfig::builder("app.exe")
    ///     .set_desktop_mode()
    ///     .build();
    /// ```
    pub fn set_desktop_mode(mut self) -> Self {
        self.dc_mode = DCMode::Desktop;
        self
    }
    
    /// Set the DC mode (advanced usage, requires importing DCMode enum)
    /// 
    /// For most cases, prefer using the convenience methods above:
    /// - `set_window_mode()`
    /// - `set_window_client_mode()`
    /// - `set_desktop_mode()`
    pub fn dc_mode(mut self, mode: DCMode) -> Self {
        self.dc_mode = mode;
        self
    }
    
    /// Set the window filter directly
    pub fn window_filter(mut self, filter: WindowFilter) -> Self {
        self.window_filter = filter;
        self
    }
    
    /// Add an include rule by window title (case-insensitive partial match)
    pub fn include_by_title(mut self, title_pattern: &str) -> Self {
        self.window_filter = self.window_filter.include_by_title(title_pattern);
        self
    }
    
    /// Add an include rule by exact window title (case-sensitive)
    pub fn include_by_exact_title(mut self, title: &str) -> Self {
        self.window_filter = self.window_filter.include_by_exact_title(title);
        self
    }
    
    /// Add an include rule for visible windows only
    pub fn include_visible_only(mut self) -> Self {
        self.window_filter = self.window_filter.include_visible_only();
        self
    }
    
    /// Add an exclude rule by window title pattern
    pub fn exclude_by_title(mut self, title_pattern: &str) -> Self {
        self.window_filter = self.window_filter.exclude_by_title(title_pattern);
        self
    }
    
    /// Add an exclude rule for invisible/hidden windows
    pub fn exclude_invisible(mut self) -> Self {
        self.window_filter = self.window_filter.exclude_invisible();
        self
    }
    
    /// Build the final ProcessConfig
    pub fn build(self) -> ProcessConfig {
        ProcessConfig {
            process_name: self.process_name,
            dc_mode: self.dc_mode,
            window_filter: if self.window_filter.is_empty() {
                None
            } else {
                Some(self.window_filter)
            },
        }
    }
}

/// Process configuration (immutable, clonable)
///
/// Contains all settings that control how the process connection is established.
/// Once created, configurations should not be modified. Clone to create variations.
///
/// # Example
/// ```no_run
/// use win_auto_utils::process::config::{ProcessConfig, DCMode};
///
/// // Use the builder (recommended)
/// let config = ProcessConfig::builder("notepad.exe")
///     .dc_mode(DCMode::WindowClient)
///     .exclude_invisible()
///     .build();
///
/// // Or create directly
/// let config = ProcessConfig::new("game.exe");
/// ```
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Target process name (e.g., "notepad.exe")
    pub process_name: String,
    
    /// Device context acquisition mode
    pub dc_mode: DCMode,
    
    /// Optional window filter for fine-grained selection
    pub window_filter: Option<WindowFilter>,
}

impl ProcessConfig {
    /// Create a new process configuration with just the process name
    pub fn new(process_name: &str) -> Self {
        Self {
            process_name: process_name.to_string(),
            dc_mode: DCMode::default(),
            window_filter: None,
        }
    }
    
    /// Create a builder for this process configuration
    ///
    /// This is the recommended way to create complex configurations.
    ///
    /// # Example
    /// ```no_run
    /// use win_auto_utils::process::ProcessConfig;
    ///
    /// let config = ProcessConfig::builder("game.exe")
    ///     .set_window_client_mode()
    ///     .exclude_invisible()
    ///     .include_by_title("Game Window")
    ///     .build();
    /// ```
    pub fn builder(process_name: &str) -> ProcessConfigBuilder {
        ProcessConfigBuilder::new(process_name)
    }
}
