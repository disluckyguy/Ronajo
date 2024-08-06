use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell::OnceCell;

#[derive(CompositeTemplate, Default)]
#[template(file = "src/gtk/preferences-dialog.blp")]
pub struct RonajoPreferencesDialog {
    #[template_child]
    pub change_config_row: TemplateChild<adw::ActionRow>,
    #[template_child]
    pub change_config_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub enable_nsfw: TemplateChild<adw::SwitchRow>,
    #[template_child]
    pub enable_ecchi: TemplateChild<adw::SwitchRow>,
    #[template_child]
    pub filter_library: TemplateChild<adw::SwitchRow>,
    #[template_child]
    pub player: TemplateChild<adw::ComboRow>,
    #[template_child]
    pub translation: TemplateChild<adw::ComboRow>,
    pub settings: OnceCell<gio::Settings>
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

        obj.setup_settings();
        obj.setup_callbacks();

        let config_row = obj.imp().change_config_row.get();

        obj.settings()
            .bind("config-path", &config_row, "subtitle")
            .build();

        obj.settings()
            .bind("filter-library", &self.filter_library.get(), "active")
            .build();
    }
}

impl WidgetImpl for RonajoPreferencesDialog {}

impl AdwDialogImpl for RonajoPreferencesDialog {}

impl PreferencesDialogImpl for RonajoPreferencesDialog {}
