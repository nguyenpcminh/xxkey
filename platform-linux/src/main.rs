use platform_linux::daemon::LinuxDaemon;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("XXKey background daemon starting on Linux...");
    let daemon = LinuxDaemon::new();
    println!(
        "Daemon initialized. Engine enabled: {}, Input type: {:?}",
        daemon.config_mgr.current.enabled, daemon.config_mgr.current.input_type
    );
    println!("XXKey Linux daemon is running in background. Press Ctrl+C to terminate.");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Register SIGINT/SIGTERM handlers
    let _ = ctrlc::set_handler(move || {
        println!("\nReceived shutdown signal. Stopping XXKey Linux daemon...");
        r.store(false, Ordering::SeqCst);
    });

    // Daemon background execution loop keeping process active
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    println!("XXKey Linux daemon shut down cleanly.");
}
