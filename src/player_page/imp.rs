use crate::core::player_data::PlayerData;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use glib::Properties;
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use std::cell::Cell;
use std::cell::RefCell;

#[derive(Debug, CompositeTemplate, Default, Properties)]
#[properties(wrapper_type = super::RonajoPlayerPage)]
#[template(file = "src/gtk/player-page.blp")]
pub struct RonajoPlayerPage {
    #[property(get, set, name="device-name" ,type = String, construct, member = name)]
    #[property(get, set, name="address" ,type = String, construct, member = address)]
    #[property(get, set, name="username" ,type = String, construct, member = username)]
    #[property(get, set, name="password" ,type = String, nullable, construct, member = password)]
    #[property(get, set, name="use-key" ,type = bool, construct, member = use_key)]
    pub data: RefCell<PlayerData>,
    #[property(get, set, construct, default = 100f64)]
    pub volume: Cell<f64>,
    #[property(get, set, construct, default = 1f64)]
    pub rate: Cell<f64>,
    #[property(get, set, construct, default = false)]
    pub paused: Cell<bool>,
    #[property(get, set, construct, default = false)]
    pub muted: Cell<bool>,
    #[property(get, set, construct)]
    pub position: Cell<f64>,
    #[property(get, set, construct)]
    pub url: RefCell<String>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub play_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub play_image: TemplateChild<gtk::Image>,
    #[template_child]
    pub seek_forward: TemplateChild<gtk::Button>,
    #[template_child]
    pub seek_backward: TemplateChild<gtk::Button>,
    #[template_child]
    pub video_scale: TemplateChild<gtk::Scale>,
    #[template_child]
    pub mute_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub volume_spin: TemplateChild<gtk::SpinButton>,
    #[template_child]
    pub rate_spin: TemplateChild<gtk::SpinButton>,
    #[template_child]
    pub position_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub duration_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub screenshot_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub quit_button: TemplateChild<gtk::Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for RonajoPlayerPage {
    const NAME: &'static str = "RonajoPlayerPage";
    type Type = super::RonajoPlayerPage;
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
impl ObjectImpl for RonajoPlayerPage {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        let (sender, receiver) = async_channel::bounded(1);
        let data = obj.data();
        let url = obj.url();
        gio::spawn_blocking(move || {
            sender.send_blocking(false).expect("thread must be open");
            data.start_session(&url).expect("failed to start session");
            sender.send_blocking(true).expect("thread must be open");
        });

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = stack)]
            self.stack,
            async move {
                while let Ok(show_player) = receiver.recv().await {
                    if show_player {
                        stack.set_visible_child_name("player");
                    } else {
                        stack.set_visible_child_name("spinner");
                    }
                }
            }
        ));
        obj.setup_callbacks();
        obj.setup_binds();
    }
}

impl WidgetImpl for RonajoPlayerPage {}

impl NavigationPageImpl for RonajoPlayerPage {}
