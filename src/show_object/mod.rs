mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use crate::show_data::ShowData;

glib::wrapper! {
    pub struct ShowObject(ObjectSubclass<imp::ShowObject>);
}

impl ShowObject {
    pub fn new(data: ShowData) -> Self {
        Object::builder()
            .property("name", data.name)
            .property("image", data.image)
            .property("description", data.description)
            .property("rating", data.rating)
            .property("in-library", data.in_library)
            .property("airing", data.airing)
            .build()
    }
}
