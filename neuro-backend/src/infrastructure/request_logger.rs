//! =============================================================================
//! Request Logger - Visual terminal logging for API requests
//! =============================================================================
//! Provides pretty console output with spinners and timing for requests.
//! =============================================================================

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

// ANSI color codes
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

/// Spinner characters for animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Request tracker for visual logging
pub struct RequestLogger {
    active: AtomicBool,
    start_time: Mutex<Option<Instant>>,
    request_type: Mutex<String>,
    request_id: AtomicU64,
}

impl RequestLogger {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            start_time: Mutex::new(None),
            request_type: Mutex::new(String::new()),
            request_id: AtomicU64::new(0),
        }
    }

    /// Start tracking a new request
    pub async fn start_request(&self, request_type: &str, info: &str) {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst) + 1;
        *self.start_time.lock().await = Some(Instant::now());
        *self.request_type.lock().await = request_type.to_string();
        self.active.store(true, Ordering::SeqCst);

        println!(
            "\n{CYAN}┌─{RESET} {BOLD}#{id}{RESET} {MAGENTA}{request_type}{RESET} {DIM}{info}{RESET}"
        );
        io::stdout().flush().ok();
    }

    /// Update the spinner with current elapsed time
    pub async fn update_spinner(&self, message: &str) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        let elapsed = if let Some(start) = *self.start_time.lock().await {
            start.elapsed().as_secs_f64()
        } else {
            0.0
        };

        let frame_idx = (elapsed * 10.0) as usize % SPINNER_FRAMES.len();
        let spinner = SPINNER_FRAMES[frame_idx];

        print!(
            "\r{CYAN}│{RESET} {YELLOW}{spinner}{RESET} {message} {DIM}({elapsed:.1}s){RESET}    "
        );
        io::stdout().flush().ok();
    }

    /// Complete the request with success
    pub async fn complete_success(&self, tokens_in: u32, tokens_out: u32, model: &str) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        let elapsed = if let Some(start) = *self.start_time.lock().await {
            start.elapsed()
        } else {
            std::time::Duration::ZERO
        };

        let elapsed_secs = elapsed.as_secs_f64();
        let tok_per_sec = if elapsed_secs > 0.0 {
            tokens_out as f64 / elapsed_secs
        } else {
            0.0
        };

        // Clear spinner line
        print!("\r{CYAN}│{RESET}                                                              \r");
        
        // Print completion info
        println!(
            "{CYAN}│{RESET} {GREEN}✓{RESET} Completed in {YELLOW}{:.2}s{RESET}",
            elapsed_secs
        );
        println!(
            "{CYAN}│{RESET}   {DIM}Model:{RESET} {CYAN}{model}{RESET}  {DIM}Tokens:{RESET} {tokens_in} → {GREEN}{tokens_out}{RESET}  {DIM}Speed:{RESET} {MAGENTA}{tok_per_sec:.1}{RESET} tok/s"
        );
        println!("{CYAN}└─{RESET}");
        io::stdout().flush().ok();

        self.active.store(false, Ordering::SeqCst);
    }

    /// Complete the request with streaming info
    pub async fn complete_stream(&self, chunks: u32, total_tokens: u32, model: &str) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        let elapsed = if let Some(start) = *self.start_time.lock().await {
            start.elapsed()
        } else {
            std::time::Duration::ZERO
        };

        let elapsed_secs = elapsed.as_secs_f64();
        let tok_per_sec = if elapsed_secs > 0.0 {
            total_tokens as f64 / elapsed_secs
        } else {
            0.0
        };

        // Clear spinner line
        print!("\r{CYAN}│{RESET}                                                              \r");
        
        // Print completion info
        println!(
            "{CYAN}│{RESET} {GREEN}✓{RESET} Stream completed in {YELLOW}{:.2}s{RESET}",
            elapsed_secs
        );
        println!(
            "{CYAN}│{RESET}   {DIM}Model:{RESET} {CYAN}{model}{RESET}  {DIM}Chunks:{RESET} {chunks}  {DIM}Tokens:{RESET} {GREEN}{total_tokens}{RESET}  {DIM}Speed:{RESET} {MAGENTA}{tok_per_sec:.1}{RESET} tok/s"
        );
        println!("{CYAN}└─{RESET}");
        io::stdout().flush().ok();

        self.active.store(false, Ordering::SeqCst);
    }

    /// Complete the request with error
    pub async fn complete_error(&self, error: &str) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        let elapsed = if let Some(start) = *self.start_time.lock().await {
            start.elapsed()
        } else {
            std::time::Duration::ZERO
        };

        // Clear spinner line
        print!("\r{CYAN}│{RESET}                                                              \r");
        
        println!(
            "{CYAN}│{RESET} {RED}✗{RESET} Failed after {YELLOW}{:.2}s{RESET}",
            elapsed.as_secs_f64()
        );
        println!("{CYAN}│{RESET}   {RED}{error}{RESET}");
        println!("{CYAN}└─{RESET}");
        io::stdout().flush().ok();

        self.active.store(false, Ordering::SeqCst);
    }

    /// Check if a request is currently active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global request logger instance
use once_cell::sync::Lazy;
pub static REQUEST_LOGGER: Lazy<Arc<RequestLogger>> = Lazy::new(|| Arc::new(RequestLogger::new()));

/// Spawn a background task to animate the spinner
pub fn spawn_spinner_task(message: String) -> tokio::task::JoinHandle<()> {
    let logger = REQUEST_LOGGER.clone();
    tokio::spawn(async move {
        while logger.is_active() {
            logger.update_spinner(&message).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
        }
    })
}
