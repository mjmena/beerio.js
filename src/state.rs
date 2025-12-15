use std::fs;
use std::sync::Arc;
use crate::model::MissionsData;

#[derive(Clone)]
pub struct AppState {
    pub missions: Arc<MissionsData>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("missions.json")?;
        let missions: MissionsData = serde_json::from_str(&content)?;
        Ok(Self {
            missions: Arc::new(missions),
        })
    }
}
