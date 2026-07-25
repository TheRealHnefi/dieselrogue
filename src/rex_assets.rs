use rltk::rex::XpFile;
use crate::PaperDoll;

rltk::embedded_resource!(MALE_SILHOUETTE, "../resources/male_silhouette.xp");
rltk::embedded_resource!(PLAYER_DOLL, "../resources/player.xp");
rltk::embedded_resource!(TANK_DOLL, "../resources/tank.xp");

pub struct RexAssets {
    pub male_silhouette: XpFile,
    pub player_doll: XpFile,
    pub tank_doll: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(MALE_SILHOUETTE, "male_silhouette.xp");
        rltk::link_resource!(PLAYER_DOLL, "player.xp");
        rltk::link_resource!(TANK_DOLL, "tank.xp");

        RexAssets {
            male_silhouette: XpFile::from_resource("male_silhouette.xp").unwrap(),
            player_doll:     XpFile::from_resource("player.xp").unwrap(),
            tank_doll:       XpFile::from_resource("tank.xp").unwrap(),
        }
    }

    pub fn get_doll(&self, doll: PaperDoll) -> &XpFile {
        match doll {
            PaperDoll::Player         => &self.player_doll,
            PaperDoll::MaleSilhouette => &self.male_silhouette,
            PaperDoll::Tank           => &self.tank_doll,
        }
    }
}