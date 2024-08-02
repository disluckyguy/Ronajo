mod imp;

use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;

glib::wrapper! {
    pub struct PasswordDialog(ObjectSubclass<imp::PasswordDialog>)
    @extends adw::Dialog, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PasswordDialog {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
