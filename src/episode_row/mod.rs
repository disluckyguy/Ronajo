mod imp;

use crate::episode_object::EpisodeObject;
use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;

glib::wrapper! {
    pub struct RonajoEpisodeRow(ObjectSubclass<imp::RonajoEpisodeRow>)
    @extends gtk::Box, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoEpisodeRow {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn bind(&self, object: &EpisodeObject) {

        let mut bindings = Vec::new();
        let name_binding = object.bind_property("number", &self.imp().episode_label.get(), "label")
            .transform_to(move |_, number: u32| {
                Some(format!("Episode {}", number))
            })
            .sync_create()
            .build();

        bindings.push(name_binding);

        self.imp().bindings.replace(bindings);


    }

    pub fn unbind(self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}
