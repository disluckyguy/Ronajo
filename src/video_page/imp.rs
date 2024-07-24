/* window.rs
 *
 * Copyright 2024 Mostafa
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use super::*;
use adw::{self, glib};
use glib::Properties;
use gst::prelude::*;
use gtk;
use std::cell::Cell;
use std::cell::RefCell;
use std::time::Instant;
#[derive(Debug, Default, gtk::CompositeTemplate, Properties)]
#[properties(wrapper_type = super::RonajoVideoPage)]
#[template(file = "src/gtk/video-page.blp")]
pub struct RonajoVideoPage {
    // Template widgets
    #[template_child]
    pub picture: TemplateChild<gtk::Picture>,
    #[template_child]
    pub header_revealer: TemplateChild<gtk::Revealer>,
    #[template_child]
    pub controls_revealer: TemplateChild<gtk::Revealer>,
    #[template_child]
    pub volume_slider_revealer: TemplateChild<gtk::Revealer>,
    #[template_child]
    pub rate_revealer: TemplateChild<gtk::Revealer>,
    #[template_child]
    pub rate_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub fullscreen_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub play_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub lock_ui_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub loop_button: TemplateChild<gtk::ToggleButton>,
    #[template_child]
    pub seek_forward: TemplateChild<gtk::Button>,
    #[template_child]
    pub seek_backward: TemplateChild<gtk::Button>,
    #[template_child]
    pub duration_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub position_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub video_slider: TemplateChild<gtk::Scale>,
    #[template_child]
    pub audio_slider: TemplateChild<gtk::Scale>,
    #[template_child]
    pub mute_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub menu_mute_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub volume_spin: TemplateChild<gtk::SpinButton>,
    #[template_child]
    pub rate_spin: TemplateChild<gtk::SpinButton>,

    //properties
    #[property(get, set, construct)]
    pub uri: RefCell<String>,
    #[property(get, set, default = 1f64, minimum = 0.25, maximum = 2f64, construct)]
    pub rate: Cell<f64>,
    #[property(get, set, default = false, construct)]
    pub paused: Cell<bool>,
    #[property(get, set, default = true, construct)]
    pub show_subtitles: Cell<bool>,
    #[property(get, set, default = false, construct)]
    pub loop_video: Cell<bool>,
    #[property(
        get,
        set,
        default = 100f64,
        minimum = 0f64,
        maximum = 100f64,
        construct
    )]
    pub volume: Cell<f64>,
    #[property(get, set, default = false, construct)]
    pub mute: Cell<bool>,
    #[property(get, set, default = true, construct)]
    pub autohide: Cell<bool>,

    //data
    pub playbin: RefCell<Option<gst::Element>>,
    pub sink: RefCell<Option<gst::Element>>,
    pub prev_xy: Cell<(f64, f64)>,
    pub prev_pos: RefCell<Option<gst::ClockTime>>,
    pub last_hidden: RefCell<Option<Instant>>,
    pub last_volume_revealed: RefCell<Option<Instant>>,
    pub last_rate_revealed: RefCell<Option<Instant>>,
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

#[glib::derived_properties]
impl ObjectImpl for RonajoVideoPage {
    fn constructed(&self) {
        let obj = self.obj();
        self.parent_constructed();
        obj.setup_stream();
        obj.setup_callbacks();
        obj.setup_binds();
    }
}
impl WidgetImpl for RonajoVideoPage {}
impl NavigationPageImpl for RonajoVideoPage {}
