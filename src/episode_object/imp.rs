use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};

#[derive(Properties, Default)]
#[properties(wrapper_type = super::EpisodeObject)]
pub struct EpisodeObject {
    #[property(get, set)]
    pub number: Cell<u32>,
    #[property(get, set, nullable)]
    pub allanime_id: RefCell<Option<String>>,
    #[property(get, set)]
    pub translation: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for EpisodeObject {
    const NAME: &'static str = "RonajoEpisodeObject";
    type Type = super::EpisodeObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for EpisodeObject {}
