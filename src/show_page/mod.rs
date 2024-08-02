mod imp;

use crate::core::show_data::*;
use crate::core::config::*;
use crate::episode_object::EpisodeObject;
use crate::episode_row::RonajoEpisodeRow;
use crate::runtime;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gdk, gio, glib};
use glib::Object;
use gtk::pango;

glib::wrapper! {
    pub struct RonajoShowPage(ObjectSubclass<imp::RonajoShowPage>)
    @extends adw::NavigationPage, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoShowPage {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn data(&self) -> ShowData {
        self
            .imp()
            .data
            .borrow()
            .clone()
            .expect("failed to get data")
    }


    pub fn bind_show_data(&self, data: &ShowData) {
        self.imp().data.replace(Some(data.clone()));

        let title_label = self.imp().title_label.get();
        let rating_label = self.imp().rating_label.get();
        let rating_row = self.imp().rating_row.get();
        let studio_label = self.imp().studio_label.get();
        let image = self.imp().image.get();
        let description_label = self.imp().description_label.get();
        let notes_text = self.imp().notes_text.get();
        let status_label = self.imp().status_label.get();
        let favourite_button = self.imp().favourite_button.get();

        self.set_allanime_id(data.allanime_id.clone());


        favourite_button.set_active(data.in_library);
        if let Some(synopsis) = data.synopsis.as_ref() {
            self.imp().synopsis_title.set_visible(true);
            description_label.set_visible(true);
            self.imp().expand_button.set_visible(true);

            description_label.set_label(synopsis);
        } else {
            self.imp().synopsis_title.set_visible(false);
            description_label.set_visible(false);
            self.imp().expand_button.set_visible(false);
        };

        rating_label.set_label(&data.rating);

        let rating_words = &data.rating.split_whitespace().collect::<Vec<&str>>();

        if rating_words[0] == "G" || rating_words[0] == "PG" {
            rating_label.add_css_class("success");
        } else if rating_words[0] == "PG-13" || rating_words[0] == "R" {
            rating_label.add_css_class("warning");
        } else if rating_words[0] == "PG-13" || rating_words[0] == "R" {
            rating_label.add_css_class("error");
        }

        if let Some(title) = data.title_english.as_ref() {
            self.set_title(title);
            title_label.set_label(title);
        } else {
            self.set_title(&data.title);
            title_label.set_label(&data.title);
        }

        favourite_button.connect_toggled(glib::clone!(
            #[strong(rename_to = data)]
            self.data(),
            move |button| {

            if button.is_active() {
                let _ = add_to_library(&data);
            } else {
                let _ = remove_from_library(data.mal_id);
            }
        }));


        studio_label.set_label(&data.studios.join(", "));

        status_label.set_label(&data.status);

        let (sender, receiver) = async_channel::bounded(1);
        runtime().spawn(glib::clone!(
            #[strong]
            sender,
            #[strong(rename_to = image)]
            data.image,
            async move {
                let response = reqwest::get(image)
                    .await
                    .expect("failed to get image")
                    .bytes()
                    .await
                    .expect("failed to convert to bytes");

                let bytes: Vec<u8> = response.to_vec();

                let gbytes = glib::Bytes::from(bytes.as_slice());
                sender.send(gbytes).await.expect("thread must be open");
            }
        ));
        glib::spawn_future_local(glib::clone!(
            #[weak]
            image,
            async move {
                while let Ok(data) = receiver.recv().await {
                    let texture = gdk::Texture::from_bytes(&data).expect("failed to make texture");
                    image.set_paintable(Some(&texture));
                }
            }
        ));
        if let Some(episodes) = self.imp().episodes.borrow().clone() {
            episodes.remove_all();
        }

        for genre in &data.genres {
            self.add_genre(genre);
        }

        for i in 1..data.sub_episodes + 1 {
            self.new_episode(i, "sub");

        }

        if let Some(text) = get_note(data.mal_id) {
            notes_text.buffer().set_text(&text);
        }

        if let Some(rating) = get_rating(data.mal_id) {
            rating_row.adjustment().set_value(rating);
        }

        let device_names = device_names().expect("failed to get device names");
        let devices_slice: Vec<_> = device_names.iter().map(String::as_str).collect();
        let devices_list = gtk::StringList::new(&devices_slice);

        self.imp().devices_row.set_model(Some(&devices_list));

        self.setup_callbacks();

        self.imp().expand_button.set_active(false);
    }

    pub fn setup_callbacks(&self) {
        let imp = self.imp();
        imp.notes_text.buffer().connect_changed(glib::clone!(
        #[strong(rename_to = id)]
        self.data().mal_id,
        move |buffer| {

            let bounds = buffer.bounds();
            let text = buffer.text(&bounds.0, &bounds.1, true);

            if text.is_empty() {
                let _ = remove_note(id);
            } else {
                let _ = save_note(id, &text);
            }
        }));

        imp.rating_row.adjustment().connect_value_notify(glib::clone!(
        #[strong(rename_to = id)]
        self.data().mal_id,
        move |adjustment| {

            let value = adjustment.value();

            if value == 0f64 {
                let _ = remove_rating(id);
            } else {
                let _ = save_rating(id, value);
            }
        }));

        imp.translation_row.connect_selected_item_notify(glib::clone!(
            #[weak(rename_to = page)]
            self,
            move |translation_row| {
                let item = translation_row.selected_item().expect("failed to get item");
                let string_object = item
                    .downcast_ref::<gtk::StringObject>()
                    .expect("object must be string object");
                let translation = string_object.string();

                match translation.as_str() {
                    "Sub" => {
                        page.episodes().remove_all();
                        for i in 1..page.data().sub_episodes + 1 {
                            page.new_episode(i, "sub");
                        }
                    }
                    "Dub" => {
                        page.episodes().remove_all();
                        for i in 1..page.data().dub_episodes + 1 {
                            page.new_episode(i, "dub");
                        }
                    }
                    _ => unreachable!()

                }
            }
        ));
    }


    pub fn bind(&self, data: &JikanData) {

        let (sender, receiver) = async_channel::bounded(1);

            runtime().spawn(glib::clone!(
                #[strong]
                sender,
                #[strong]
                data,
                async move {
                    sender.send(None)
                        .await
                        .expect("thread must be open");
                    let show_data = ShowData::from_jikan_data(data).await;
                    sender.send(Some(show_data))
                        .await
                        .expect("thread must be open")
                }
            ));

            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = page)]
                self,
                async move {
                    while let Ok(data) = receiver.recv().await {
                        if let Some(data) = data {
                            page.bind_show_data(&data);
                            page.imp().page_stack.set_visible_child_name("main");
                        } else {
                            page.imp().page_stack.set_visible_child_name("spinner");
                        }
                    }
                }
            ));
    }

    pub fn add_genre(&self, name: &str) {
        let card = gtk::FlowBoxChild::new();
        card.add_css_class("background");
        let label = gtk::Label::new(Some(name));
        label.add_css_class("accent");
        label.add_css_class("caption-heading");

        card.set_child(Some(&label));

        self.imp().genres.append(&card);

    }

    pub fn episodes(&self) -> gio::ListStore {
        self.imp()
            .episodes
            .borrow()
            .clone()
            .expect("failed to get episodes")
    }

    pub fn new_episode(&self, number: u32, translation: &str) {
        let episode = EpisodeObject::new(number, self.allanime_id(), translation);

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

        factory.connect_setup(move |_, list_item| {
            let episode_row = RonajoEpisodeRow::new();

            list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&episode_row));
        });

        factory.connect_bind(glib::clone!(
        #[weak(rename_to = remote_play_row)]
        self.imp().remote_play_row,
        #[weak(rename_to = device_row)]
        self.imp().devices_row,
        move |_, listitem| {

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

            episode_row.connect_clicked(glib::clone!(
            #[weak]
            episode_row,
            #[weak]
            episode_object,
            #[weak]
            remote_play_row,
            move |_| {
                let id = episode_object.allanime_id().expect("failed to get id");
                let translation = episode_object.translation();
                let number = episode_object.number();
                let (sender, receiver) = async_channel::bounded(1);
                runtime().spawn(glib::clone!(
            #[strong]
            sender,
            #[strong]
            id,
            #[strong]
            translation,
            #[strong]
            number,
            async move {

                let data = api_get_episode(id, translation, number).await.expect("failed to search");
                sender.send(data).await.expect("thread must be open");
            }
        ));
        glib::spawn_future_local(glib::clone!(
            #[weak]
            episode_row,
            #[strong(rename_to = remote_play)]
            remote_play_row.enables_expansion(),
            #[weak]
            device_row,
            async move {
                while let Ok(data) = receiver.recv().await {
                    if remote_play {
                        let selected_item = device_row.selected_item().expect("failed to get selected item");
                        let device_object = selected_item
                            .downcast_ref::<gtk::StringObject>()
                            .expect("Object must be String Object");
                        let device_name = device_object.string().to_string();

                        let device_data = get_device(device_name).expect("failed to get device");


                        let json = serde_json::json!({
                            "data": &device_data,
                            "url": &data
                        });

                        episode_row.activate_action("win.play-remote-video", Some(&json.to_string().to_variant()))
                            .expect("The action does not exist.");

                    } else {
                        episode_row.activate_action("win.play-video", Some(&data.to_variant()))
                            .expect("The action does not exist.");
                    }


                }
            }
        ));

            }));

        }));

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

        self.episodes().bind_property("n-items", &imp.episode_title.get(), "visible")
            .transform_to(|_, n_items: u32| {
                if n_items == 0 {
                    return Some(false);
                } else {
                    return Some(true);
                }
            })
            .sync_create()
            .build();

        imp.expand_button
            .bind_property("active", &imp.description_label.get(), "ellipsize")
            .transform_to(|_, active: bool| {
                let mut ellipsize_mode = pango::EllipsizeMode::End;

                if active {
                    ellipsize_mode = pango::EllipsizeMode::None;
                }

                Some(ellipsize_mode)
            })
            .sync_create()
            .build();

        imp.expand_button
            .bind_property("active", &imp.expand_button.get(), "label")
            .transform_to(|_, active: bool| {
                let icon_name = if active {
                    String::from("Show Less")
                } else {
                    String::from("Show More")
                };

                Some(icon_name)
            })
            .sync_create()
            .build();

        imp.favourite_button
            .bind_property("active", &imp.favourite_button.get(), "icon-name")
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

