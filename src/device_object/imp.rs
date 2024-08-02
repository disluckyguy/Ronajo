use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use crate::core::player_data::PlayerData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::DeviceObject)]
pub struct DeviceObject {
    #[property(get, set, name = "device-name", type = String, member = name)]
    #[property(get, set, name = "address", type = String, member = address)]
    #[property(get, set, name = "username", type = String, member = username)]
    #[property(get, set, name = "password", type = String, member = password, nullable)]
    #[property(get, set, name = "use-key", type = bool, member = use_key)]
    pub data: RefCell<PlayerData>,
}

#[glib::object_subclass]
impl ObjectSubclass for DeviceObject {
    const NAME: &'static str = "RonajoDeviceObject";
    type Type = super::DeviceObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for DeviceObject {}
