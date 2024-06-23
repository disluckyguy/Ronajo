use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gtk::CompositeTemplate;
use gtk::TemplateChild;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/io/github/ronajo/resources/ronajo-preferences-dialog.ui")]
pub struct RonajoPreferencesDialog {
    #[template_child]
    change_config_row: TemplateChild<adw::ActionRow>,
    #[template_child]
    change_config_button: TemplateChild<gtk::Button>,

}

#[glib::object_subclass]
impl ObjectSubclass for RonajoPreferencesDialog {
    const NAME: &'static str = "RonajoPreferencesDialog";
    type Type = super::RonajoPreferencesDialog;
    type ParentType = adw::PreferencesDialog;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
       obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for RonajoPreferencesDialog {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        let settings = gio::Settings::new("io.github.ronajo");

        let config_row = obj.imp().change_config_row.get();

        settings.bind("config-path", &config_row, "subtitle")
            .build();
    }
}

impl WidgetImpl for RonajoPreferencesDialog {}

impl AdwDialogImpl for RonajoPreferencesDialog {}

impl PreferencesDialogImpl for RonajoPreferencesDialog {}
