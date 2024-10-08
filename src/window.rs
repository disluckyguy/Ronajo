/* window.rs
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
use gio::Settings;
use crate::core::show_data::*;
use crate::core::player_data::PlayerData;
use crate::core::config::*;
use crate::runtime;
use crate::show_card::RonajoShowCard;
use crate::show_object::ShowObject;
use crate::show_page::RonajoShowPage;
use crate::video_page::RonajoVideoPage;
use crate::player_page::RonajoPlayerPage;
use adw::prelude::*;
use gst::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use std::cell;
use std::cell::OnceCell;
use std::ops::{Add, Sub};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "src/gtk/window.blp")]
    pub struct RonajoWindow {
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub show_view: TemplateChild<gtk::GridView>,
        #[template_child]
        pub library_view: TemplateChild<gtk::GridView>,
        #[template_child]
        pub show_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub library_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub shows_search_bar: TemplateChild<gtk::SearchBar>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        pub shows: cell::RefCell<Option<gio::ListStore>>,
        pub library_shows: cell::RefCell<Option<gio::ListStore>>,
        pub settings: OnceCell<gio::Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RonajoWindow {
        const NAME: &'static str = "RonajoWindow";
        type Type = super::RonajoWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RonajoWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();

            let obj = self.obj();

            obj.setup_settings();
            obj.setup_gactions();

            obj.setup_shows();
            obj.setup_callbacks();
            obj.setup_factory();

            obj.setup_library_shows();
            obj.setup_library_callbacks();
            obj.setup_library_factory();
            obj.setup_bindings();

        }
    }
    impl WidgetImpl for RonajoWindow {}
    impl WindowImpl for RonajoWindow {}
    impl ApplicationWindowImpl for RonajoWindow {}
    impl AdwApplicationWindowImpl for RonajoWindow {}
}

glib::wrapper! {
    pub struct RonajoWindow(ObjectSubclass<imp::RonajoWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl RonajoWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }


    pub fn change_config_path(&self) {
        let file_dialog = gtk::FileDialog::new();
        let cancellable = gtk::gio::Cancellable::new();
        file_dialog.select_folder(Some(self), Some(&cancellable), |result| {
            if let Ok(file) = result {

                let new_path = file
                    .path()
                    .expect("failed to get file path")
                    .into_os_string()
                    .into_string()
                    .expect("failed to convert to string");
                crate::core::config::change_config_path(new_path);
            }
        });
    }

    pub fn reload_library(&self) {
        self.library_shows().remove_all();
        for show in library_shows().expect("failed to get library shows") {
            self.new_library_show(show.0);
        }
    }

    pub fn play_video(&self, _action: &gio::SimpleAction, param: Option<&adw::glib::Variant>) {
        let view = self.imp().navigation_view.get();
        let parameter = param
            .expect("Could not get parameter.")
            .get::<String>()
            .expect("The variant needs to be of type `String`.");

        let data: VideoData = serde_json::from_str(&parameter).expect("failed to deserialize");
        let video_page = RonajoVideoPage::new(&data.allanime_id, &data.title, data.episode_number, &data.translation, data.total_episodes);



        view.push(&video_page);
    }

    pub fn play_remote_video(&self, _action: &gio::SimpleAction, param: Option<&adw::glib::Variant>) {
        let view = self.imp().navigation_view.get();
        let parameter = param
            .expect("Could not get parameter.")
            .get::<String>()
            .expect("The variant needs to be of type `String`.");

        let (sender, receiver) = async_channel::bounded(1);

        let data: RemoteVideoData = serde_json::from_str(&parameter).expect("failed to parse");
        let player: String = self.settings().get("player");

        let entry = adw::PasswordEntryRow::builder()
                .title("password")
                .build();

            let list_box = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .css_classes(["boxed-list"])
                .build();

            list_box.append(&entry);

            let alert_dialog = adw::AlertDialog::builder()
                .heading("Device Password")
                .body("Type the Device's Password to Use It")
                .close_response("cancel")
                .default_response("open")
                .extra_child(&list_box)
                .build();
            alert_dialog.add_responses(&[("cancel", "Cancel"),("continue", "Continue")]);
            alert_dialog.set_response_appearance("cancel", adw::ResponseAppearance::Destructive);
            alert_dialog.set_response_appearance("continue", adw::ResponseAppearance::Suggested);

            alert_dialog.connect_response(None, glib::clone!(
            #[strong]
            data,
            #[weak]
            entry,
            #[strong]
            sender,
            move |_, response| {
                if response == "continue" {

                    let mut data = data.clone();

                    data.device_data.password = Some(entry.text().to_string());
                    let sender = sender.clone();
                    gio::spawn_blocking(move || {
                        sender
                            .send_blocking((false, None ))
                            .expect("thread must be open");


                        if let Err(err) = data.device_data.validate() {

                            sender
                                .send_blocking((true, Some(err.to_string())))
                                .expect("thread must be open");
                        } else {

                            sender
                                .send_blocking((true, None))
                                .expect("thread must be open");
                        }


                    });
                }
            }));

        if !data.device_data.use_key {

            alert_dialog.present(Some(self));
        } else {
            let mut data = data.clone();
            data.device_data.password = None;
            let player_page = RonajoPlayerPage::new(&data.device_data, &data.video_data, &player);
            view.push(&player_page);
        }

        glib::spawn_future_local(glib::clone!(
            #[strong]
            data,
            #[weak]
            entry,
            #[strong]
            player,
            #[weak]
            view,
            async move {
                while let Ok(enable_button) = receiver.recv().await {

                    if let None = enable_button.1 {
                        if enable_button.0 {
                            let mut data = data.clone();
                            data.device_data.password = Some(entry.text().to_string());
                            let player_page = RonajoPlayerPage::new(&data.device_data, &data.video_data, &player);
                            view.push(&player_page);
                        }
                    }
                }

        }));

    }

    pub fn toggle_fullscreen(&self) {
        if !self.is_fullscreen() {
            self.fullscreen();
        } else {
            self.unfullscreen();
        }
    }

    pub fn toggle_pause(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            if video_page.paused() {
                video_page.set_paused(false);
            } else {
                video_page.set_paused(true);
            }
        };

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let paused = player_page.paused();

            player_page.set_paused(!paused);

        };
    }

    pub fn toggle_autohide(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            if video_page.autohide() {
                video_page.set_autohide(false);
            } else {
                video_page.set_autohide(true);
            }
        };
    }

    pub fn toggle_mute(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            if video_page.mute() {
                video_page.set_mute(false);
            } else {
                video_page.set_mute(true);
            }
        };

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let mute = player_page.muted();

            player_page.set_muted(!mute);

        };

    }

    pub fn raise_volume(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            let volume = video_page.volume();
            let mut new_volume = volume + 10f64;
            if new_volume > 100f64 {
                new_volume = 100f64;
            }
            video_page.set_volume(new_volume);
        };

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let volume = player_page.volume();
            if volume + 10f64 >= 100f64 {
                player_page.set_volume(100f64);
            } else {
                player_page.set_volume(volume + 10f64);
            }

        };
    }

    pub fn lower_volume(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            let volume = video_page.volume();
            let mut new_volume = volume - 10f64;
            if new_volume < 0f64 {
                new_volume = 0f64;
            }
            video_page.set_volume(new_volume);
        };

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let volume = player_page.volume();
            if volume - 10f64 <= 0f64 {
                player_page.set_volume(0f64);
            } else {
                player_page.set_volume(volume - 10f64);
            }

        };
    }

    pub fn raise_rate(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            let rate = video_page.rate();
            let mut new_rate = rate + 0.25;
            if new_rate > 2f64 {
                new_rate = 2f64;
            }
            video_page.set_rate(new_rate);
        };

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let rate = player_page.rate();
            if rate + 0.25 >= 2f64 {
                player_page.set_rate(2f64);
            } else {
                player_page.set_rate(rate + 0.25);
            }

        };
    }

    pub fn lower_rate(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            let rate = video_page.rate();
            let mut new_rate = rate - 0.25;
            if new_rate < 0.25 {
                new_rate = 0.25;
            }
            video_page.set_rate(new_rate);
        };

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let rate = player_page.rate();
            if rate - 0.25 <= 0.25 {
                player_page.set_rate(0.25);
            } else {
                player_page.set_rate(rate - 0.25);
            }

        };
    }

    pub fn seek_forward(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            let playbin = video_page.playbin();
            if let Some(stream_time) = playbin.query_position::<gst::ClockTime>() {
                let seek_time = gst::ClockTime::from_seconds(10).add(stream_time);
                let duration = playbin
                    .query_duration::<gst::ClockTime>()
                    .expect("failed to get duration");
                if seek_time.seconds() > duration.seconds() {
                    let _ = playbin
                        .seek_simple(
                            gst::SeekFlags::FLUSH,
                            duration.sub(gst::ClockTime::from_seconds(1)),
                        );
                    return;
                }
                let _ = playbin
                    .seek_simple(gst::SeekFlags::FLUSH, seek_time);
            };
        }

        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let data = player_page.data();
            let player = player_page.player();

            gio::spawn_blocking(move || {
                data.seek_forward(&player).expect("failed to seek");
            });
        };
    }

    pub fn seek_backward(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            let playbin = video_page.playbin();
            if let Some(stream_time) = playbin.query_position::<gst::ClockTime>() {
                let seek_time = stream_time.overflowing_sub(gst::ClockTime::from_seconds(10));

                if seek_time.1 {
                    let _ = playbin
                        .seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_seconds(0));
                    return;
                }
                let _ = playbin
                    .seek_simple(gst::SeekFlags::FLUSH, seek_time.0);
            };
        }
        if let Some(player_page) = visible_page.downcast_ref::<RonajoPlayerPage>() {
            let data = player_page.data();
            let player = player_page.player();

            gio::spawn_blocking(move || {
                data.seek_backward(&player).expect("failed to seek");
            });
        };
    }


    pub fn toggle_loop(&self) {
        let view = self.imp().navigation_view.get();
        let visible_page = view.visible_page().expect("failed to get page");
        if let Some(video_page) = visible_page.downcast_ref::<RonajoVideoPage>() {
            if video_page.loop_video() {
                video_page.set_loop_video(false);
            } else {
                video_page.set_loop_video(true);
            }
        }
    }

    fn setup_gactions(&self) {

        let reload_library_action = gio::ActionEntry::builder("reload-library")
            .activate(move |window: &Self, _, _| window.reload_library())
            .build();

        let change_config_action = gio::ActionEntry::builder("change-config")
            .activate(move |window: &Self, _, _| window.change_config_path())
            .build();

        let play_video_action = gio::ActionEntry::builder("play-video")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |window: &Self, action, parameter| window.play_video(action, parameter))
            .build();

        let play_remote_video_action = gio::ActionEntry::builder("play-remote-video")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |window: &Self, action, parameter| window.play_remote_video(action, parameter))
            .build();

        let toggle_fullscreen_action = gio::ActionEntry::builder("toggle-fullscreen")
            .activate(move |window: &Self, _, _| window.toggle_fullscreen())
            .build();

        let action_filter = self.settings().create_action("filter");
        self.add_action(&action_filter);

        self.add_action_entries([change_config_action, play_video_action, play_remote_video_action, toggle_fullscreen_action, reload_library_action,]);


        let seek_forward_action = gio::ActionEntry::builder("seek-forward")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.seek_forward()
            ))
            .build();

        let seek_backward_action = gio::ActionEntry::builder("seek-backward")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.seek_backward()
            ))
            .build();

        let toggle_pause_action = gio::ActionEntry::builder("toggle-pause")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.toggle_pause()
            ))
            .build();

        let toggle_autohide_action = gio::ActionEntry::builder("toggle-autohide")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.toggle_autohide()
            ))
            .build();

        let toggle_loop_action = gio::ActionEntry::builder("toggle-loop")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.toggle_loop()
            ))
            .build();

        let toggle_mute_action = gio::ActionEntry::builder("toggle-mute")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.toggle_mute()
            ))
            .build();

        let raise_volume_action = gio::ActionEntry::builder("raise-volume")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.raise_volume()
            ))
            .build();

        let lower_volume_action = gio::ActionEntry::builder("lower-volume")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.lower_volume()
            ))
            .build();

        let raise_rate_action = gio::ActionEntry::builder("raise-rate")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.raise_rate()
            ))
            .build();

        let lower_rate_action = gio::ActionEntry::builder("lower-rate")
            .activate(glib::clone!(
                #[weak(rename_to= window)]
                self,
                move |_: &gio::SimpleActionGroup, _, _| window.lower_rate()
            ))
            .build();

        let video_actions = gio::SimpleActionGroup::new();
        video_actions.add_action_entries([
            seek_forward_action,
            seek_backward_action,
            toggle_loop_action,
            toggle_pause_action,
            toggle_autohide_action,
            toggle_mute_action,
            raise_rate_action,
            lower_rate_action,
            raise_volume_action,
            lower_volume_action,
        ]);

        self.insert_action_group("vid", Some(&video_actions));
    }

    fn setup_settings(&self) {
        let settings = Settings::new("io.github.Ronajo");
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    pub fn shows(&self) -> gio::ListStore {
        self.imp()
            .shows
            .borrow()
            .clone()
            .expect("failed to get shows")
    }

    pub fn setup_shows(&self) {
        let imp = self.imp();
        let model = gio::ListStore::new::<ShowObject>();

        imp.shows.replace(Some(model));

        let filter_model = gtk::FilterListModel::new(Some(self.shows()), self.filter());
        let selection_model = gtk::NoSelection::new(Some(filter_model.clone()));

        imp.show_view.set_model(Some(&selection_model));

        self.settings().connect_changed(
            Some("filter"),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[weak]
                filter_model,
                move |_, _| {

                    filter_model.set_filter(window.filter().as_ref());
                }
            ),
        );
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();



        imp.stack.connect_visible_child_notify(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |stack| {
                if stack.visible_child_name().unwrap() == "library" {
                    window.library_shows().remove_all();
                    for show in library_shows().expect("failed to get library shows") {
                        window.new_library_show(show.0);
                    }
                }
        }));
        let (sender, receiver) = async_channel::bounded(1);
        imp.search_entry.connect_search_changed(glib::clone!(
                move |entry| {
            let sender = sender.clone();
            let text = entry.text().trim().to_string();
            runtime().spawn(glib::clone!(
                #[strong]
                text,
                async move {
                    sender.send(None)
                        .await
                        .expect("thread must be open");
                    let response = jikan_search(text)
                        .await
                        .expect("failed to search");
                    sender.send(Some(response))
                        .await
                        .expect("thread must be open");
                }
            ));


        }));
        glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                async move {
                    while let Ok(data) = receiver.recv().await {
                        if let Some(shows) = data {
                            obj.shows().remove_all();
                            if shows.is_empty() {
                                obj.imp().show_stack.set_visible_child_name("empty-shows");
                                return;
                            }

                            for iter in shows {
                                obj.new_show(iter);
                            }
                            obj.imp().show_stack.set_visible_child_name("show-page")
                        } else {
                            obj.imp().show_stack.set_visible_child_name("spinner");
                        }

                    }
                }
            ));
    }

    pub fn setup_bindings(&self) {
        let imp = self.imp();

        self.library_shows()
            .bind_property("n-items", &imp.library_stack.get(), "visible-child-name")
            .transform_to(|_, n_items: u32| {
                if n_items == 0 {
                    return Some("empty-library");
                } else {
                    return Some("library-page");
                }
            })
            .sync_create()
            .build();

        imp.shows_search_bar.connect_entry(&imp.search_entry.get());
    }

    pub fn new_show(&self, data: JikanData) {
        let show = ShowObject::new(data);

        self.shows().append(&show);
    }

    pub fn setup_factory(&self) {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(glib::clone!(
            move |_, list_item| {
                let show_card = RonajoShowCard::new();

                list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&show_card));
            }
        ));

        factory.connect_bind(move |_, listitem| {
            let show_object = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .item()
                .and_downcast::<ShowObject>()
                .expect("item must be ShowObject");

            let show_card = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .child()
                .and_downcast::<RonajoShowCard>()
                .expect("item must be ShowObject");

             show_card.bind(&show_object);
        });


        factory.connect_unbind(move |_, listitem| {
            let show_card = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .child()
                .and_downcast::<RonajoShowCard>()
                .expect("item must be ShowObject");

            show_card.unbind();
        });

        self.imp().show_view.set_factory(Some(&factory));

        self.imp().show_view.connect_activate(glib::clone!(
            #[weak(rename_to = view)]
            self.imp().navigation_view,
            move |grid_view, position| {
                let model = grid_view.model().expect("failed to get model");
                let selection_model = model.downcast_ref::<gtk::NoSelection>()
                    .expect("model must be of type NoSelection");
                let list_model = selection_model.model()
                    .expect("failed to get selection model");
                let item = list_model.item(position)
                    .expect("failed to get item");

                let show_object = item
                    .downcast_ref::<ShowObject>()
                    .expect("Object must be of type Episode Object");

                let page = RonajoShowPage::new();

                         let data = show_object
                            .imp()
                            .data
                            .borrow()
                            .clone()
                            .expect("failed to get data");

                        page.bind(&data);

                        view.push(&page);
            }));
    }

    pub fn library_shows(&self) -> gio::ListStore {
        self.imp()
            .library_shows
            .borrow()
            .clone()
            .expect("failed to get library shows")
    }

    pub fn setup_library_shows(&self) {
        let imp = self.imp();
        let model = gio::ListStore::new::<ShowObject>();
        let filter_library: bool = self.settings().get("filter-library");

        imp.library_shows.replace(Some(model));

        let filter_model = if filter_library {
            gtk::FilterListModel::new(Some(self.library_shows()), self.filter())
        } else {
            gtk::FilterListModel::new(Some(self.library_shows()), None::<gtk::CustomFilter>)
        };
        let selection_model = gtk::NoSelection::new(Some(filter_model.clone()));
        imp.library_view.set_model(Some(&selection_model));

        self.settings().connect_changed(
            Some("filter-library"),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[weak]
                filter_model,
                move |settings, _| {

                    let state: bool = settings.get("filter-library");
                    if state {
                        filter_model.set_filter(window.filter().as_ref());
                    } else {
                        filter_model.set_filter(None::<&gtk::CustomFilter>);
                    }

                }
            ),
        );

        self.settings().connect_changed(
            Some("filter"),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[weak]
                filter_model,
                move |settings, _| {
                    let filter_library: bool = settings.get("filter-library");
                    if filter_library {
                        filter_model.set_filter(window.filter().as_ref());
                    }
                }
            ),
        );

        for show in library_shows().expect("failed to get library shows") {
            self.new_library_show(show.0);
        }
    }

    pub fn setup_library_callbacks(&self) {
        // let imp = self.imp();
    }

    pub fn new_library_show(&self, data: JikanData) {
        let show = ShowObject::new(data);

        self.library_shows().append(&show);
    }

    pub fn setup_library_factory(&self) {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(
            move |_, list_item| {
                let show_card = RonajoShowCard::new();

                list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&show_card));
            }
        );

        factory.connect_bind(
            move |_, listitem| {
            let show_object = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .item()
                .and_downcast::<ShowObject>()
                .expect("item must be ShowObject");

            let show_card = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .child()
                .and_downcast::<RonajoShowCard>()
                .expect("item must be ShowObject");

            show_card.bind(&show_object);
        });

        factory.connect_unbind(move |_, listitem| {
            let show_card = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .child()
                .and_downcast::<RonajoShowCard>()
                .expect("item must be ShowObject");

            show_card.unbind();
        });

        self.imp().library_view.set_factory(Some(&factory));

        self.imp().library_view.connect_activate(glib::clone!(
            #[weak(rename_to = view)]
            self.imp().navigation_view,
            move |grid_view, position| {
                let model = grid_view.model().expect("failed to get model");
                let selection_model = model.downcast_ref::<gtk::NoSelection>()
                    .expect("model must be of type NoSelection");
                let list_model = selection_model.model()
                    .expect("failed to get selection model");
                let item = list_model.item(position)
                    .expect("failed to get item");

                let show_object = item
                    .downcast_ref::<ShowObject>()
                    .expect("Object must be of type Episode Object");

                let page = RonajoShowPage::new();

                         let data = show_object
                            .imp()
                            .data
                            .borrow()
                            .clone()
                            .expect("failed to get data");

                        let show = get_library_show(data.mal_id).expect("failed to get library show").1;
                        page.bind_show_data(&show);

                        view.push(&page);
            }));
    }

    fn filter(&self) -> Option<gtk::CustomFilter> {
        // Get filter_state from settings
        let filter_state: String = self.settings().get("filter");

        // Create custom filters
        let filter_nsfw = gtk::CustomFilter::new(|obj| {
            let show_object = obj
                .downcast_ref::<ShowObject>()
                .expect("The object needs to be of type `ShowObject`.");

            !(show_object.is_ecchi() || show_object.is_adult())
        });
        let filter_sfw_with_ecchi = gtk::CustomFilter::new(|obj| {
            let show_object = obj
                .downcast_ref::<ShowObject>()
                .expect("The object needs to be of type `ShowObject`.");

            !show_object.is_adult()
        });

        // Return the correct filter
        match filter_state.as_str() {
            "sfw" => Some(filter_nsfw),
            "nsfw" => None,
            "sfw-with-ecchi" => Some(filter_sfw_with_ecchi),
            "nsfw-with-ecchi" => None,
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Default, serde::Deserialize)]
pub struct RemoteVideoData {
    pub video_data: VideoData,
    device_data: PlayerData
}
#[derive(Clone, Default, serde::Deserialize)]
pub struct VideoData {
    pub allanime_id: String,
    pub title: String,
    pub episode_number: u32,
    pub translation: String,
    pub total_episodes: u32,
}
