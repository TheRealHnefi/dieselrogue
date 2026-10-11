use rltk::rex::XpFile;

rltk::embedded_resource!(TITLE_SCREEN, "../resources/title.xp");

pub struct RexAssets {
    pub title_screen: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(TITLE_SCREEN, "title.xp");

        RexAssets {
            title_screen:    XpFile::from_resource("title.xp").unwrap(),
        }
    }
}