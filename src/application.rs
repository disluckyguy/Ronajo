/* application.rs
 *
 * Copyright 2024 Mostafa
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gdk, gio, glib};

use crate::config::VERSION;
use crate::preferences_dialog::RonajoPreferencesDialog;
use crate::devices_dialog::DevicesDialog;
use crate::RonajoWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct RonajoApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for RonajoApplication {
        const NAME: &'static str = "RonajoApplication";
        type Type = super::RonajoApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for RonajoApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.connect_startup(move |_| {
                let provider = gtk::CssProvider::new();
                provider.load_from_resource("/io/github/Ronajo/gtk/styles.css");

                // Add the provider to the default screen
                gtk::style_context_add_provider_for_display(
                    &gdk::Display::default().expect("Could not connect to a display."),
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            });
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("win.toggle-fullscreen", &["F11"]);
            obj.set_accels_for_action("vid.toggle-mute", &["<Ctrl>m"]);
            obj.set_accels_for_action("vid.toggle-pause", &["<Ctrl>p", "<Ctrl>k"]);
            obj.set_accels_for_action("vid.toggle-autohide", &["<Ctrl>h"]);
            obj.set_accels_for_action("vid.toggle-loop", &["<Ctrl>l"]);
            obj.set_accels_for_action("vid.raise-volume", &["<Ctrl>Up"]);
            obj.set_accels_for_action("vid.lower-volume", &["<Ctrl>Down"]);
            obj.set_accels_for_action("vid.seek-forward", &["<Ctrl>Right"]);
            obj.set_accels_for_action("vid.seek-backward", &["<Ctrl>Left"]);
            obj.set_accels_for_action("vid.raise-rate", &["<Alt>Right"]);
            obj.set_accels_for_action("vid.lower-rate", &["<Alt>Left"]);
        }
    }

    impl ApplicationImpl for RonajoApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {

            let application = self.obj();
            crate::core::config::setup_config();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = RonajoWindow::new(&*application);
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();



            // application.connect_shutdown(glib::clone!(
            //     #[weak]
            //     window,
            //     move |_| {
            //         window.close();
            //     }
            // ));
        }
    }

    impl GtkApplicationImpl for RonajoApplication {}
    impl AdwApplicationImpl for RonajoApplication {}
}

glib::wrapper! {
    pub struct RonajoApplication(ObjectSubclass<imp::RonajoApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl RonajoApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let add_device_action = gio::ActionEntry::builder("devices-dialog")
            .activate(move |app: &Self, _, _| app.devices_dialog())
            .build();

        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| app.show_preferences())
            .build();
        self.add_action_entries([quit_action, about_action, preferences_action, add_device_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("Ronajo")
            .application_icon("io.github.Ronajo")
            .developer_name("Mostafa")
            .version(VERSION)
            .developers(vec!["Mostafa"])
            .copyright("© 2024 Mostafa")
            .build();

        about.present(Some(&window));
    }

    fn show_preferences(&self) {
        let dialog = RonajoPreferencesDialog::new();
        let window = self.active_window().expect("failed to get window");
        dialog.present(Some(&window));
    }

    fn devices_dialog(&self) {
        let dialog = DevicesDialog::new();
        let window = self.active_window().expect("failed to get window");
        dialog.present(Some(&window));
    }
}

