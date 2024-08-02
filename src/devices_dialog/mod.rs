mod imp;

use adw::{glib, gio};
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use crate::device_object::DeviceObject;
use crate::core::player_data::PlayerData;
use crate::core::config::*;

glib::wrapper! {
    pub struct DevicesDialog(ObjectSubclass<imp::DevicesDialog>)
    @extends adw::Dialog, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DevicesDialog {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn devices(&self) -> gio::ListStore {
        self
            .imp()
            .devices
            .borrow()
            .clone()
            .expect("failed to get devices")
    }

    pub fn new_device(&self, data: &PlayerData) {
        let object = DeviceObject::from_player_data(data);
        self.devices().append(&object);
    }

    pub fn reset_fields(&self) {
        let imp = self.imp();
        imp.name_row.set_text("");
        imp.address_row.set_text("");
        imp.username_row.set_text("");
        imp.password_row.set_text("");
    }


    pub fn setup_callbacks(&self) {
        let imp = self.imp();

        imp.add_button.connect_clicked(glib::clone!(
            #[weak(rename_to = navigation_view)]
            imp.navigation_view,
            #[weak(rename_to = page)]
            self,
            move |_| {
                navigation_view.push_by_tag("add-device-page");
                page.reset_fields();
            }
        ));

        imp.status_add.connect_clicked(glib::clone!(
            #[weak(rename_to = navigation_view)]
            imp.navigation_view,
            #[weak(rename_to = page)]
            self,
            move |_| {
                navigation_view.push_by_tag("add-device-page");
                page.reset_fields();
            }
        ));

        imp.key_auth_button.connect_toggled(glib::clone!(
            #[weak(rename_to = save_button)]
            imp.save_button.get(),
            #[strong(rename_to = address_row)]
            imp.address_row.get(),
            #[strong(rename_to = username_row)]
            imp.username_row.get(),
            #[strong(rename_to = password_row)]
            imp.password_row.get(),
            #[strong(rename_to = empty_password)]
            imp.empty_password.get(),
            move |button| {
                let address_empty = address_row.text().is_empty();
                let username_empty = username_row.text().is_empty();
                let password_empty = password_row.text().is_empty();
                if button.is_active() {
                    empty_password.set_visible(false);
                    if username_empty || address_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }
                } else {
                    if username_empty || address_empty || password_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }
                    if password_empty {
                        empty_password.set_visible(true);
                    } else {
                        empty_password.set_visible(false);
                    }
                }
        }));

        imp.password_row.connect_changed(glib::clone!(
            #[weak(rename_to = save_button)]
            imp.save_button.get(),
            #[strong(rename_to = address_row)]
            imp.address_row.get(),
            #[strong(rename_to = username_row)]
            imp.username_row.get(),
            #[strong(rename_to = key_auth_button)]
            imp.key_auth_button.get(),
            #[strong(rename_to = empty_password)]
            imp.empty_password.get(),
            move |password_row| {
                let address_empty = address_row.text().is_empty();
                let username_empty = username_row.text().is_empty();
                let password_empty = password_row.text().is_empty();
                if key_auth_button.is_active() {
                    if username_empty || address_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }
                    empty_password.set_visible(false);
                } else {
                    if username_empty || address_empty || password_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }
                    if password_empty {
                        empty_password.set_visible(true);
                    } else {
                        empty_password.set_visible(false);
                    }
                }
        }));

        imp.username_row.connect_changed(glib::clone!(
            #[weak(rename_to = save_button)]
            imp.save_button.get(),
            #[strong(rename_to = address_row)]
            imp.address_row.get(),
            #[strong(rename_to = password_row)]
            imp.password_row.get(),
            #[strong(rename_to = key_auth_button)]
            imp.key_auth_button.get(),
            #[strong(rename_to = empty_username)]
            imp.empty_username.get(),
            move |username_row| {
                let address_empty = address_row.text().is_empty();
                let username_empty = username_row.text().is_empty();
                let password_empty = password_row.text().is_empty();
                if key_auth_button.is_active() {
                    if username_empty || address_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }


                } else {
                    if username_empty || address_empty || password_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }

                }
                if username_empty {
                    empty_username.set_visible(true);
                } else {
                    empty_username.set_visible(false);
                }
        }));

        imp.address_row.connect_changed(glib::clone!(
            #[weak(rename_to = save_button)]
            imp.save_button.get(),
            #[strong(rename_to = username_row)]
            imp.username_row.get(),
            #[strong(rename_to = password_row)]
            imp.password_row.get(),
            #[strong(rename_to = key_auth_button)]
            imp.key_auth_button.get(),
            #[strong(rename_to = empty_address)]
            imp.empty_address.get(),
            move |address_row| {
                let address_empty = address_row.text().is_empty();
                let username_empty = username_row.text().is_empty();
                let password_empty = password_row.text().is_empty();
                if key_auth_button.is_active() {
                    if username_empty || address_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }


                } else {
                    if username_empty || address_empty || password_empty {
                        save_button.set_sensitive(false);
                    } else {
                        save_button.set_sensitive(true);
                    }

                }
                if address_empty {
                    empty_address.set_visible(true);
                } else {
                    empty_address.set_visible(false);
                }
            }));

            imp.save_button.connect_clicked(glib::clone!(
                #[weak(rename_to = name_row)]
                imp.name_row.get(),
                #[weak(rename_to = address_row)]
                imp.address_row.get(),
                #[weak(rename_to = username_row)]
                imp.username_row.get(),
                #[weak(rename_to = password_row)]
                imp.password_row.get(),
                #[weak(rename_to = key_auth_button)]
                imp.key_auth_button.get(),
                #[weak(rename_to = page)]
                self,
                move |button| {
                    let name = name_row.text().to_string();
                    let address = address_row.text().to_string();
                    let username = username_row.text().to_string();
                    let password = password_row.text().to_string();
                    let use_key = key_auth_button.is_active();

                    if use_key {
                        let data = PlayerData::new(name, address, username, None, use_key);
                        save_device(&data).expect("failed to save");
                        page.new_device(&data);
                    } else {
                        let data = PlayerData::new(name, address, username, Some(password), use_key);
                        save_device(&data).expect("failed to save");
                        page.new_device(&data);
                    }

                    button.activate_action("navigation.pop", None).expect("action does not exist");

                }
            ));

            let (sender, receiver) = async_channel::bounded(1);


        imp.save_button.connect_clicked(glib::clone!(
        #[weak(rename_to = name_row)]
        imp.name_row.get(),
        #[weak(rename_to = address_row)]
        imp.address_row.get(),
        #[weak(rename_to = username_row)]
        imp.username_row.get(),
        #[weak(rename_to = password_row)]
        imp.password_row.get(),
        #[weak(rename_to = key_auth_button)]
        imp.key_auth_button.get(),
        move |_| {
            let name = name_row.text();
            let address = address_row.text();
            let username = username_row.text();
            let password = password_row.text();
            let use_key = key_auth_button.is_active();
            let sender = sender.clone();
            gio::spawn_blocking(move || {
                sender
                    .send_blocking((false, None ))
                    .expect("thread must be open");
                let player  = if use_key {
                    PlayerData::new(name.to_string(), address.to_string(), username.to_string(), None, use_key)
                } else {
                    PlayerData::new(name.to_string(), address.to_string(), username.to_string(), Some(password.to_string()), use_key)
                };

                if let Err(err) = player.validate() {
                    sender
                        .send_blocking((true, Some(err.to_string())))
                        .expect("thread must be open");
                } else {
                    sender
                        .send_blocking((true, None ))
                        .expect("thread must be open");
                }


            });
        }));



        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = button)]
            imp.save_button.get(),
            #[strong(rename_to = address_row)]
            imp.address_row.get(),
            #[strong(rename_to = username_row)]
            imp.username_row.get(),
            #[strong(rename_to = password_row)]
            imp.password_row.get(),
            #[strong(rename_to = key_auth_button)]
            imp.key_auth_button.get(),
            #[strong(rename_to = toast_overlay)]
            imp.add_device_toast_overlay.get(),
            async move {
                while let Ok(enable_button) = receiver.recv().await {
                    button.set_sensitive(enable_button.0);
                    address_row.set_sensitive(enable_button.0);
                    username_row.set_sensitive(enable_button.0);
                    password_row.set_sensitive(enable_button.0);
                    key_auth_button.set_sensitive(enable_button.0);

                    if let Some(err) = enable_button.1 {
                        let toast = adw::Toast::new(&err);

                        match err.trim() {
                            "failed to lookup address information: Temporary failure in name resolution" => toast.set_title("Couldn't connect to address"),
                            "[Session(-18)] Authentication failed (username/password)" => toast.set_title("Invalid username or password"),
                            "[Session(-42)] failed connecting with agent" => toast.set_title("Agent unavailable"),
                            _ => {}
                        };

                        toast_overlay.add_toast(toast);
                    } else {
                        if enable_button.0 {
                            button.activate_action("navigation.pop", None).expect("action doesn't exist");
                        }
                    }
                }

        }));
        }



    pub fn create_device_row(&self, object: &DeviceObject) -> adw::ActionRow {
        let delete_button = gtk::Button::builder()
            .icon_name("edit-delete-symbolic")
            .build();

        delete_button.connect_clicked(glib::clone!(
        #[weak]
        object,
        #[weak(rename_to = page)]
        self,
        move |_| {
            let _ = remove_device(&object.device_name());
            let pos = page.devices().find(&object).expect("failed to find device object");
            page.devices().remove(pos);
        }));

        delete_button.set_css_classes(&["flat"]);
        let row = adw::ActionRow::builder()
            .activatable(false)
            .build();
        row.add_suffix(&delete_button);

        object.bind_property("device-name", &row, "title")
            .sync_create()
            .build();

        row
    }

    pub fn setup_devices(&self) {
        let model = gio::ListStore::new::<DeviceObject>();
        self.imp().devices.replace(Some(model));
        let selection_model = gtk::NoSelection::new(Some(self.devices()));



        for device in devices().expect("failed to get devices") {
            self.new_device(&device);
        }

        self.imp().devices_list.bind_model(Some(&selection_model), glib::clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or_panic]
            move |obj| {
                let device_object = obj
                    .downcast_ref::<DeviceObject>()
                    .expect("object needs to be Device Object");
                let row = page.create_device_row(&device_object);

                row.upcast()
            }
        ));
    }

    pub fn setup_binds(&self) {
        let imp = self.imp();
        self.devices().bind_property("n-items", &imp.devices_stack.get(), "visible-child-name")
        .transform_to(move |_, n_items: u32| {
            if n_items == 0 {
                Some("no-devices".to_value())
            } else {
                Some("devices-list".to_value())
            }
        })
        .sync_create()
        .build();

    }
}
