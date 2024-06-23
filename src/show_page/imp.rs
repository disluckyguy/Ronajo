use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell;
use std::vec;
use crate::show_object::ShowObject;
use crate::episode_object::EpisodeObject;

#[derive(Debug, CompositeTemplate, Default)]
#[template(resource = "/io/github/ronajo/resources/ronajo-show-page.ui")]
pub struct RonajoShowPage {
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub add_to_lib_btn: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub status_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub description_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub expand_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub rating_row: TemplateChild<adw::SpinRow>,
    #[template_child]
    pub episode_view: TemplateChild<gtk::ListView>,
    pub episodes: cell::RefCell<Option<gio::ListStore>>,
    pub bindings: cell::RefCell<vec::Vec<glib::Binding>>,

}

#[glib::object_subclass]
impl ObjectSubclass for RonajoShowPage {
    const NAME: &'static str = "RonajoShowPage";
    type Type = super::RonajoShowPage;
    type ParentType = adw::NavigationPage;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
       obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for RonajoShowPage {
    fn constructed(&self) {
        let obj = self.obj();

        obj.setup_episodes();
        obj.setup_factory();
        obj.setup_bindings();

        obj.new_episode();
    }
}

impl WidgetImpl for RonajoShowPage {}

impl NavigationPageImpl for RonajoShowPage {}
