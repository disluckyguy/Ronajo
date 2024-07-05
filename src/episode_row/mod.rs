mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use crate::episode_object::EpisodeObject;
use std::vec;

glib::wrapper! {
    pub struct RonajoEpisodeRow(ObjectSubclass<imp::RonajoEpisodeRow>)
    @extends gtk::Button, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoEpisodeRow {
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    pub fn bind(&self, _object: &EpisodeObject) {


    }

    pub fn unbind(self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

