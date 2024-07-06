use chrono::{DateTime, Utc};
use log::debug;
use serde::{Serialize, Deserialize};

/// Log of damage 
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageLog {
    timestamp: DateTime::<Utc>,
    damage: isize,
    other_player: String,
    other_ship: String,
    weapon: String,
    destination: Destination,
}

impl DamageLog {
    /// Builds a new DamageLog
    pub fn new(timestamp: DateTime::<Utc>, damage: isize, other_player: String, other_ship: String, weapon: String, destination: Destination) -> Self {
        debug!("Creating a new damagelog ({},{},{},{},{},{:?})", timestamp, damage, other_player, other_ship, weapon, destination);
        DamageLog { timestamp, damage, other_player, other_ship, weapon, destination }
    } 
}


/// Log of logi reps
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogiLog {
    timestamp: DateTime::<Utc>,
    amount: isize,
    other_player: String,
    other_ship: String,
    rep_type: String,
    destination: Destination,
}

impl LogiLog {
    /// Builds a new [LogiLog]
    pub fn new(timestamp: DateTime::<Utc>, amount: isize, other_player: String, other_ship: String, rep_type: String, destination: Destination) -> Self {
        debug!("Creating a new damagelog ({},{},{},{},{},{:?})", timestamp, amount, other_player, other_ship, rep_type, destination);
        LogiLog { timestamp, amount, other_player, other_ship, rep_type, destination }
    }
}

/// Represents if the log is created by the player or not
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    /// The log is received by the player
    /// (being shot at, being repped, ...)
    Receiving,
    /// The log is initiated by the player
    /// (shooting someone, repping someone, ...)
    Dealing,
}

/// General struct embedding any kind of log
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Log {
    Damage(DamageLog), 
    Logi(LogiLog),
}

