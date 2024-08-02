use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell::RefCell;

#[derive(CompositeTemplate, Default)]
#[template(file = "src/gtk/devices-dialog.blp")]
pub struct PasswordDialog {
    #[template_child]
    pub password_entry: TemplateChild<adw::PasswordEntryRow>,
}

#[glib::object_subclass]
impl ObjectSubclass for PasswordDialog {
    const NAME: &'static str = "PasswordDialog";
    type Type = super::PasswordDialog;
    type ParentType = adw::AlertDialog;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for PasswordDialog {
    fn constructed(&self) {
        self.parent_constructed();

    }
}

impl WidgetImpl for PasswordDialog {}

impl AdwDialogImpl for PasswordDialog {}

impl AdwAlertDialogImpl for PasswordDialog {}
