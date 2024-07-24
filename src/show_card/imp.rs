use crate::core::show_data::JikanData;
use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell;
use std::vec;
// use glib::properties;

#[derive(CompositeTemplate, Default)]
#[template(file = "src/gtk/show-card.blp")]
pub struct RonajoShowCard {
    #[template_child]
    pub picture: TemplateChild<gtk::Picture>,
    #[template_child]
    pub show_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    pub bindings: cell::RefCell<vec::Vec<glib::Binding>>,
    pub data: cell::RefCell<Option<JikanData>>,
}

#[glib::object_subclass]
impl ObjectSubclass for RonajoShowCard {
    const NAME: &'static str = "RonajoShowCard";
    type Type = super::RonajoShowCard;
    type ParentType = gtk::Box;

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
    }
}

impl WidgetImpl for RonajoShowCard {}
impl BoxImpl for RonajoShowCard {}
