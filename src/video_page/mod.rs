mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use std::cell;
use std::vec;

glib::wrapper! {
    pub struct RonajoVideoPage(ObjectSubclass<imp::RonajoVideoPage>)
    @extends adw::NavigationPage, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoVideoPage {
    pub fn new() -> Self {
        Object::builder()
            .build()
    }
}
