#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
//! Opens the Termy XL window (the standalone Trends Desk). It does not publish.
//! The Termy X app and its shortcut are a different program.
//!
//! ```sh
//! cargo run -p termy --bin trends_desk
//! ```
//!
//! A live run uses `opencli` on PATH. The packaged CLI name is `termy-xcli`,
//! never `x.exe`.

fn main() {
    termy::trends_desk::run_standalone();
}
