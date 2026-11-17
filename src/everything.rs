// The search backend. `EverythingBackend` is the trait the UI talks to, so the
// UI never knows whether it's getting real results from voidtools' Everything
// or fake ones for layout testing.
//
// `MockBackend` is what we compile with on any OS (including this Linux
// sandbox) so we can actually run and look at the UI. `EverythingBackend`'s
// real implementation (IPC/FFI into Everything64.dll) is stubbed under
// `#[cfg(windows)]` below -- it compiles but every call currently returns
// "not wired up yet". That's the next step once the layout looks right.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    Executables,
    Folders,
    Documents,
    Images,
}

impl FilterMode {
    pub const ALL: [FilterMode; 5] = [
        FilterMode::All,
        FilterMode::Executables,
        FilterMode::Folders,
        FilterMode::Documents,
        FilterMode::Images,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            FilterMode::All => "All",
            FilterMode::Executables => "Apps",
            FilterMode::Folders => "Folders",
            FilterMode::Documents => "Docs",
            FilterMode::Images => "Images",
        }
    }

    // The actual Everything search-syntax fragment this mode prepends to the
    // user's query, e.g. "ext:exe foo" -- Everything already understands this,
    // we don't reimplement filtering, just build the right query string.
    pub fn query_prefix(&self) -> &'static str {
        match self {
            FilterMode::All => "",
            FilterMode::Executables => "ext:exe;bat;cmd;lnk ",
            FilterMode::Folders => "folder: ",
            FilterMode::Documents => "ext:pdf;docx;doc;xlsx;pptx;txt;md ",
            FilterMode::Images => "ext:png;jpg;jpeg;gif;webp;svg ",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub full_path: String,
    pub is_dir: bool,
}

pub trait EverythingBackend {
    /// Run a query (already including the filter prefix) and return results,
    /// most-relevant first. Implementations should be fast enough to call on
    /// every keystroke.
    fn search(&mut self, query: &str, max_results: usize) -> Vec<SearchResult>;

    /// Human-readable backend status shown in the UI's footer, e.g.
    /// "Connected to Everything" or "Everything not running".
    fn status(&self) -> String;
}

// ---------------------------------------------------------------------
// Mock backend -- fixed in-memory dataset, filtered by substring + the
// query_prefix's extension list. Good enough to test scrolling, highlighting,
// keyboard nav, and the filter tabs without touching Windows at all.
// ---------------------------------------------------------------------
pub struct MockBackend {
    data: Vec<SearchResult>,
}

impl Default for MockBackend {
    fn default() -> Self {
        let sample = [
            ("chrome.exe", r"C:\Program Files\Google\Chrome\Application\chrome.exe", false),
            ("notepad.exe", r"C:\Windows\System32\notepad.exe", false),
            ("Everything.exe", r"C:\Program Files\Everything\Everything.exe", false),
            ("Projects", r"C:\Users\you\Projects", true),
            ("Downloads", r"C:\Users\you\Downloads", true),
            ("resume.docx", r"C:\Users\you\Documents\resume.docx", false),
            ("budget.xlsx", r"C:\Users\you\Documents\budget.xlsx", false),
            ("vacation.jpg", r"C:\Users\you\Pictures\vacation.jpg", false),
            ("logo.png", r"C:\Users\you\Pictures\logo.png", false),
            ("steam.exe", r"C:\Program Files (x86)\Steam\steam.exe", false),
            ("report.pdf", r"C:\Users\you\Documents\report.pdf", false),
            ("Games", r"C:\Users\you\Games", true),
        ];
        Self {
            data: sample
                .into_iter()
                .map(|(name, path, is_dir)| SearchResult {
                    name: name.to_string(),
                    full_path: path.to_string(),
                    is_dir,
                })
                .collect(),
        }
    }
}

impl EverythingBackend for MockBackend {
    fn search(&mut self, query: &str, max_results: usize) -> Vec<SearchResult> {
        // Very rough stand-in for Everything's own filtering: pull any
        // "ext:a;b;c " prefix off the front, match the rest as a case-
        // insensitive substring of the name.
        let (ext_filter, text) = match query.split_once(' ') {
            Some((prefix, rest)) if prefix.starts_with("ext:") || prefix.starts_with("folder:") => {
                (Some(prefix), rest)
            }
            _ => (None, query),
        };
        let text = text.trim().to_lowercase();

        self.data
            .iter()
            .filter(|r| {
                let name_ok = text.is_empty() || r.name.to_lowercase().contains(&text);
                let ext_ok = match ext_filter {
                    None => true,
                    Some(f) if f.starts_with("folder:") => r.is_dir,
                    Some(f) => {
                        let exts: Vec<&str> = f.trim_start_matches("ext:").split(';').collect();
                        exts.iter().any(|e| r.name.to_lowercase().ends_with(&format!(".{e}")))
                    }
                };
                name_ok && ext_ok
            })
            .take(max_results)
            .cloned()
            .collect()
    }

    fn status(&self) -> String {
        "Mock backend (no Everything connection -- layout-testing mode)".to_string()
    }
}

// ---------------------------------------------------------------------
// Real backend -- talks to Everything via its documented IPC window
// (see voidtools' SDK: Everything_SetSearchW / Everything_QueryW via
// Everything64.dll, or the WM_COPYDATA IPC protocol if we skip the DLL).
// Left unimplemented on purpose: this is the next milestone, once the UI
// skeleton is approved. Only compiled on Windows.
// ---------------------------------------------------------------------
#[cfg(windows)]
pub struct EverythingIpcBackend {
    connected: bool,
}

#[cfg(windows)]
impl Default for EverythingIpcBackend {
    fn default() -> Self {
        // TODO: locate the "EVERYTHING_TASKBAR_NOTIFICATION" window via
        // FindWindowW, or load Everything64.dll with LoadLibraryW and
        // resolve Everything_SetSearchW / Everything_QueryW with GetProcAddress.
        Self { connected: false }
    }
}

#[cfg(windows)]
impl EverythingBackend for EverythingIpcBackend {
    fn search(&mut self, _query: &str, _max_results: usize) -> Vec<SearchResult> {
        // TODO: wire up Everything_SetSearchW + Everything_QueryW(TRUE),
        // then walk results with Everything_GetResultFullPathNameW.
        Vec::new()
    }

    fn status(&self) -> String {
        if self.connected {
            "Connected to Everything".to_string()
        } else {
            "Everything not running or IPC not wired up yet".to_string()
        }
    }
}
