use crate::core::player_data::PlayerData;
use crate::core::show_data::*;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use crate::runtime;
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
    #[property(get, set, construct)]
    pub allanime_id: RefCell<String>,
    #[property(get, set, construct)]
    pub title: RefCell<String>,
    #[property(get, set, construct)]
    pub episode_number: Cell<u32>,
    #[property(get, set, construct)]
    pub total_episodes: Cell<u32>,
    #[property(get, set, construct)]
    pub translation: RefCell<String>,
    #[property(
        get,
        set,
        default = 100f64,
        minimum = 0f64,
        maximum = 100f64,
        construct
    )]
    pub volume: Cell<f64>,
    #[property(get, set, default = 1f64, minimum = 0.25, maximum = 2f64, construct)]
    pub rate: Cell<f64>,
    #[property(get, set, construct, default = false)]
    pub paused: Cell<bool>,
    #[property(get, set, construct, default = false)]
    pub muted: Cell<bool>,
    #[property(get, set, construct)]
    pub position: Cell<f64>,
    #[property(get, set, construct)]
    pub duration: Cell<f64>,
    #[property(get, set, construct)]
    pub player: RefCell<String>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub episode_label: TemplateChild<gtk::Label>,
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
        let player = obj.player();
        let allanime_id = obj.allanime_id();
        let translation = obj.translation();
        let episode_number = obj.episode_number();
        runtime().spawn(
        async move  {
            sender
                .send(false)
                .await
                .expect("thread must be open");
            let uri = api_get_episode(allanime_id, translation, episode_number).await.expect("failed to get episode");
            data.start_session(&uri, &player).expect("failed to start session");
            sender
                .send(true)
                .await
                .expect("thread must be open");
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
