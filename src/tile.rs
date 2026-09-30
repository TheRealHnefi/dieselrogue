#[derive(PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    Ground,
    Road,
    Doorway,
    Fence,
    Window
}