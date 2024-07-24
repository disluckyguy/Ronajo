use std::cell::RefCell;

use crate::core::show_data::JikanData;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::ShowObject)]
pub struct ShowObject {
    #[property(get, set)]
    pub name: RefCell<String>,
    #[property(get, set)]
    pub image: RefCell<String>,
    pub data: RefCell<Option<JikanData>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ShowObject {
    const NAME: &'static str = "RonajoShowObject";
    type Type = super::ShowObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for ShowObject {}
