use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell;
use std::vec;

#[derive(Debug, CompositeTemplate, Default)]
#[template(resource = "/io/github/ronajo/resources/ronajo-video-page.ui")]
pub struct RonajoVideoPage {
    #[template_child]
    pub video: TemplateChild<gtk::Video>,

}

#[glib::object_subclass]
impl ObjectSubclass for RonajoVideoPage {
    const NAME: &'static str = "RonajoVideoPage";
    type Type = super::RonajoVideoPage;
    type ParentType = adw::NavigationPage;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
       obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for RonajoVideoPage {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
    }
}

impl WidgetImpl for RonajoVideoPage {}

impl NavigationPageImpl for RonajoVideoPage {}
