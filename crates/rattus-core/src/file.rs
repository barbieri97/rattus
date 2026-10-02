//! Saving and opening experiments (`.rattus` files, JSON).

use serde::{Deserialize, Serialize};

use crate::sim::Simulation;

pub const FILE_FORMAT: &str = "rattus";
pub const FILE_VERSION: u32 = 1;
pub const FILE_EXTENSION: &str = "rattus";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveFile {
    format: String,
    version: u32,
    app_version: String,
    simulation: Simulation,
}

#[derive(Deserialize)]
struct Header {
    format: Option<String>,
    version: Option<u32>,
}

pub fn save_to_string(sim: &Simulation, app_version: &str) -> Result<String, String> {
    let file = SaveFile {
        format: FILE_FORMAT.to_string(),
        version: FILE_VERSION,
        app_version: app_version.to_string(),
        simulation: sim.clone(),
    };
    serde_json::to_string(&file).map_err(|e| format!("Could not save the experiment: {e}"))
}

pub fn load_from_str(text: &str) -> Result<Simulation, String> {
    let header: Header = serde_json::from_str(text)
        .map_err(|_| "This is not a Rattus experiment file.".to_string())?;
    if header.format.as_deref() != Some(FILE_FORMAT) {
        return Err("This is not a Rattus experiment file.".into());
    }
    match header.version {
        Some(v) if v <= FILE_VERSION => {}
        _ => {
            return Err(
                "This file was saved by a newer version of Rattus. Please update the app.".into(),
            );
        }
    }
    let file: SaveFile =
        serde_json::from_str(text).map_err(|e| format!("The experiment file is damaged: {e}"))?;
    let mut sim = file.simulation;
    sim.sanitize();
    Ok(sim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Command, RatTemplate};

    #[test]
    fn round_trip_continues_identically() {
        let mut a = Simulation::new(RatTemplate::BarTrained, 11);
        a.advance(120.0);
        let text = save_to_string(&a, "test").unwrap();
        let mut b = load_from_str(&text).unwrap();
        a.apply(Command::GivePellet).unwrap();
        b.apply(Command::GivePellet).unwrap();
        a.advance(300.0);
        b.advance(300.0);
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.recorder().events, b.recorder().events);
    }

    #[test]
    fn rejects_other_files() {
        assert!(load_from_str("{}").is_err());
        assert!(load_from_str("not json").is_err());
        assert!(load_from_str(r#"{"format":"rattus","version":99}"#).is_err());
    }
}
