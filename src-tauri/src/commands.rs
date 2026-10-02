//! Commands invoked by the interface.

use rattus_core::file::{load_from_str, save_to_string};
use rattus_core::protocol::{Command, HostControl, HostStatus, SimMessage};
use rattus_core::recorder::ExportDataset;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};

use crate::SharedHost;

/// Starts streaming frames to the interface; the first message is the full state.
#[tauri::command]
pub fn subscribe(host: State<'_, SharedHost>, on_message: Channel<SimMessage>) {
    let mut host = host.lock();
    host.channel = Some(on_message);
    host.send_reset();
}

#[tauri::command]
pub fn sim_command(host: State<'_, SharedHost>, command: Command) -> Result<(), String> {
    let mut host = host.lock();
    let resets = matches!(command, Command::NewRat { .. });
    host.sim.apply(command)?;
    if resets {
        host.send_reset();
    }
    Ok(())
}

#[tauri::command]
pub fn host_control(
    host: State<'_, SharedHost>,
    control: HostControl,
) -> Result<HostStatus, String> {
    let mut host = host.lock();
    let status = &mut host.status;
    match control {
        HostControl::Pause => status.paused = true,
        HostControl::Resume => status.paused = false,
        HostControl::SetSpeed { speed } => {
            if !speed.is_finite() || !(0.1..=16.0).contains(&speed) {
                return Err("Speed must be between 0.1 and 16.".into());
            }
            status.speed = speed;
        }
        HostControl::SetIsolated { isolated } => status.isolated = isolated,
        HostControl::SetIsolatedSpeed { speed } => {
            if !speed.is_finite() || !(10.0..=10_000.0).contains(&speed) {
                return Err("Accelerated speed must be between 10 and 10000.".into());
            }
            status.isolated_speed = speed;
        }
    }
    Ok(host.status.clone())
}

#[tauri::command]
pub fn save_experiment(
    app: AppHandle,
    host: State<'_, SharedHost>,
    path: String,
) -> Result<(), String> {
    let text = {
        let host = host.lock();
        save_to_string(&host.sim, &app.package_info().version.to_string())?
    };
    std::fs::write(&path, text).map_err(|e| format!("Could not write {path}: {e}"))
}

#[tauri::command]
pub fn open_experiment(
    host: State<'_, SharedHost>,
    path: String,
    random_start: bool,
) -> Result<(), String> {
    let text = std::fs::read_to_string(&path).map_err(|e| format!("Could not read {path}: {e}"))?;
    let mut sim = load_from_str(&text)?;
    if random_start {
        sim.place_near_center();
    }
    let mut host = host.lock();
    host.sim = sim;
    host.send_reset();
    Ok(())
}

#[tauri::command]
pub fn export_csv(
    host: State<'_, SharedHost>,
    path: String,
    dataset: ExportDataset,
) -> Result<(), String> {
    let csv = host.lock().sim.export_csv(dataset);
    std::fs::write(&path, csv).map_err(|e| format!("Could not write {path}: {e}"))
}
