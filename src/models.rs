use chrono::{DateTime, Utc};

/// Log of damage either inflicted to a player or dealt by another player
#[derive(Debug, PartialEq, Eq)]
pub struct DamageLog {
    timestamp: DateTime::<Utc>,
    damage: isize,
    other_player: String,
    other_ship: String,
    weapon: String,
    destination: Destination,
}

impl DamageLog {
    pub fn new(timestamp: DateTime::<Utc>, damage: isize, other_player: String, other_ship: String, weapon: String, destination: Destination) -> Self {
        DamageLog { timestamp, damage, other_player, other_ship, weapon, destination }
    } 
}


/// Log of logi reps
#[derive(Debug, PartialEq, Eq)]
pub struct LogiLog {
    timestamp: DateTime::<Utc>,
    amount: isize,
    other_player: String,
    other_ship: String,
    rep_type: String,
    destination: Destination,
}

impl LogiLog {
    pub fn new(timestamp: DateTime::<Utc>, amount: isize, other_player: String, other_ship: String, rep_type: String, destination: Destination) -> Self {
        LogiLog { timestamp, amount, other_player, other_ship, rep_type, destination }
    }
}

/// Represents if the log is received from another player (being shot at, being repped, ...) or
/// if you're its source (you're shooting, you're repping, ...)
#[derive(Debug, PartialEq, Eq)]
pub enum Destination {
    Receiving,
    Dealing,
}

/// General struct embedding any kind of log
#[derive(Debug, PartialEq, Eq)]
pub enum Log {
    Damage(DamageLog), 
    Logi(LogiLog),
}
