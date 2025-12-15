use std::fs;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::model::MissionsData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum LobbyStatus {
    Waiting,
    Started,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub name: String,
    pub is_traitor: bool,
}

#[derive(Clone, Debug)]
pub struct Lobby {
    pub id: String,
    pub players: Vec<Player>,
    pub status: LobbyStatus,
    pub seed: String,
}

#[derive(Clone)]
pub struct AppState {
    pub missions: Arc<MissionsData>,
    pub lobbies: Arc<RwLock<HashMap<String, Lobby>>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("missions.json")?;
        let missions: MissionsData = serde_json::from_str(&content)?;
        Ok(Self {
            missions: Arc::new(missions),
            lobbies: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}
