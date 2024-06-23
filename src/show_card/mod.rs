mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use crate::show_object::ShowObject;
use std::vec;

glib::wrapper! {
    pub struct RonajoShowCard(ObjectSubclass<imp::RonajoShowCard>)
    @extends gtk::Button, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoShowCard {
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    pub fn bind(&self, object: &ShowObject) {
        let title_label = self.imp().title_label.get();
        let description_label = self.imp().description_label.get();
        let image = self.imp().image.get();
        let mut bindings = self.imp().bindings.borrow_mut();
        let object_data = object.imp().data.borrow().clone();

        self.imp().data.replace(object_data);

        let title_binding = object.bind_property("name", &title_label, "label")
            .sync_create()
            .build();

        bindings.push(title_binding);

        let description_binding = object.bind_property("description", &description_label, "label")
            .sync_create()
            .build();

        bindings.push(description_binding);

        let image_binding = object.bind_property("image", &image, "file")
            .sync_create()
            .build();



        bindings.push(image_binding);

    }

    pub fn unbind(self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}


