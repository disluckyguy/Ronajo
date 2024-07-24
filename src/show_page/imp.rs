use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell;
use crate::core::show_data::ShowData;
use glib::Properties;
use std::cell::RefCell;
use std::cell::Cell;

#[derive(Debug, CompositeTemplate, Default, Properties)]
#[properties(wrapper_type = super::RonajoShowPage)]
#[template(file = "src/gtk/show-page.blp")]
pub struct RonajoShowPage {
    #[template_child]
    pub page_stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
    #[template_child]
    pub rating_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub studio_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub favourite_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub status_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub synopsis_title: TemplateChild<gtk::Label>,
    #[template_child]
    pub description_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub expand_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub notes_text: TemplateChild<gtk::TextView>,
    #[template_child]
    pub rating_row: TemplateChild<adw::SpinRow>,
    #[template_child]
    pub translation_row: TemplateChild<adw::ComboRow>,
    #[template_child]
    pub episode_view: TemplateChild<gtk::ListView>,

    #[property(get, set, nullable, construct)]
    allanime_id: RefCell<Option<String>>,
    pub data: RefCell<Option<ShowData>>,

    pub episodes: cell::RefCell<Option<gio::ListStore>>,
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
#[glib::derived_properties]
impl ObjectImpl for RonajoShowPage {
    fn constructed(&self) {
        let obj = self.obj();
        obj.setup_episodes();
        obj.setup_factory();
        obj.setup_bindings();
    }
}

impl WidgetImpl for RonajoShowPage {}

impl NavigationPageImpl for RonajoShowPage {}
