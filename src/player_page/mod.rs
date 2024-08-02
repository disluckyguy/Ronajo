mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use crate::core::player_data::PlayerData;
use std::time::Duration;
use crate::tools::*;
use std::cell::RefCell;

glib::wrapper! {
    pub struct RonajoPlayerPage(ObjectSubclass<imp::RonajoPlayerPage>)
    @extends adw::NavigationPage, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoPlayerPage {
    pub fn new(data: &PlayerData, url: &str) -> Self {
        Object::builder()
            .property("device-name", &data.name)
            .property("address", &data.address)
            .property("username", &data.username)
            .property("password", &data.password)
            .property("use-key", data.use_key)
            .property("url", url)
            .build()
    }

    pub fn data(&self) -> PlayerData {
        self
            .imp()
            .data
            .borrow()
            .clone()
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();
        let data = self.data();



        let page_weak = self.downgrade();

        let timeout_id = glib::timeout_add_local(Duration::from_millis(500),
        move || {
            if let Some(page) = page_weak.upgrade() {
                let data = page.data();

                let (sender, receiver) = async_channel::bounded(1);
                gio::spawn_blocking(move || {

                    if let Ok(position) = data.get_position() {
                        sender
                            .send_blocking(position)
                            .expect("The channel needs to be open.");
                    }

                });

                glib::spawn_future_local(glib::clone!(
                    #[strong]
                    page,
                    async move {
                        while let Ok(position) = receiver.recv().await {
                            page.set_position(position);
                        }
                    }
                ));

                let (sender, receiver) = async_channel::bounded(1);
                let data = page.data();
                gio::spawn_blocking(move || {

                    if let Ok(duration) = data.get_duration() {
                        sender
                            .send_blocking(duration)
                            .expect("The channel needs to be open.");
                    }

                });

                glib::spawn_future_local(glib::clone!(
                    #[strong]
                    page,
                    async move {
                        while let Ok(duration) = receiver.recv().await {
                            page.imp().duration_label.set_label(&seconds_to_timestamp(duration));
                            page.imp().video_scale.adjustment().set_upper(duration);
                        }
                    }
                ));

            };
            glib::ControlFlow::Continue
            });



        let timeout_id = RefCell::new(Some(timeout_id));
        self.connect_unrealize(
            move |_| {
                let data = data.clone();
                gio::spawn_blocking(move || {
                    data.quit().expect("failed to quit");
                });
                if let Some(timeout_id) = timeout_id.borrow_mut().take() {
                    timeout_id.remove();
                }
            }
        );

        let (sender, receiver) = async_channel::bounded(1);

        imp.seek_forward.connect_clicked(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |_| {
                let data = data.clone();
                let sender = sender.clone();
                gio::spawn_blocking(move || {
                    data.seek_forward().expect("failed to seek");
                    sender
                        .send_blocking(10f64)
                        .expect("thread must be open");
                });
            }
        ));

        glib::spawn_future_local(glib::clone!(
        #[weak(rename_to = scale)]
        self.imp().video_scale,
        async move {
            while let Ok(value) = receiver.recv().await {
                println!("sent");

                let adjustment_value = scale.adjustment().value();
                scale.adjustment().set_value(adjustment_value + value);
            }
        }
        ));

        let (sender, receiver) = async_channel::bounded(1);

        imp.seek_backward.connect_clicked(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |_| {
                let data = data.clone();
                let sender = sender.clone();
                gio::spawn_blocking(move || {
                    data.seek_backward().expect("failed to seek");
                    sender
                        .send_blocking(10f64)
                        .expect("thread must be open");
                });
            }
        ));

        glib::spawn_future_local(glib::clone!(
        #[weak(rename_to = scale)]
        self.imp().video_scale,
        async move {
            while let Ok(value) = receiver.recv().await {
                println!("sent");
                let adjustment_value = scale.adjustment().value();
                scale.adjustment().set_value(adjustment_value - value);
            }
        }
        ));

        imp.quit_button.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| {
                page.activate_action("navigation.pop", Some(&"player-page".to_variant())).expect("failed to activate action");
            }
        ));

        imp.screenshot_button.connect_clicked(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |_| {
                let data = data.clone();
                gio::spawn_blocking(move || {
                    data.screenshot().expect("failed to screenshot");
                });
            }
        ));

        self.connect_muted_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                if page.muted() {

                    gio::spawn_blocking(move || {
                        data.set_muted(true).expect("failed to mute");
                    });

                } else {

                    gio::spawn_blocking(move || {
                        data.set_muted(false).expect("failed to unmute");
                    });
                }
            }
        ));

        self.connect_paused_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();

                if page.paused() {
                    gio::spawn_blocking(move || {
                        data.set_paused(true).expect("failed to mute");
                    });

                } else {
                    gio::spawn_blocking(move || {
                        data.set_paused(false).expect("failed to unmute");
                    });

                }
            }
        ));

        imp.video_scale.connect_change_value(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |_scale, _, value| {
                let data = data.clone();
                let value = value.clone();
                println!("{}", value);
                gio::spawn_blocking(move || {
                    let _ = data.seek_to(value);
                });
                false.into()
            }
        ));


        self.connect_rate_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                let rate = page.rate();
                gio::spawn_blocking(move || {
                    data.set_rate(rate).expect("failed to mute");
                });
            }
        ));

        self.connect_volume_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                let volume = page.volume();
                gio::spawn_blocking(move || {
                    data.set_volume(volume).expect("failed to mute");
                });
            }
        ));
    }

    pub fn setup_binds(&self) {
    let imp = self.imp();
        imp.mute_button.bind_property("active", self, "muted")
            .bidirectional()
            .sync_create()
            .build();

        imp.play_button.bind_property("active", self, "paused")
            .bidirectional()
            .sync_create()
            .build();

        imp.play_button.bind_property("active", &imp.play_image.get(), "icon-name")
            .transform_to(move |_, active| {
                if active {
                    Some("media-playback-start-symbolic".to_string())
                } else {
                    Some("media-playback-pause-symbolic".to_string())
                }
            })
            .sync_create()
            .build();

        self.bind_property("volume", &imp.volume_spin.adjustment(), "value")
            .bidirectional()
            .sync_create()
            .build();

        self.bind_property("rate", &imp.rate_spin.adjustment(), "value")
            .bidirectional()
            .sync_create()
            .build();

        self.bind_property("position", &imp.video_scale.adjustment(), "value")
            .sync_create()
            .build();

        self.bind_property("position", &imp.position_label.get(), "label")
            .transform_to(move |_, position| {
                Some(seconds_to_timestamp(position))
            })
            .sync_create()
            .build();


    }
}


