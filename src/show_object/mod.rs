mod imp;

use crate::core::show_data::JikanData;
use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;

glib::wrapper! {
    pub struct ShowObject(ObjectSubclass<imp::ShowObject>);
}

impl ShowObject {
    pub fn new(data: JikanData) -> Self {
        let title = if data.title_english.is_some() {
            &data.title_english.clone().unwrap()
        } else {
            &data.title
        };

        let object: Self = Object::builder()
            .property("name", title)
            .property("image", &data.images.jpg.large_image_url)
            .build();

        object.imp().data.replace(Some(data));

        object
    }
}
