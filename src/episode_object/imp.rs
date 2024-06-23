use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use crate::show_data::EpisodeData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::EpisodeObject)]
pub struct EpisodeObject {
    #[property(name = "stream-url", get, set, type = String, member = stream_url)]
    #[property(name = "number", get, set, type = u32, member = number)]
    pub data: RefCell<EpisodeData>,
}

#[glib::object_subclass]
impl ObjectSubclass for EpisodeObject {
    const NAME: &'static str = "RonajoEpisodeObject";
    type Type = super::EpisodeObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for EpisodeObject {}
