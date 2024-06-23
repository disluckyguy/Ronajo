mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use crate::show_data::EpisodeData;

glib::wrapper! {
    pub struct EpisodeObject(ObjectSubclass<imp::EpisodeObject>);
}

impl EpisodeObject {
    pub fn new(data: EpisodeData) -> Self {
        Object::builder()
            .property("number", data.number)
            .property("stream-url", data.stream_url)
            .build()
    }
}
