use crate::rex::XpFile;

crate::embedded_resource!(TITLE_SCREEN, "../resources/title.xp");

pub struct RexAssets {
    pub title_screen: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        crate::link_resource!(TITLE_SCREEN, "title.xp");

        RexAssets {
            title_screen:    XpFile::from_resource("title.xp").unwrap(),
        }
    }
}