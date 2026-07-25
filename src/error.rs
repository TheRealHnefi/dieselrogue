#[derive(Debug)]
pub enum Error {
    Generic,
    BadPrecondition,
    UnsolvableSituation
}

#[derive(Debug)]
pub struct GameError {
    pub error: Error,
    pub message: String
}

impl From<()> for GameError {
    fn from(_err: ()) -> Self {
        GameError {error: Error::Generic, message: String::from("")}
    }
}
