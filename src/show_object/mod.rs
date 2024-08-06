mod imp;

use crate::core::show_data::JikanData;
use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;

glib::wrapper! {
    pub struct ShowObject(ObjectSubclass<imp::ShowObject>);
}

impl ShowObject {
    pub fn new(data: JikanData) -> Self {
        let title = if data.title_english.is_some() {
            &data.title_english.clone().unwrap()
        } else {
            &data.title
        };

        let object: Self = Object::builder()
            .property("name", title)
            .property("image", &data.images.jpg.large_image_url)
            .build();

        object.imp().data.replace(Some(data));

        object
    }

    pub fn data(&self) -> JikanData{
        self
            .imp()
            .data
            .borrow()
            .clone()
            .expect("failed to get data")
    }

    pub fn is_adult(&self) -> bool {
        let rating_option = self.data().rating;
        if let None = rating_option {
            return false;
        }
        let rating = rating_option.unwrap();
        let rating_words: Vec<&str> = rating.split_whitespace().collect();

        if rating_words[0] == "Rx" {
            return true;
        }
        false
    }

    pub fn is_ecchi(&self) -> bool {
        let rating_option = self.data().rating;
        if let None = rating_option {
            return false;
        }
        let rating = rating_option.unwrap();
        let rating_words: Vec<&str> = rating.split_whitespace().collect();

        if rating_words[0] == "R+" {
            return true;
        }
        false
    }
}
