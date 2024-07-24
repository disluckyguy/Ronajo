mod imp;

use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;

glib::wrapper! {
    pub struct EpisodeObject(ObjectSubclass<imp::EpisodeObject>);
}

impl EpisodeObject {
    pub fn new(number: u32, allanime_id: Option<String>, translation: &str) -> Self {
        Object::builder()
            .property("number", number)
            .property("allanime-id", allanime_id)
            .property("translation", translation)
            .build()
    }
}
