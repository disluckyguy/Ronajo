use adw::glib;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell;
// use glib::properties;

#[derive(CompositeTemplate, Default)]
// #[properties(wrapper_type = super::EpisodeRow)]
#[template(file = "src/gtk/episode-row.blp")]
pub struct RonajoEpisodeRow {
    #[template_child]
    pub episode_label: TemplateChild<gtk::Label>,
    pub bindings: cell::RefCell<Vec<glib::Binding>>,
}

#[glib::object_subclass]
impl ObjectSubclass for RonajoEpisodeRow {
    const NAME: &'static str = "RonajoEpisodeRow";
    type Type = super::RonajoEpisodeRow;
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
impl ObjectImpl for RonajoEpisodeRow {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for RonajoEpisodeRow {}

impl BoxImpl for RonajoEpisodeRow {}
