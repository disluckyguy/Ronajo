mod imp;

use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use crate::core::player_data::PlayerData;
use glib::Object;

glib::wrapper! {
    pub struct DeviceObject(ObjectSubclass<imp::DeviceObject>);
}

impl DeviceObject {
    pub fn new(device_name: String, address: String, username: String, password: Option<String>, use_key: bool) -> Self {
        Object::builder()
            .property("device-name", device_name)
            .property("address", address)
            .property("username", username)
            .property("password", password)
            .property("use-key", use_key)
            .build()
    }

    pub fn from_player_data(data: &PlayerData) -> Self {
        Object::builder()
            .property("device-name", &data.name)
            .property("address", &data.address)
            .property("username", &data.username)
            .property("password", &data.password)
            .property("use-key", data.use_key)
            .build()
    }

    pub fn player_data(&self) -> PlayerData {
        self
            .imp()
            .data
            .borrow()
            .clone()
    }
}
