pub struct GameLog {
    pub entries: Vec<String>
}

impl GameLog {
    pub fn log(&mut self, message: String) {
        self.entries.push(message);
    }
}