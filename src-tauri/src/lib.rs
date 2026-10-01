//! Desktop shell for Rattus: owns the simulation, runs it on a background thread and talks
//! to the Vue interface through commands and a streaming channel.

mod commands;
mod runner;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rattus_core::Simulation;
use rattus_core::protocol::{HostStatus, RatTemplate, SimMessage};
use tauri::ipc::Channel;

/// The simulation and the state of the runner that drives it.
pub struct Host {
    pub sim: Simulation,
    pub status: HostStatus,
    pub channel: Option<Channel<SimMessage>>,
}

impl Host {
    /// Sends the whole state to the interface (after opening a file, a new rat, ...).
    pub fn send_reset(&mut self) {
        let full = self.sim.full_state(self.status.clone());
        if let Some(channel) = &self.channel
            && channel.send(SimMessage::Reset(Box::new(full))).is_err()
        {
            self.channel = None;
        }
    }
}

#[derive(Clone)]
pub struct SharedHost(Arc<Mutex<Host>>);

impl SharedHost {
    pub fn lock(&self) -> MutexGuard<'_, Host> {
        // A panic while holding the lock leaves the simulation usable; keep going.
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x5EED)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared = SharedHost(Arc::new(Mutex::new(Host {
        sim: Simulation::new(RatTemplate::Naive, random_seed()),
        status: HostStatus::default(),
        channel: None,
    })));
    runner::spawn(shared.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(shared)
        .invoke_handler(tauri::generate_handler![
            commands::subscribe,
            commands::sim_command,
            commands::host_control,
            commands::save_experiment,
            commands::open_experiment,
            commands::export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rattus");
}
