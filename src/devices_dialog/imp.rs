use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell::RefCell;

#[derive(CompositeTemplate, Default)]
#[template(file = "src/gtk/devices-dialog.blp")]
pub struct DevicesDialog {
    #[template_child]
    pub devices_toast_overlay: TemplateChild<adw::ToastOverlay>,
    #[template_child]
    pub devices_stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub devices_list: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub add_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub status_add: TemplateChild<gtk::Button>,
    #[template_child]
    pub add_device_toast_overlay: TemplateChild<adw::ToastOverlay>,
    #[template_child]
    pub navigation_view: TemplateChild<adw::NavigationView>,
    #[template_child]
    pub name_row: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub address_row: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub username_row: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub password_row: TemplateChild<adw::PasswordEntryRow>,
    #[template_child]
    pub key_auth_button: TemplateChild<gtk::CheckButton>,
    #[template_child]
    pub save_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub empty_name: TemplateChild<gtk::Label>,
    #[template_child]
    pub empty_address: TemplateChild<gtk::Label>,
    #[template_child]
    pub empty_username: TemplateChild<gtk::Label>,
    #[template_child]
    pub empty_password: TemplateChild<gtk::Label>,


    pub devices: RefCell<Option<gio::ListStore>>
}

#[glib::object_subclass]
impl ObjectSubclass for DevicesDialog {
    const NAME: &'static str = "DevicesDialog";
    type Type = super::DevicesDialog;
    type ParentType = adw::Dialog;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for DevicesDialog {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        obj.setup_devices();
        obj.setup_callbacks();
        obj.setup_binds();

    }
}

impl WidgetImpl for DevicesDialog {}

impl AdwDialogImpl for DevicesDialog {}
