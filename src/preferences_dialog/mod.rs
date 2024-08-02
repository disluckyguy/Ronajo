mod imp;

use adw::{gio, glib};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use crate::device_object::DeviceObject;
use crate::core::config::*;

glib::wrapper! {
    pub struct RonajoPreferencesDialog(ObjectSubclass<imp::RonajoPreferencesDialog>)
    @extends adw::PreferencesDialog, adw::Dialog, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoPreferencesDialog {
    pub fn new() -> Self {
        Object::builder().build()
    }

    // pub fn setup_callbacks(&self) {

    // }
}
