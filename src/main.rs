// main.rs — mactemp: Lightweight macOS menu bar system monitor
//
// Entry point for the application. Sets up the status bar item,
// spawns a background ticker thread, and runs the main event loop.
//
// Architecture:
//   - Single-instance enforcement via exclusive file lock (flock)
//   - Background thread sends ticks every 2 seconds via mpsc channel
//   - Main thread (event loop) receives ticks, reads sensors, updates UI
//   - SMC reader lives on the main thread (macsmc::Smc is !Send + !Sync)
//   - No main window — the app is entirely in the menu bar

mod autostart;
mod menu;
mod smc;
mod system_stats;

use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use system_status_bar_macos::{sync_infinite_event_loop, Menu, StatusItem};

use smc::SmcReader;
use system_stats::StatsReader;

/// Refresh interval in seconds.
const REFRESH_INTERVAL_SECS: u64 = 2;

/// Path to the lock file used for single-instance enforcement.
const LOCK_FILE_PATH: &str = "/tmp/mactemp.lock";

/// Attempts to acquire an exclusive lock on the lock file.
/// Returns the File handle (which must be kept alive to hold the lock)
/// or None if another instance is already running.
fn acquire_instance_lock() -> Option<File> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(LOCK_FILE_PATH)
        .ok()?;

    // Try to acquire an exclusive, non-blocking lock (LOCK_EX | LOCK_NB).
    // If another instance holds the lock, flock returns -1 immediately
    // instead of blocking, so we know to exit.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };

    if result == 0 {
        Some(file) // Lock acquired — we are the only instance
    } else {
        None // Lock held by another instance
    }
}

fn main() {
    // Single-instance enforcement: acquire exclusive file lock.
    // The _lock_file must remain alive for the entire process lifetime —
    // when the process exits (normally or crashes), the OS releases the lock.
    let _lock_file = match acquire_instance_lock() {
        Some(file) => file,
        None => {
            eprintln!("[mactemp] Another instance is already running. Exiting.");
            std::process::exit(0);
        }
    };

    eprintln!("[mactemp] Starting macOS system monitor...");

    // Channel for tick events from background thread → main thread
    let (tick_sender, tick_receiver) = mpsc::channel::<()>();

    // Clone sender for the Refresh menu button
    let refresh_sender = tick_sender.clone();

    // Spawn background ticker thread — sends a tick every REFRESH_INTERVAL_SECS
    thread::spawn(move || {
        loop {
            if tick_sender.send(()).is_err() {
                // Receiver dropped — main thread exited
                break;
            }
            thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECS));
        }
    });

    // Create status bar item with initial loading text
    let status_item = RefCell::new(StatusItem::new("⏳ mactemp", Menu::new(vec![])));

    // Initialize readers on the main thread
    // SmcReader may fail if SMC is unavailable (e.g., in a VM)
    let smc_reader: RefCell<Option<SmcReader>> = RefCell::new(SmcReader::new());
    let stats_reader = RefCell::new(StatsReader::new());

    eprintln!("[mactemp] Entering event loop (refresh every {}s)", REFRESH_INTERVAL_SECS);

    // Main event loop — processes ticks on the main thread
    // This blocks forever (until the app quits via the Quit menu item)
    sync_infinite_event_loop(tick_receiver, move |_| {
        // Read temperatures from SMC (gracefully handles unavailability)
        let temps = {
            let mut smc = smc_reader.borrow_mut();
            match smc.as_mut() {
                Some(reader) => reader.read_temperatures(),
                None => smc::TemperatureData::default(),
            }
        };

        // Read CPU and RAM stats
        let stats = stats_reader.borrow_mut().read_stats();

        // Update the menu bar title (compact format)
        let title = menu::build_title(&temps, &stats);
        status_item.borrow_mut().set_title(&title);

        // Update the dropdown menu (detailed stats + action items)
        let dropdown = menu::build_menu(&temps, &stats, refresh_sender.clone());
        status_item.borrow_mut().set_menu(dropdown);
    });
}
