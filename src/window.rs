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
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gst::prelude::*;
use std::cell;
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

    pub fn play_video(&self, _action: &gio::SimpleAction, param: Option<&adw::glib::Variant>) {
        let view = self.imp().navigation_view.get();
        let parameter = param
            .expect("Could not get parameter.")
            .get::<String>()
            .expect("The variant needs to be of type `String`.");
        let video_page = RonajoVideoPage::new(parameter);



        view.push(&video_page);
    }

    pub fn play_remote_video(&self, _action: &gio::SimpleAction, param: Option<&adw::glib::Variant>) {
        let view = self.imp().navigation_view.get();
        let parameter = param
            .expect("Could not get parameter.")
            .get::<String>()
            .expect("The variant needs to be of type `String`.");

        let value: serde_json::Value = serde_json::from_str(&parameter).expect("failed to parse");
        let data: PlayerData = serde_json::from_value(value.get("data").unwrap().clone()).expect("failed to parse");
        let url: String = serde_json::from_value(value.get("url").unwrap().clone()).expect("failed to parse");

        if !data.use_key {
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
            #[strong]
            url,
            #[weak]
            view,
            #[weak]
            entry,
            move |_, response| {
                if response == "continue" {
                    let mut data = data.clone();
                    data.password = Some(entry.text().to_string());
                    let player_page = RonajoPlayerPage::new(&data, &url);
                    view.push(&player_page);
                }
            }));
            alert_dialog.present(Some(self));
        }

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
                    playbin
                        .seek_simple(
                            gst::SeekFlags::FLUSH,
                            duration.sub(gst::ClockTime::from_seconds(1)),
                        )
                        .expect("failed to seek");
                    return;
                }
                playbin
                    .seek_simple(gst::SeekFlags::FLUSH, seek_time)
                    .expect("failed to seek");
            };
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
                    playbin
                        .seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_seconds(0))
                        .expect("failed to seek");
                    return;
                }
                playbin
                    .seek_simple(gst::SeekFlags::FLUSH, seek_time.0)
                    .expect("failed to seek");
            };
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
        self.add_action_entries([change_config_action, play_video_action, play_remote_video_action]);

        let toggle_fullscreen_action = gio::ActionEntry::builder("toggle-fullscreen")
            .activate(move |window: &Self, _, _| window.toggle_fullscreen())
            .build();
        self.add_action_entries([toggle_fullscreen_action]);

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

        let selection_model = gtk::NoSelection::new(Some(self.shows()));
        imp.show_view.set_model(Some(&selection_model));
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
                #[strong]
                sender,
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

        let view = self.imp().navigation_view.get();

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

            show_card.imp().show_button.connect_clicked(glib::clone!(
                    #[weak]
                    view,
                    #[weak]
                    show_card,
                    move |_| {
                        let page = RonajoShowPage::new();

                        let data = show_card
                            .imp()
                            .data
                            .borrow()
                            .clone()
                            .expect("failed to get data");

                        page.bind(&data);

                        view.push(&page);
                    }
                ));

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

        imp.library_shows.replace(Some(model));

        let selection_model = gtk::NoSelection::new(Some(self.library_shows()));
        imp.library_view.set_model(Some(&selection_model));

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

        let view = self.imp().navigation_view.get();

        factory.connect_setup(
            move |_, list_item| {
                let show_card = RonajoShowCard::new();

                list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&show_card));
            }
        );

        factory.connect_bind(glib::clone!(
            #[weak]
            view,
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

            show_card.imp().show_button.connect_clicked(glib::clone!(
                    #[weak]
                    view,
                    #[weak]
                    show_card,
                    move |_| {
                        let page = RonajoShowPage::new();

                        let data = show_card
                            .imp()
                            .data
                            .borrow()
                            .clone()
                            .expect("failed to get data");

                        let show = get_library_show(data.mal_id).expect("failed to get library show").1;
                        page.bind_show_data(&show);

                        page.bind(&data);

                        view.push(&page);
                    }
                ));

            show_card.bind(&show_object);
        }));

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
    }
}

