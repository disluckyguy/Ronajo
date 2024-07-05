mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Object;
use std::cell;
use std::vec;
use gtk::pango;
use crate::episode_row::RonajoEpisodeRow;
use crate::show_object::ShowObject;
use crate::episode_object::EpisodeObject;
use crate::show_data::EpisodeData;

glib::wrapper! {
    pub struct RonajoShowPage(ObjectSubclass<imp::RonajoShowPage>)
    @extends adw::NavigationPage, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoShowPage {
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    pub fn bind(&self, object: &ShowObject) {
        self.unbind();
        let title_label = self.imp().title_label.get();
        let image = self.imp().image.get();
        let description_label = self.imp().description_label.get();
        let status_label = self.imp().status_label.get();
        let rating_row = self.imp().rating_row.get();
        let library_button = self.imp().add_to_lib_btn.get();

        let mut bindings = self.imp().bindings.borrow_mut();

        let title_binding = object.bind_property("name", &title_label, "label")
            .sync_create()
            .build();

        bindings.push(title_binding);

        let image_binding = object.bind_property("image", &image, "file")
            .sync_create()
            .build();

        bindings.push(image_binding);

        let description_binding = object.bind_property("description", &description_label, "label")
            .sync_create()
            .build();

        bindings.push(description_binding);

        let status_binding = object.bind_property("airing", &status_label, "label")
            .sync_create()
            .transform_to(|_, airing| {
                let label = if airing {
                    "Airing".to_string()
                } else {
                    "Finsihed Airing".to_string()
                };
                Some(label.to_value())
            })
            .build();

        bindings.push(status_binding);


        let rating_binding = object.bind_property("rating", &rating_row, "value")
            .sync_create()
            .build();

        bindings.push(rating_binding);

        let in_library_binding = object.bind_property("in-library", &library_button, "active")
            .bidirectional()
            .sync_create()
            .build();

        bindings.push(in_library_binding);

        self.imp().expand_button.set_active(false);
    }

    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }

    pub fn episodes(&self) -> gio::ListStore {
        self
            .imp()
            .episodes
            .borrow()
            .clone()
            .expect("failed to get episodes")
    }

    pub fn new_episode(&self) {
        let data = EpisodeData {
            number: 0,
            stream_url: String::new(),
        };
        let episode = EpisodeObject::new(data);




        self.episodes().append(&episode);
    }

    pub fn setup_episodes(&self) {
        let imp = self.imp();
        let model = gio::ListStore::new::<EpisodeObject>();

        imp.episodes.replace(Some(model));

        let selection_model = gtk::NoSelection::new(Some(self.episodes()));
        imp.episode_view.set_model(Some(&selection_model));
    }

    pub fn setup_factory(&self) {
        let factory = gtk::SignalListItemFactory::new();


        factory.connect_setup(move |_, list_item|{
            let episode_row = RonajoEpisodeRow::new();

            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&episode_row));
        });

        factory.connect_bind(move |_, listitem| {

            let episode_object = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .item()
                .and_downcast::<EpisodeObject>()
                .expect("item must be Episodebject");

            let episode_row = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .child()
                .and_downcast::<RonajoEpisodeRow>()
                .expect("item must be EpisodeRow");

            episode_row.bind(&episode_object);

            episode_row.connect_clicked(glib::clone!(@weak episode_row => move |_| {
                let parameter = String::from("http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4");
                episode_row.activate_action("win.play-video", Some(&parameter.to_variant()))
                            .expect("The action does not exist.");
            }));

        });

        factory.connect_unbind(move |_, listitem| {

            let episode_row = listitem
                .downcast_ref::<gtk::ListItem>()
                .expect("item must be ListItem")
                .child()
                .and_downcast::<RonajoEpisodeRow>()
                .expect("item must be EpisodeRow");

            episode_row.unbind();

        });

        self.imp().episode_view.set_factory(Some(&factory));
    }

    pub fn setup_bindings(&self) {

        let imp = self.imp();

        imp.expand_button.bind_property("active",&imp.description_label.get(), "ellipsize")
            .transform_to(|_, active: bool| {
                let mut ellipsize_mode = pango::EllipsizeMode::End;

                if active {
                    ellipsize_mode = pango::EllipsizeMode::None;
                }

                Some(ellipsize_mode)
            })
            .sync_create()
            .build();

        imp.expand_button.bind_property("active",&imp.description_label.get(), "ellipsize")
            .transform_to(|_, active: bool| {
                let mut ellipsize_mode = pango::EllipsizeMode::End;

                if active {
                    ellipsize_mode = pango::EllipsizeMode::None;
                }

                Some(ellipsize_mode)
            })
            .sync_create()
            .build();

        imp.expand_button.bind_property("active",&imp.expand_button.get(), "icon-name")
            .transform_to(|_, active: bool| {
                let icon_name = if active {
                    String::from("up-symbolic")
                } else {
                    String::from("down-symbolic")
                };

                Some(icon_name)
            })
            .sync_create()
            .build();

        imp.add_to_lib_btn.bind_property("active",&imp.add_to_lib_btn.get(), "icon-name")
            .transform_to(|_, active: bool| {
                let icon_name = if active {
                    String::from("heart-filled-symbolic")
                } else {
                    String::from("heart-outline-thick-symbolic")
                };

                Some(icon_name.to_value())
            })
            .sync_create()
            .build();

    }
}
