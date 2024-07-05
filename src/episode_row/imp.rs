use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::vec;
use std::cell;
// use glib::properties;

#[derive(CompositeTemplate, Default)]
// #[properties(wrapper_type = super::EpisodeRow)]
#[template(file = "src/resources/episode-row.blp")]
pub struct RonajoEpisodeRow {

    pub bindings: cell::RefCell<Vec<glib::Binding>>,

}

#[glib::object_subclass]
impl ObjectSubclass for RonajoEpisodeRow {
    const NAME: &'static str = "RonajoEpisodeRow";
    type Type = super::RonajoEpisodeRow;
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
impl ObjectImpl for RonajoEpisodeRow {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for RonajoEpisodeRow {}

impl ButtonImpl for RonajoEpisodeRow {}
