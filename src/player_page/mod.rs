mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use crate::core::player_data::PlayerData;
use std::time::Duration;
use crate::tools::*;
use std::cell::RefCell;
use crate::window::VideoData;
use crate::runtime;
use crate::core::show_data::*;

glib::wrapper! {
    pub struct RonajoPlayerPage(ObjectSubclass<imp::RonajoPlayerPage>)
    @extends adw::NavigationPage, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoPlayerPage {
    pub fn new(device_data: &PlayerData, video_data: &VideoData, player: &str) -> Self {
        Object::builder()
            .property("device-name", &device_data.name)
            .property("address", &device_data.address)
            .property("username", &device_data.username)
            .property("password", &device_data.password)
            .property("use-key", device_data.use_key)
            .property("player", player)
            .property("allanime_id", &video_data.allanime_id)
            .property("title", &video_data.title)
            .property("translation", &video_data.translation)
            .property("episode_number", video_data.episode_number)
            .property("total_episodes", video_data.total_episodes)
            .build()
    }

    pub fn data(&self) -> PlayerData {
        self
            .imp()
            .data
            .borrow()
            .clone()
    }

    pub fn play_next_episode(&self) {


        let (sender, receiver) = async_channel::bounded(1);
        let data = self.data();
        let player = self.player();
        let allanime_id = self.allanime_id();
        let translation = self.translation();
        let episode_number = self.episode_number() + 1;

        self.data().quit(&player).expect("failed to quit");
        runtime().spawn(
        async move  {
            sender
                .send(false)
                .await
                .expect("thread must be open");
            let uri = api_get_episode(allanime_id, translation, episode_number).await.expect("failed to get episode");
            data.start_session(&uri, &player).expect("failed to start session");
            sender
                .send(true)
                .await
                .expect("thread must be open");
        });

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = stack)]
            self.imp().stack,
            #[weak(rename_to = page)]
            self,
            async move {
                while let Ok(show_player) = receiver.recv().await {
                    if show_player {
                        stack.set_visible_child_name("player");
                        page.set_episode_number(page.episode_number() + 1);

                    } else {
                        stack.set_visible_child_name("spinner");
                    }
                }
            }
        ));
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();


        let page_weak = self.downgrade();

        let timeout_id = glib::timeout_add_local(Duration::from_millis(500),
        move || {
            if let Some(page) = page_weak.upgrade() {
                let data = page.data();
                let player = page.player();

                let (sender, receiver) = async_channel::bounded(1);
                gio::spawn_blocking(move || {

                    if let Ok(position) = data.get_position(&player) {
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
                let player = page.player();

                gio::spawn_blocking(move || {

                    if let Ok(duration) = data.get_duration(&player) {
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
                            page.set_duration(duration);
                            page.imp().duration_label.set_label(&seconds_to_timestamp(duration));
                            page.imp().video_scale.adjustment().set_upper(duration);
                        }
                    }
                ));

            };
            glib::ControlFlow::Continue
            });


        imp.video_scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |scale| {

                let value = scale.value();
                let difference = (page.duration() - value).abs();
                if difference <= 10f64 && page.duration() != 0f64 {
                if page.episode_number() == page.total_episodes() {
                    page.activate_action("navigation.pop", None)
                        .expect("action does not exist");
                } else {
                    scale.set_value(0f64);
                    page.play_next_episode();
                }
            }
        }));



        let timeout_id = RefCell::new(Some(timeout_id));
        self.connect_unrealize(
            move |page| {
                let data = page.data();
                let player = page.player();

                gio::spawn_blocking(move || {
                    data.quit(&player).expect("failed to quit");
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
            #[strong(rename_to = player)]
            self.player(),
            move |_| {
                let data = data.clone();
                let player = player.clone();
                let sender = sender.clone();
                gio::spawn_blocking(move || {
                    data.seek_forward(&player).expect("failed to seek");
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

                let adjustment_value = scale.adjustment().value();
                scale.adjustment().set_value(adjustment_value + value);
            }
        }
        ));

        let (sender, receiver) = async_channel::bounded(1);

        imp.seek_backward.connect_clicked(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            #[strong(rename_to = player)]
            self.player(),
            move |_| {
                let data = data.clone();
                let player = player.clone();
                let sender = sender.clone();
                gio::spawn_blocking(move || {
                    data.seek_backward(&player).expect("failed to seek");
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
                let adjustment_value = scale.adjustment().value();
                scale.adjustment().set_value(adjustment_value - value);
            }
        }
        ));

        imp.quit_button.connect_clicked(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| {
                page.activate_action("navigation.pop", None).expect("failed to activate action");
            }
        ));

        imp.screenshot_button.connect_clicked(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            #[strong(rename_to = player)]
            self.player(),
            move |_| {
                let data = data.clone();
                let player = player.clone();
                gio::spawn_blocking(move || {
                    data.screenshot(&player).expect("failed to screenshot");
                });
            }
        ));

        self.connect_muted_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                let player = page.player();
                if page.muted() {

                    gio::spawn_blocking(move || {
                        data.set_muted(true, &player).expect("failed to mute");
                    });

                } else {

                    gio::spawn_blocking(move || {
                        data.set_muted(false, &player).expect("failed to unmute");
                    });
                }
            }
        ));

        self.connect_paused_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                let player = page.player();

                if page.paused() {
                    gio::spawn_blocking(move || {
                        data.set_paused(true, &player).expect("failed to mute");
                    });

                } else {
                    gio::spawn_blocking(move || {
                        data.set_paused(false, &player).expect("failed to unmute");
                    });

                }
            }
        ));

        imp.video_scale.connect_change_value(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            #[strong(rename_to = player)]
            self.player(),
            move |_scale, _, value| {
                let data = data.clone();
                let value = value.clone();
                let player = player.clone();
                gio::spawn_blocking(move || {
                    let _ = data.seek_to(value, &player);
                });
                false.into()
            }
        ));


        self.connect_rate_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                let player = page.player();
                let rate = page.rate();
                gio::spawn_blocking(move || {
                    data.set_rate(rate, &player).expect("failed to mute");
                });
            }
        ));

        self.connect_volume_notify(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |page| {
                let data = data.clone();
                let volume = page.volume();
                let player = page.player();
                gio::spawn_blocking(move || {
                    data.set_volume(volume, &player).expect("failed to mute");
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

        self.bind_property("episode_number", &imp.episode_label.get(), "label")
            .transform_to(move |_, episode_number: u32| {
                Some(format!("Episode {}", episode_number).to_value())
            })
            .sync_create()
            .build();

        self.bind_property("title", &imp.title_label.get(), "label")
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
