use gtk::gio;
use gtk::prelude::*;
use std::fs;
use super::show_data::*;
use std::error::Error;
use std::io::Write;
use super::player_data::PlayerData;

pub fn change_config_path(path: String) {
    let settings = gio::Settings::new("io.github.ronajo");
    let old_path = if settings.string("config-path") == "Home" {
        home::home_dir()
        .expect("failed to get home dir")
        .into_os_string()
        .into_string()
        .expect("failed to convert to string")
    } else {
        settings.string("config-path").to_string()
    };

    fs::rename(format!("{}/.ronajo", old_path), format!("{}/.ronajo", path))
        .expect("failed to move folder");
}

pub fn setup_config() {
    let settings = gio::Settings::new("io.github.ronajo");
    let path = if settings.string("config-path") == "Home" {
        home::home_dir()
            .expect("failed to get home dir")
            .into_os_string()
            .into_string()
            .expect("failed to convert to string")
        } else {
            settings.string("config-path").to_string()
        };
    let _ = fs::create_dir(format!("{}/.ronajo", path));
    let _ = fs::create_dir(format!("{}/.ronajo/ratings", path));
    let _ = fs::create_dir(format!("{}/.ronajo/notes", path));
    let _ = fs::create_dir(format!("{}/.ronajo/devices", path));
    let _ = fs::create_dir(format!("{}/.ronajo/library", path));
}

pub fn library_path() -> String {
    let settings = gio::Settings::new("io.github.ronajo");
    let path = if settings.string("config-path") == "Home" {
        home::home_dir()
            .expect("failed to get home dir")
            .into_os_string()
            .into_string()
            .expect("failed to convert to string")
        } else {
            settings.string("config-path").to_string()
        };
    format!("{}/.ronajo/library", path)
}

pub fn notes_path() -> String {
    let settings = gio::Settings::new("io.github.ronajo");
    let path = if settings.string("config-path") == "Home" {
        home::home_dir()
            .expect("failed to get home dir")
            .into_os_string()
            .into_string()
            .expect("failed to convert to string")
        } else {
            settings.string("config-path").to_string()
        };
    format!("{}/.ronajo/notes", path)
}

pub fn ratings_path() -> String {
    let settings = gio::Settings::new("io.github.ronajo");
    let path = if settings.string("config-path") == "Home" {
        home::home_dir()
            .expect("failed to get home dir")
            .into_os_string()
            .into_string()
            .expect("failed to convert to string")
        } else {
            settings.string("config-path").to_string()
        };
    format!("{}/.ronajo/ratings", path)
}

pub fn devices_path() -> String {
    let settings = gio::Settings::new("io.github.ronajo");
    let path = if settings.string("config-path") == "Home" {
        home::home_dir()
            .expect("failed to get home dir")
            .into_os_string()
            .into_string()
            .expect("failed to convert to string")
        } else {
            settings.string("config-path").to_string()
        };
    format!("{}/.ronajo/ratings", path)
}

pub fn library_shows() -> Result<Vec<(JikanData, ShowData)>, Box<dyn Error>> {
    let mut shows = Vec::new();

    for entry in fs::read_dir(library_path())? {
        let file = fs::read_to_string(entry?.path())?;
        let value: serde_json::Value = serde_json::from_str(&file)?;
        if let (Some(jikan_value), Some(show_value)) = (value.get("jikan"), value.get("show")) {
            let jikan_data = serde_json::from_value(jikan_value.clone())?;
            let show_data = serde_json::from_value(show_value.clone())?;
            shows.push((jikan_data, show_data));
        }
    }
    Ok(shows)
}

pub fn get_library_show(id: u32) -> Result<(JikanData, ShowData), Box<dyn Error>> {
    let file = fs::read_to_string(format!("{}/{}", library_path(), id))?;
    let file_value: serde_json::Value  = serde_json::from_str(&file)?;
    let jikan_value = file_value.get("jikan").expect("failed to get jikan value");
    let show_value = file_value.get("show").expect("failed to get jikan value");
    let show = (serde_json::from_value(jikan_value.clone())?, serde_json::from_value(show_value.clone())?);
    Ok(show)
}


pub fn add_to_library(data: &ShowData) -> Result<(), Box<dyn Error>> {
    let json = serde_json::json!({
        "show": data,
        "jikan": data.to_jikan_data()
    });
    let file = std::fs::File::create(format!("{}/{}", library_path(), data.mal_id))?;
    serde_json::to_writer_pretty(file, &json)?;
    Ok(())
}

pub fn remove_from_library(id: u32) -> Result<(), Box<dyn Error>>  {
    fs::remove_file(format!("{}/{}", library_path(), id))?;
    Ok(())
}

pub fn in_library(id: u32) -> bool {
    for entry in fs::read_dir(library_path()).expect("failed to read directory") {
        let entry_name = entry.expect("failed to get entry").file_name();
        let name = entry_name.to_str().expect("failed to convert to string");

        if &id.to_string() == name {
            return true;
        }

    }

    false
}

pub fn save_note(id: u32, text: &str) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(format!("{}/{}", notes_path(), id))?;
    file.write(text.as_bytes())?;
    Ok(())
}

pub fn remove_note(id: u32)  -> Result<(), Box<dyn Error>> {
    fs::remove_file(format!("{}/{}", notes_path(), id))?;
    Ok(())
}

pub fn get_note(id: u32) -> Option<String> {
    if let Ok(text) = fs::read_to_string(format!("{}/{}", notes_path(), id)) {
        return Some(text)
    }
    None
}

pub fn save_rating(id: u32, value: f64) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(format!("{}/{}", ratings_path(), id))?;
    file.write(value.to_string().as_bytes())?;
    Ok(())
}

pub fn remove_rating(id: u32)  -> Result<(), Box<dyn Error>> {
    fs::remove_file(format!("{}/{}", ratings_path(), id))?;
    Ok(())
}

pub fn get_rating(id: u32) -> Option<f64> {
    if let Ok(text) = fs::read_to_string(format!("{}/{}", ratings_path(), id)) {
        let rating: f64 = text.parse().expect("failed to parse");
        return Some(rating)
    }
    None
}

pub fn devices() -> Result<Vec<PlayerData>, Box<dyn Error>> {
    let mut devices = Vec::new();

    for entry in fs::read_dir(devices_path())? {
        let file = fs::read_to_string(entry?.path())?;
        let device: PlayerData = serde_json::from_str(&file)?;

        devices.push(device);

    }
    Ok(devices)
}

pub fn device_names() -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = Vec::new();

    for entry in fs::read_dir(devices_path())? {
        names.push(entry?.file_name().to_string_lossy().to_string());

    }
    Ok(names)
}

pub fn get_device(name: String) -> Result<PlayerData, Box<dyn Error>> {
    let file = fs::read_to_string(format!("{}/{}", devices_path(), name))?;
    let device: PlayerData  = serde_json::from_str(&file)?;
    Ok(device)
}


pub fn save_device(data: &PlayerData) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::create(format!("{}/{}", devices_path(), data.name))?;
    serde_json::to_writer_pretty(file, data)?;
    Ok(())
}

pub fn remove_device(name: &str) -> Result<(), Box<dyn Error>>  {
    fs::remove_file(format!("{}/{}", devices_path(), name))?;
    Ok(())
}

