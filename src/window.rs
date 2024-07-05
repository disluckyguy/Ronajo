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
use crate::show_data;
use adw::subclass::prelude::*;
use adw::prelude::*;
use gst::prelude::*;
use adw::{gio, glib};
use std::cell;
use std::vec;
use crate::show_object::ShowObject;
use crate::show_card::RonajoShowCard;
use crate::show_page::RonajoShowPage;
use crate::video_page::RonajoVideoPage;
use std::fs;
use std::ops::{Add, Sub};


mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "src/resources/window.blp")]
    pub struct RonajoWindow {
        // Template widgets

        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub show_view: TemplateChild<gtk::ListView>,
        // #[template_child]
        // pub shows_scrollable: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub library_view: TemplateChild<gtk::ListView>,
        // #[template_child]
        // pub library_scrollable: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub empty_shows: TemplateChild<gtk::StackPage>,
        #[template_child]
        pub empty_library: TemplateChild<gtk::StackPage>,

        // #[template_child]
        // pub library_clamp: TemplateChild<adw::Clamp>,
        // #[template_child]
        // pub shows_clamp: TemplateChild<adw::Clamp>,

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

            obj.setup_show_page();
            // obj.setup_video_page();
            obj.setup_shows();
            obj.setup_callbacks();
            obj.setup_factory();

            obj.setup_library_shows();
            obj.setup_library_callbacks();
            obj.setup_library_factory();
            obj.setup_bindings();

            obj.new_show();
            // obj.new_library_show();

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

    pub fn setup_show_page(&self) {
            let imp = self.imp();

            let show_page = RonajoShowPage::new();

            imp.navigation_view.add(&show_page);
    }

    pub fn setup_video_page(&self) {
        let imp = self.imp();

        let video_page = RonajoVideoPage::new();

        imp.navigation_view.add(&video_page);
    }

    pub fn change_config_path(&self) {
        let file_dialog = gtk::FileDialog::new();
        let cancellable = gtk::gio::Cancellable::new();
        file_dialog.select_folder(Some(self), Some(&cancellable), |result| {
            if result.is_ok() {
                let settings = gio::Settings::new("io.github.ronajo");
                let file = result.expect("invalid result");
                let path = if settings.string("config-path") == "Home" {
                    home::home_dir().expect("failed to get home dir").into_os_string().into_string().expect("failed to convert to string")
                } else {
                    settings.string("config-path").to_string()
                };

                let new_path = file.path().expect("failed to get file path").into_os_string().into_string().expect("failed to convert to string");

                fs::rename(format!("{}/.ronajo", path), format!("{}/.ronajo", new_path)).expect("failed to move folder");

                settings.set_string("config-path", &new_path).expect("failed to set config path");
            }
        });
    }

    pub fn play_video(&self, _action: &gio::SimpleAction, param: Option<&adw::glib::Variant>) {
        let view = self.imp().navigation_view.get();
        let video_page = RonajoVideoPage::new();
        let parameter = param
                .expect("Could not get parameter.")
                .get::<String>()
                .expect("The variant needs to be of type `String`.");

        video_page.playbin().set_property("url", parameter);

        view.push(&video_page);


    }

    pub fn toggle_fullscreen(&self) {
        if !self.is_fullscreened() {
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
                let duration = playbin.query_duration::<gst::ClockTime>().expect("failed to get duration");
                if seek_time.seconds() > duration.seconds() {
                    playbin.seek_simple(gst::SeekFlags::FLUSH, duration.sub(gst::ClockTime::from_seconds(1))).expect("failed to seek");
                    return;
                }
                playbin.seek_simple(gst::SeekFlags::FLUSH, seek_time).expect("failed to seek");
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
                    playbin.seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_seconds(0)).expect("failed to seek");
                    return;
                }
                playbin.seek_simple(gst::SeekFlags::FLUSH, seek_time.0).expect("failed to seek");
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
        self.add_action_entries([change_config_action, play_video_action]);

        let toggle_fullscreen_action = gio::ActionEntry::builder("toggle-fullscreen")
            .activate(move |window: &Self, _, _| window.toggle_fullscreen())
            .build();
        self.add_action_entries([toggle_fullscreen_action]);

        let seek_forward_action = gio::ActionEntry::builder("seek-forward")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.seek_forward()))
            .build();

        let seek_backward_action = gio::ActionEntry::builder("seek-backward")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.seek_backward()))
            .build();

        let toggle_pause_action = gio::ActionEntry::builder("toggle-pause")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.toggle_pause()))
            .build();

        let toggle_autohide_action = gio::ActionEntry::builder("toggle-autohide")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.toggle_autohide()))
            .build();

        let toggle_loop_action = gio::ActionEntry::builder("toggle-loop")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.toggle_loop()))
            .build();

        let toggle_mute_action = gio::ActionEntry::builder("toggle-mute")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.toggle_mute()))
            .build();

        let raise_volume_action = gio::ActionEntry::builder("raise-volume")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.raise_volume()))
            .build();

        let lower_volume_action = gio::ActionEntry::builder("lower-volume")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.lower_volume()))
            .build();

        let raise_rate_action = gio::ActionEntry::builder("raise-rate")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.raise_rate()))
            .build();

        let lower_rate_action = gio::ActionEntry::builder("lower-rate")
            .activate(glib::clone!(@weak self as window => move |_: &gio::SimpleActionGroup, _, _| window.lower_rate()))
            .build();


        let video_actions = gio::SimpleActionGroup::new();
        video_actions.add_action_entries([seek_forward_action, seek_backward_action, toggle_loop_action, toggle_pause_action, toggle_autohide_action
        , toggle_mute_action, raise_rate_action, lower_rate_action, raise_volume_action, lower_volume_action]);

        self.insert_action_group("vid" ,Some(&video_actions));
    }

    pub fn shows(&self) -> gio::ListStore {
        self
            .imp()
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

        // let empty_shows = self.imp().empty_shows.get();
        // let shows_clamp = self.imp().shows_clamp.get();
        // let shows_scrollable = self.imp().shows_scrollable.get();
        // if self.shows().n_items() == 0 {
        //     shows_scrollable.set_child(Some(&empty_shows));
        // } else {
        //     shows_scrollable.set_child(Some(&shows_clamp));
        // }

    }

    pub fn setup_callbacks(&self) {
        // let imp = self.imp();


    }

    pub fn setup_bindings(&self) {

        // let shows_scrollable = self.imp().shows_scrollable.get();
        // let library_scrollable = self.imp().library_scrollable.get();
        // let shows_clamp = self.imp().shows_clamp.get();
        // let library_clamp = self.imp().library_clamp.get();
        // let empty_shows = self.imp().empty_shows.get();
        // let empty_library = self.imp().empty_library.get();

        // self.shows().connect_items_changed(glib::clone!(@weak shows_clamp, @weak empty_shows, @weak shows_scrollable => move |shows,_,_,_| {
        //     if shows.n_items() == 0 {
        //         shows_scrollable.set_child(Some(&empty_shows));
        //     } else {
        //         shows_scrollable.set_child(Some(&shows_clamp));
        //     }
        // }));

        // self.library_shows().connect_items_changed(glib::clone!(@weak library_clamp, @weak empty_library, @weak library_scrollable => move |shows,_,_,_| {
        //     if shows.n_items() == 0 {
        //         library_scrollable.set_child(Some(&empty_library));
        //     } else {
        //         library_scrollable.set_child(Some(&library_clamp));
        //     }
        // }));

    }

    pub fn new_show(&self) {

        let data = show_data::ShowData {
            name: "Cowboy Bebop".to_string(),
            description: "Crime is timeless. By the year 2071, humanity has expanded across the galaxy, filling the surface of other planets with settlements like those on Earth. These new societies are plagued by murder, drug use, and theft, and intergalactic outlaws are hunted by a growing number of tough bounty hunters. Spike Spiegel and Jet Black pursue criminals throughout space to make a humble living. Beneath his goofy and aloof demeanor, Spike is haunted by the weight of his violent past. Meanwhile, Jet manages his own troubled memories while taking care of Spike and the Bebop, their ship. The duo is joined by the beautiful con artist Faye Valentine, odd child Edward Wong Hau Pepelu Tivrusky IV, and Ein, a bioengineered Welsh Corgi. While developing bonds and working to catch a colorful cast of criminals, the Bebop crew's lives are disrupted by a menace from Spike's past. As a rival's maniacal plot continues to unravel, Spike must choose between life with his newfound family or revenge for his old wounds.".to_string(),
            image: "/home/mostafa/test.webp".to_string(),
            airing: false,
            rating: 3.2,
            episodes: vec::Vec::new(),
            in_library: false,
        };
        let show = ShowObject::new(data);


        self.shows().append(&show);
    }

    pub fn setup_factory(&self) {

        let factory = gtk::SignalListItemFactory::new();

        let view = self.imp().navigation_view.get();


        factory.connect_setup(glib::clone!(@weak view => move |_, list_item|{
            let show_card = RonajoShowCard::new();

            show_card.connect_clicked(glib::clone!( @weak view, @weak show_card => move |_| {
                let page = view.find_page("show-page")
                    .expect("failed to get page");
                let show_page = page
                    .downcast_ref::<RonajoShowPage>()
                    .expect("needs to be RonajoShowPage");

                let data = show_card.imp().data.borrow().clone();

                let show_object = ShowObject::new(data);

                show_page.bind(&show_object);

                view.push(show_page);



            }));
            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&show_card));
        }));

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
    }

    pub fn library_shows(&self) -> gio::ListStore {
        self
            .imp()
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

        // let empty_library = self.imp().empty_library.get();
        // let library_clamp = self.imp().library_clamp.get();
        // let library_scrollable = self.imp().library_scrollable.get();
        // if self.library_shows().n_items() == 0 {
        //     library_scrollable.set_child(Some(&empty_library));
        // } else {
        //     library_scrollable.set_child(Some(&library_clamp));
        // }

    }

    pub fn setup_library_callbacks(&self) {
        // let imp = self.imp();


    }

    pub fn new_library_show(&self) {

        let data = show_data::ShowData {
            name: "Cowboy Bebop".to_string(),
            description: "Crime is timeless. By the year 2071, humanity has expanded across the galaxy, filling the surface of other planets with settlements like those on Earth. These new societies are plagued by murder, drug use, and theft, and intergalactic outlaws are hunted by a growing number of tough bounty hunters. Spike Spiegel and Jet Black pursue criminals throughout space to make a humble living. Beneath his goofy and aloof demeanor, Spike is haunted by the weight of his violent past. Meanwhile, Jet manages his own troubled memories while taking care of Spike and the Bebop, their ship. The duo is joined by the beautiful con artist Faye Valentine, odd child Edward Wong Hau Pepelu Tivrusky IV, and Ein, a bioengineered Welsh Corgi. While developing bonds and working to catch a colorful cast of criminals, the Bebop crew's lives are disrupted by a menace from Spike's past. As a rival's maniacal plot continues to unravel, Spike must choose between life with his newfound family or revenge for his old wounds.".to_string(),
            image: "/home/mostafa/test.webp".to_string(),
            airing: false,
            rating: 3.2,
            episodes: vec::Vec::new(),
            in_library: true,
        };

        let show = ShowObject::new(data);


        self.library_shows().append(&show);
    }

    pub fn setup_library_factory(&self) {

        let factory = gtk::SignalListItemFactory::new();

        let view = self.imp().navigation_view.get();


        factory.connect_setup(glib::clone!(@weak view => move |_, list_item|{
            let show_card = RonajoShowCard::new();

            show_card.connect_clicked(glib::clone!( @weak view, @weak show_card => move |_| {
                let page = view.find_page("show-page")
                    .expect("failed to get page");
                let show_page = page
                    .downcast_ref::<RonajoShowPage>()
                    .expect("needs to be RonajoShowPage");

                let data = show_card.imp().data.borrow().clone();

                let show_object = ShowObject::new(data);

                show_page.bind(&show_object);

                view.push(show_page);



            }));
            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&show_card));
        }));

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

        self.imp().library_view.set_factory(Some(&factory));
    }
}



