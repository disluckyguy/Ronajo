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

    fn setup_settings(&self) {
        let settings = gio::Settings::new("io.github.ronajo");
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &gio::Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();

        let filter: String = self.settings().get("filter");

        match filter.as_str() {
            "nsfw" => {
                imp.enable_nsfw.set_active(true);
                imp.enable_ecchi.set_active(false);
            }
            "nsfw-with-ecchi" => {
                imp.enable_nsfw.set_active(true);
                imp.enable_ecchi.set_active(true);
            }
            "sfw-with-ecchi" => {
                imp.enable_nsfw.set_active(false);
                imp.enable_ecchi.set_active(true);
            }
            "sfw" => {
                imp.enable_nsfw.set_active(false);
                imp.enable_ecchi.set_active(false);
            }
            _ => unreachable!()
        };

        imp.enable_nsfw.connect_active_notify(glib::clone!(
            #[weak(rename_to = ecchi_row)]
            imp.enable_ecchi,
            move |switch| {
                if switch.is_active() {
                    if ecchi_row.is_active() {
                        switch.activate_action("win.filter", Some(&"nsfw-with-ecchi".to_variant()))
                            .expect("action does not exist");
                    } else {
                        switch.activate_action("win.filter", Some(&"nsfw".to_variant()))
                            .expect("action does not exist");
                    }
                } else {
                    if ecchi_row.is_active() {
                        switch.activate_action("win.filter", Some(&"sfw-with-ecchi".to_variant()))
                            .expect("action does not exist");
                    } else {
                        switch.activate_action("win.filter", Some(&"sfw".to_variant()))
                            .expect("action does not exist");
                    }

                }
        }));

        imp.enable_ecchi.connect_active_notify(glib::clone!(
            move |switch| {
                if switch.is_active() {
                    switch.activate_action("win.filter", Some(&"sfw-with-ecchi".to_variant()))
                        .expect("action does not exist");
                } else {
                    switch.activate_action("win.filter", Some(&"sfw".to_variant()))
                        .expect("action does not exist");

                }
        }));

    }
}
