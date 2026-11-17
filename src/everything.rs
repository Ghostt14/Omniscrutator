// The search backend. `EverythingBackend` is the trait the UI talks to, so the
// UI never knows whether it's getting real results from voidtools' Everything
// or fake ones for layout testing.
//
// `MockBackend` is what we compile with on any OS (including the Linux
// sandbox this skeleton was first built in) so it can actually run and be
// looked at. The real implementation (IPC/FFI into Everything64.dll) is
// stubbed under `#[cfg(windows)]` below -- it compiles but returns nothing
// yet. That's the next milestone.

use serde::{Deserialize, Serialize};

// Search is always unfiltered ("All") now -- per-type tabs were removed from
// the main UI. `ResultPriority` instead lives in Settings and just re-orders
// results: matches of the chosen category are stably sorted to the front,
// everything else keeps its normal relevance order behind them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultPriority {
    General,
    Executables,
    Folders,
    Documents,
    Images,
}

impl ResultPriority {
    pub const ALL: [ResultPriority; 5] = [
        ResultPriority::General,
        ResultPriority::Executables,
        ResultPriority::Folders,
        ResultPriority::Documents,
        ResultPriority::Images,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ResultPriority::General => "General (no re-ordering)",
            ResultPriority::Executables => "Apps first",
            ResultPriority::Folders => "Folders first",
            ResultPriority::Documents => "Documents first",
            ResultPriority::Images => "Images first",
        }
    }

    /// Does this result belong to the category this priority boosts?
    /// `General` boosts nothing -- natural relevance order is kept.
    pub fn matches(&self, r: &SearchResult) -> bool {
        let lower = r.name.to_lowercase();
        let ends_with_any = |exts: &[&str]| exts.iter().any(|e| lower.ends_with(&format!(".{e}")));
        match self {
            ResultPriority::General => false,
            ResultPriority::Executables => ends_with_any(&["exe", "bat", "cmd", "lnk"]),
            ResultPriority::Folders => r.is_dir,
            ResultPriority::Documents => ends_with_any(&["pdf", "docx", "doc", "xlsx", "pptx", "txt", "md"]),
            ResultPriority::Images => ends_with_any(&["png", "jpg", "jpeg", "gif", "webp", "svg"]),
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
    /// Run an unfiltered query and return results, most-relevant first.
    /// Re-ordering by `ResultPriority` happens afterward, in the UI layer,
    /// not here -- the backend's job is just "find matches of this text".
    fn search(&mut self, query: &str, max_results: usize) -> Vec<SearchResult>;

    /// Human-readable backend status shown in the UI's footer, e.g.
    /// "Connected to Everything" or "Everything not running".
    fn status(&self) -> String;
}

// ---------------------------------------------------------------------
// Mock backend -- fixed in-memory dataset, substring-filtered. Good enough
// to test scrolling, highlighting, keyboard nav, sorting and the math/
// settings command handling without touching Windows at all.
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
        let text = query.trim().to_lowercase();
        self.data
            .iter()
            .filter(|r| text.is_empty() || r.name.to_lowercase().contains(&text))
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
// (voidtools' SDK: Everything_SetSearchW / Everything_QueryW via
// Everything64.dll, or the WM_COPYDATA IPC protocol if we skip the DLL).
// Left unimplemented on purpose: next milestone, once the UI is approved.
// Only compiled on Windows.
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
