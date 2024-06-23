use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::vec;
use std::cell;
use crate::show_data::ShowData;
// use glib::properties;

#[derive(CompositeTemplate, Default)]
// #[properties(wrapper_type = super::ShowCard)]
#[template(resource = "/io/github/ronajo/resources/ronajo-show-card.ui")]
pub struct RonajoShowCard {
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub description_label: TemplateChild<gtk::Label>,
    pub bindings: cell::RefCell<vec::Vec<glib::Binding>>,
    pub data: cell::RefCell<ShowData>,

}

#[glib::object_subclass]
impl ObjectSubclass for RonajoShowCard {
    const NAME: &'static str = "RonajoShowCard";
    type Type = super::RonajoShowCard;
    type ParentType = gtk::Button;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
       obj.init_template();
    }
}

// Trait shared by all GObjects
// #[glib::derived_properties]
impl ObjectImpl for RonajoShowCard {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
    }
}

impl WidgetImpl for RonajoShowCard {}

impl ButtonImpl for RonajoShowCard {}

