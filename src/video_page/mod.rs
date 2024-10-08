use crate::tools::*;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{self, gdk, glib};
use glib::Object;
use gst::prelude::*;
use gtk;
use crate::runtime;
use std::cell::RefCell;
use std::ops::Add;
use std::time::Instant;
use crate::core::show_data::*;
mod imp;

glib::wrapper! {
    pub struct RonajoVideoPage(ObjectSubclass<imp::RonajoVideoPage>)
    @extends adw::NavigationPage, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoVideoPage {
    pub fn new(allanime_id: &str, title: &str, episode_number: u32, translation: &str, total_episodes: u32) -> Self {
        Object::builder()
            .property("allanime_id", allanime_id)
            .property("title", title)
            .property("translation", translation)
            .property("episode_number", episode_number)
            .property("total_episodes", total_episodes)
            .build()
    }

    pub fn playbin(&self) -> gst::Element {
        self.imp()
            .playbin
            .borrow()
            .clone()
            .expect("failed to get playbin")
    }

    pub fn sink(&self) -> gst::Element {
        self.imp()
            .sink
            .borrow()
            .clone()
            .expect("failed to get playbin")
    }

    pub fn last_hidden(&self) -> Instant {
        self.imp()
            .last_hidden
            .borrow()
            .clone()
            .expect("failed to get last hidden")
    }

    pub fn last_volume_revealed(&self) -> Instant {
        self.imp()
            .last_volume_revealed
            .borrow()
            .clone()
            .expect("failed to get last hidden")
    }

    pub fn last_rate_revealed(&self) -> Instant {
        self.imp()
            .last_rate_revealed
            .borrow()
            .clone()
            .expect("failed to get last hidden")
    }

    pub fn prev_xy(&self) -> (f64, f64) {
        self.imp().prev_xy.get().clone()
    }

    pub fn prev_pos(&self) -> gst::ClockTime {
        self.imp()
            .prev_pos
            .borrow()
            .clone()
            .expect("failed to get last hidden")
    }

    pub fn update_rate(&self) {
        if let Some(duration) = self.playbin().query_duration::<gst::ClockTime>() {
            if let Some(position) = self.playbin().query_position::<gst::ClockTime>() {
                let seek_event = gst::event::Seek::new(
                    self.rate(),
                    gst::SeekFlags::FLUSH,
                    gst::SeekType::Set,
                    position,
                    gst::SeekType::Set,
                    duration,
                );
                let audio_sink = self.playbin().property::<gst::Element>("audio-sink");
                audio_sink.send_event(seek_event);
            } else {
                let seek_event = gst::event::Seek::new(
                    self.rate(),
                    gst::SeekFlags::FLUSH,
                    gst::SeekType::Set,
                    self.prev_pos(),
                    gst::SeekType::Set,
                    duration,
                );
                let audio_sink = self.playbin().property::<gst::Element>("audio-sink");
                audio_sink.send_event(seek_event);
            }
        }
    }

    pub fn play_next_episode(&self) {
        let (sender, receiver) = async_channel::bounded(1);
        // Connect to "clicked" signal of `button`
        runtime().spawn(glib::clone!(
            #[strong(rename_to = id)]
            self.allanime_id(),
            #[strong(rename_to = translation)]
            self.translation(),
            #[strong(rename_to = episode_number)]
            self.episode_number() + 1,
            #[strong]
            sender,
            async move {
                sender
                    .send(None)
                    .await
                    .expect("The channel needs to be open.");
                let uri = api_get_episode(id, translation, episode_number).await.expect("failed to get episode");
                sender
                    .send(Some(uri))
                    .await
                    .expect("The channel needs to be open.");
            }
        ));

    // The main loop executes the asynchronous block
    glib::spawn_future_local(glib::clone!(
        #[weak(rename_to = page)]
        self,
        #[weak(rename_to = video_slider)]
        self.imp().video_slider,
        #[weak(rename_to = play_button)]
        self.imp().play_button,
        #[weak(rename_to = seek_forward)]
        self.imp().seek_forward,
        #[weak(rename_to = seek_backward)]
        self.imp().seek_backward,
        async move {

            while let Ok(response) = receiver.recv().await {
                if let Some(uri) = response {
                    let playbin = page.playbin();
                     playbin.set_state(gst::State::Ready)
                         .expect("failed to set playbin to Null state");
                     playbin.set_property("uri", uri);

                     playbin.set_state(gst::State::Playing)
                         .expect("failed to set playbin to playing state");
                    video_slider.set_sensitive(true);
                    play_button.set_sensitive(true);
                    seek_forward.set_sensitive(true);
                    seek_backward.set_sensitive(true);
                    page.set_episode_number(page.episode_number() + 1);
                } else {
                    video_slider.set_sensitive(false);
                    play_button.set_sensitive(false);
                    seek_forward.set_sensitive(false);
                    seek_backward.set_sensitive(false);
                }

            }
        }));
    }

    pub fn setup_stream(&self) {

        let sink = gst::ElementFactory::make("gtk4paintablesink")
            .build()
            .expect("failed to make paintable sink");
        let playbin = gst::ElementFactory::make("playbin")
            .property("video-sink", &sink)
            .build()
            .expect("failed to make playbin");

        let paintable = sink.property::<gdk::Paintable>("paintable");

        self.imp().picture.set_paintable(Some(&paintable));

        self.imp().playbin.replace(Some(playbin));

        self.imp().sink.replace(Some(sink));

        let (sender, receiver) = async_channel::bounded(1);
        // Connect to "clicked" signal of `button`
        runtime().spawn(glib::clone!(
            #[strong(rename_to = id)]
            self.allanime_id(),
            #[strong(rename_to = translation)]
            self.translation(),
            #[strong(rename_to = episode_number)]
            self.episode_number(),
            #[strong]
            sender,
            async move {
                sender
                    .send(None)
                    .await
                    .expect("The channel needs to be open.");
                let uri = api_get_episode(id, translation, episode_number).await.expect("failed to get episode");
                sender
                    .send(Some(uri))
                    .await
                    .expect("The channel needs to be open.");
            }
        ));

    // The main loop executes the asynchronous block
    glib::spawn_future_local(glib::clone!(
        #[weak(rename_to = page)]
        self,
        #[weak(rename_to = video_slider)]
        self.imp().video_slider,
        #[weak(rename_to = play_button)]
        self.imp().play_button,
        #[weak(rename_to = seek_forward)]
        self.imp().seek_forward,
        #[weak(rename_to = seek_backward)]
        self.imp().seek_backward,
        async move {

            while let Ok(response) = receiver.recv().await {
                if let Some(uri) = response {
                    let playbin = page.playbin();
                     playbin.set_state(gst::State::Ready)
                         .expect("failed to set playbin to Null state");
                     playbin.set_property("uri", uri);

                     playbin.set_state(gst::State::Playing)
                         .expect("failed to set playbin to playing state");
                    video_slider.set_sensitive(true);
                    play_button.set_sensitive(true);
                    seek_forward.set_sensitive(true);
                    seek_backward.set_sensitive(true);
                } else {
                    video_slider.set_sensitive(false);
                    play_button.set_sensitive(false);
                    seek_forward.set_sensitive(false);
                    seek_backward.set_sensitive(false);
                }

            }
    }));


        self.imp().prev_xy.replace((0f64, 0f64));
        self.imp()
            .prev_pos
            .replace(Some(gst::ClockTime::from_seconds(0)));
        self.imp().last_hidden.replace(Some(Instant::now()));
        self.imp()
            .last_volume_revealed
            .replace(Some(Instant::now()));
        self.imp().last_rate_revealed.replace(Some(Instant::now()));

        let playbin = self.playbin();

        let playbin_weak = playbin.downgrade();
        let duration_label = RefCell::new(self.imp().duration_label.get());
        let position_label = RefCell::new(self.imp().position_label.get());
        let video_slider = RefCell::new(self.imp().video_slider.get());
        self.set_rate(1f64);
        let page_weak = self.downgrade();

        let timeout_id =
            glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                let Some(playbin) = playbin_weak.upgrade() else {
                    return glib::ControlFlow::Break;
                };

                let video_slider = video_slider.borrow().clone();
                let position = playbin.query_position::<gst::ClockTime>();
                let duration = playbin.query_duration::<gst::ClockTime>();
                if let Some(duration) = playbin.query_duration::<gst::ClockTime>() {
                    video_slider
                        .adjustment()
                        .set_upper(duration.seconds() as f64);
                };

                if let Some(page) = page_weak.upgrade() {
                    if page.autohide() {
                        if page.last_hidden().elapsed().as_secs() >= 5 {
                            page.imp().header_revealer.set_reveal_child(false);
                            page.imp().controls_revealer.set_reveal_child(false);
                            page.set_cursor_from_name(Some("none"));
                        } else {
                            page.imp().header_revealer.set_reveal_child(true);
                            page.imp().controls_revealer.set_reveal_child(true);
                            page.set_cursor_from_name(Some("default"));
                        }
                    } else {
                        page.imp().header_revealer.set_reveal_child(true);
                        page.imp().controls_revealer.set_reveal_child(true);
                        page.set_cursor_from_name(Some("default"));
                    }
                    if page.last_volume_revealed().elapsed().as_secs() >= 2 {
                        page.imp().volume_slider_revealer.set_reveal_child(false);
                    } else {
                        page.imp().volume_slider_revealer.set_reveal_child(true);
                    }

                    if page.last_rate_revealed().elapsed().as_secs() >= 1 {
                        page.imp().rate_revealer.set_reveal_child(false);
                    } else {
                        page.imp().rate_revealer.set_reveal_child(true);
                    }

                    if let Some(position) = position {
                        page.imp().prev_pos.replace(Some(position));
                    }
                }

                if let Some(position) = position {
                    position_label.borrow().clone().set_label(&format!(
                        "{}",
                        seconds_to_timestamp(position.seconds() as f64)
                    ));
                }

                if let Some(duration) = duration {
                    duration_label.borrow().clone().set_label(&format!(
                        "{}",
                        seconds_to_timestamp(duration.seconds() as f64)
                    ));
                }

                if let Some(position) = position {
                    video_slider
                        .adjustment()
                        .set_value(position.seconds() as f64);
                }

                glib::ControlFlow::Continue
            });

        let bus = playbin.bus().unwrap();

        let page_weak = self.downgrade();

        let bus_watch = bus
            .add_watch_local(
                glib::clone!(
                #[weak(rename_to = page)]
                self,
                #[upgrade_or_panic]
                move |_, msg| {
                use gst::MessageView;

                match msg.view() {

                    MessageView::Eos(..) => {
                        page.playbin()
                            .set_state(gst::State::Null)
                            .expect("Unable to set the playbin to the `Playing` state");

                        page.set_paused(true);

                        if page.loop_video() {
                            page.set_paused(false);
                        } else {
                            if page.episode_number() == page.total_episodes() {
                                page.activate_action("navigation.pop", None)
                                    .expect("action does not exist");
                            } else {
                                page.play_next_episode()
                            }

                        }
                    }
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {} ({:?})",
                            err.src().map(|s| s.path_string()),
                            err.error(),
                            err.debug()
                        );
                    }
                    _ => (),
                };

                glib::ControlFlow::Continue
            }))
            .expect("Failed to add bus watch");

        let timeout_id = RefCell::new(Some(timeout_id));
        let playbin = RefCell::new(Some(self.playbin()));
        let bus_watch = RefCell::new(Some(bus_watch));

        self.connect_destroy(move |_| {
            drop(bus_watch.borrow_mut().take());

            if let Some(playbin) = playbin.borrow_mut().take() {
                playbin
                    .set_state(gst::State::Null)
                    .expect("Unable to set the playbin to the `Null` state");
            }
            if let Some(timeout_id) = timeout_id.borrow_mut().take() {
                timeout_id.remove();
            }
        });
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();

        let playbin = self.playbin();

        let click_event = gtk::GestureClick::new();

        self.connect_unrealize(glib::clone!(
            move |page| {
                page.playbin()
                    .set_state(gst::State::Null)
                    .expect("failed to set state");
            }
        ));

        click_event.connect_pressed(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, _, _, _| {
                page.imp().last_hidden.replace(Some(Instant::now()));
            }
        ));

        self.add_controller(click_event);

        let mouse_motion_event = gtk::EventControllerMotion::new();
        mouse_motion_event.connect_motion(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_, x, y| {
                if page.prev_xy() == (x, y) {
                    return;
                }
                page.imp().last_hidden.replace(Some(Instant::now()));
                page.imp().prev_xy.replace((x, y));
            }
        ));

        self.add_controller(mouse_motion_event);

        let mouse_scroll_event =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);

        let page_weak = self.downgrade();

        mouse_scroll_event.connect_scroll(move |_, _, y| {
            if let Some(page) = page_weak.upgrade() {
                let volume = page.volume();

                if y < 0f64 {
                    let mut new_volume = volume + 5f64;
                    if new_volume > 100f64 {
                        new_volume = 100f64;
                    }
                    page.set_volume(new_volume);
                } else {
                    let mut new_volume = volume - 5f64;
                    if new_volume < 0f64 {
                        new_volume = 0f64;
                    }
                    page.set_volume(new_volume);
                }
            }

            false.into()
        });

        self.add_controller(mouse_scroll_event);

        imp.video_slider.connect_change_value(glib::clone!(
            #[weak]
            playbin,
            #[upgrade_or_panic]
            move |_,_, value| {
                let _ = playbin
                    .seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_seconds(value as u64));

                false.into()
            }
        ));

        self.connect_rate_notify(move |page| {
            page.update_rate();
        });

        self.connect_paused_notify(move |page| {
            if page.paused() {
                let _ = page.playbin()
                    .set_state(gst::State::Paused);
            } else {
                let _ = page.playbin()
                    .set_state(gst::State::Playing);
            }
        });

        imp.rate_label.connect_label_notify(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |_| {
                page.imp().last_rate_revealed.replace(Some(Instant::now()));
            }
        ));

        imp.audio_slider
            .adjustment()
            .connect_value_changed(glib::clone!(
                #[weak(rename_to = page)]
                self,
                move |_| {
                    page.imp()
                        .last_volume_revealed
                        .replace(Some(Instant::now()));
                }
            ));
        let mute_button = self.imp().mute_button.get();
        let menu_mute_button = self.imp().menu_mute_button.get();

        // self.connect_volume_notify(glib::clone!(
        //     #[weak]
        //     mute_button,
        //     #[weak]
        //     menu_mute_button,
        //     move |page| {
        //         let volume = page.volume();

        //         if volume == 0f64 || page.mute() {
        //             mute_button.set_icon_name("audio-volume-muted-symbolic");
                    // if menu_mute_button.is_visible() {
        //             menu_mute_button.set_icon_name("audio-volume-muted-symbolic");
                    // }
        //         } else if volume > 0f64 && volume <= 33f64 {
        //             mute_button.set_icon_name("audio-volume-low-symbolic");

                    // if menu_mute_button.is_visible() {
        //             menu_mute_button.set_icon_name("audio-volume-low-symbolic");
                    // }
        //         } else if volume > 33f64 && volume <= 66f64 {
        //             mute_button.set_icon_name("audio-volume-medium-symbolic");
                    // if menu_mute_button.is_visible() {
        //             menu_mute_button.set_icon_name("audio-volume-medium-symbolic");
                    // }
        //         } else if volume > 66f64 && volume <= 100f64 {
        //             mute_button.set_icon_name("audio-volume-high-symbolic");
                    // if menu_mute_button.is_visible() {
        //             menu_mute_button.set_icon_name("audio-volume-high-symbolic");
                    // }
        //         }
        //     }
        // ));
    }

    pub fn setup_binds(&self) {
        let imp = self.imp();

        self.set_volume(100f64);

        let playbin = self.playbin();

        let pages = gtk::Window::list_toplevels();

        for widget in pages {
            let window = widget
                .downcast_ref::<gtk::Window>()
                .expect("failed to get winodw");
            if window.title() == Some("ronajo".to_string().into()) {
                window
                    .bind_property("fullscreened", &imp.fullscreen_button.get(), "active")
                    .sync_create()
                    .build();
            }
        }

        imp.play_button
            .bind_property("active", &imp.play_button.get(), "icon-name")
            .transform_to(|_, active| {
                let mut icon_name = "media-playback-pause-symbolic";
                if active {
                    icon_name = "media-playback-start-symbolic";
                }

                Some(icon_name)
            })
            .sync_create()
            .build();

        imp.play_button
            .bind_property("active", &imp.play_button.get(), "tooltip-text")
            .transform_to(|_, active| {
                let mut tooltip_text = "Pause";
                if active {
                    tooltip_text = "Play";
                }

                Some(tooltip_text)
            })
            .sync_create()
            .build();

        imp.play_button
            .bind_property("active", self, "paused")
            .bidirectional()
            .sync_create()
            .build();

        imp.fullscreen_button
            .bind_property("active", &imp.fullscreen_button.get(), "icon-name")
            .transform_to(|_, active| {
                let mut icon_name = "arrows-pointing-outward-symbolic";
                if active {
                    icon_name = "arrows-pointing-inward-symbolic";
                }
                Some(icon_name)
            })
            .sync_create()
            .build();

        self.bind_property("volume", &playbin, "volume")
            .bidirectional()
            .transform_to(|_, volume: f64| {
                let new_volume: f64 = volume / 100f64;
                Some(new_volume)
            })
            .transform_from(|_, volume: f64| {
                let new_volume: f64 = volume * 100f64;
                Some(new_volume)

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

        self.bind_property("mute", &playbin, "mute")
            .bidirectional()
            .sync_create()
            .build();

        imp.menu_mute_button
            .bind_property("active", self, "mute")
            .bidirectional()
            .sync_create()
            .build();

        imp.menu_mute_button
            .bind_property("active", self, "mute")
            .bidirectional()
            .sync_create()
            .build();

        imp.lock_ui_button
            .bind_property("active", self, "autohide")
            .bidirectional()
            .invert_boolean()
            .sync_create()
            .build();

        imp.loop_button
            .bind_property("active", self, "loop-video")
            .bidirectional()
            .sync_create()
            .build();

        imp.lock_ui_button
            .bind_property("active", &imp.lock_ui_button.get(), "icon-name")
            .transform_to(|_, active| {
                let mut icon_name = "padlock2-open-symbolic";
                if active {
                    icon_name = "padlock2-symbolic";
                }

                Some(icon_name)
            })
            .sync_create()
            .build();

        self.bind_property("volume", &imp.audio_slider.adjustment(), "value")
            .bidirectional()
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

        self.bind_property("rate", &imp.rate_label.get(), "label")
            .transform_to(|_, rate: f64| {
                let label = format!("{}x", rate);
                Some(label)
            })
            .sync_create()
            .build();
    }
}
