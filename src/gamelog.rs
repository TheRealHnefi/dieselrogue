/// How a log line is tinted in the UI.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum LogKind { Info, Enemy }

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub text: String,
    pub kind: LogKind,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameLog {
    pub entries: Vec<LogEntry>
}

impl GameLog {
    pub fn log(&mut self, message: String) {
        self.entries.push(LogEntry { text: message, kind: LogKind::Info });
    }

    /// An enemy's deed — tinted red in the log panel.
    pub fn log_enemy(&mut self, message: String) {
        self.entries.push(LogEntry { text: message, kind: LogKind::Enemy });
    }

    /// A deed by the player (plain) or by anyone else (enemy tint).
    pub fn log_deed(&mut self, by_player: bool, message: String) {
        if by_player { self.log(message) } else { self.log_enemy(message) }
    }
}
