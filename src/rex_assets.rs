use rltk::rex::XpFile;

rltk::embedded_resource!(MALE_SILHOUETTE, "../resources/male_silhouette.xp");

pub struct RexAssets {
    pub male_silhouette: XpFile
}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(MALE_SILHOUETTE, "male_silhouette.xp");

        RexAssets {
            male_silhouette: XpFile::from_resource("male_silhouette.xp").unwrap()
        }
    }
}