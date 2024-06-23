
use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use crate::show_data::ShowData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::ShowObject)]
pub struct ShowObject {
    #[property(name = "name", get, set, type = String, member = name)]
    #[property(name = "description", get, set, type = String, member = description)]
    #[property(name = "image", get, set, type = String, member = image)]
    #[property(name = "airing", get, set, type = bool, member = airing)]
    #[property(name = "rating", get, set, type = f32, member = rating)]
    #[property(name = "in-library", get, set, type = bool, member = in_library)]
    pub data: RefCell<ShowData>,
}

#[glib::object_subclass]
impl ObjectSubclass for ShowObject {
    const NAME: &'static str = "RonajoShowObject";
    type Type = super::ShowObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for ShowObject {}
