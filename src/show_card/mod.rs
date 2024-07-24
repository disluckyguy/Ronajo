mod imp;

use crate::core::show_data::JikanData;
use crate::runtime;
use crate::show_object::ShowObject;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gdk, gio, glib};
use glib::Object;
use std::io::Read;

glib::wrapper! {
    pub struct RonajoShowCard(ObjectSubclass<imp::RonajoShowCard>)
    @extends gtk::Box, gtk::Widget, glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RonajoShowCard {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn data(&self) -> Option<JikanData> {
        self.imp().data.borrow().clone()
    }

    pub fn bind(&self, object: &ShowObject) {
        let title_label = self.imp().title_label.get();
        let picture = self.imp().picture.get();
        let mut bindings = self.imp().bindings.borrow_mut();
        let object_data = object
            .imp()
            .data
            .borrow()
            .clone()
            .expect("failed to get data");

        self.imp().data.replace(Some(object_data));

        let title_binding = object
            .bind_property("name", &title_label, "label")
            .sync_create()
            .build();

        bindings.push(title_binding);

        let (sender, receiver) = async_channel::bounded(1);
        runtime().spawn(glib::clone!(
            #[strong]
            sender,
            #[strong(rename_to = image)]
            object.image(),
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
            picture,
            async move {
                while let Ok(data) = receiver.recv().await {
                    let texture = gdk::Texture::from_bytes(&data).expect("failed to make texture");
                    picture.set_paintable(Some(&texture));
                }
            }
        ));
    }

    pub fn unbind(self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}
