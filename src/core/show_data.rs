use reqwest::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{error::Error, vec::Vec};
use url::form_urlencoded;

static API_REFERER: &str = "https://allanime.to";
static API_BASE: &str = "allanime.day";
static API: &str = "https://api.allanime.day/api";
static API_USERAGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0";

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ShowData {
    pub mal_id: u32,
    pub allanime_id: Option<String>,
    pub title: String,
    pub title_english: Option<String>,
    pub rating: String,
    pub status: String,
    pub dub_episodes: u32,
    pub sub_episodes: u32,
    pub genres: Vec<String>,
    pub studios: Vec<String>,
    pub image: String,
    pub synopsis: Option<String>,
    pub in_library: bool
}

impl ShowData {
    pub async fn from_jikan_data(data: JikanData) -> Self {
        let mut show_data = ShowData::default();
        show_data.title = String::from(&data.title);
        show_data.title_english = data.title_english;
        show_data.mal_id = data.mal_id;
        show_data.synopsis = data.synopsis;
        show_data.status = data.status;
        show_data.image = data.images.jpg.large_image_url;
        show_data.studios = data.studios.into_iter().map(|studio| studio.name).collect();
        show_data.genres = data.genres.into_iter().map(|genre| genre.name).collect();
        show_data.rating = data.rating;
        show_data.in_library = super::config::in_library(data.mal_id);
        let allanime_search = api_search(String::from(&data.title), "sub".to_string(), true).await.expect("failed to search");
        for show in &allanime_search {
                if show.name.trim().to_lowercase() == data.title.trim().to_lowercase() {
                    show_data.allanime_id = Some(String::from(&show.id));
                    show_data.sub_episodes = show.available_episodes.sub;
                    show_data.dub_episodes = show.available_episodes.dub;
                    return show_data;
                }
        }




        show_data
    }

    pub fn to_jikan_data(&self) -> JikanData {
        let mut jikan_data = JikanData::default();
        jikan_data.title = self.title.clone();
        jikan_data.title_english = self.title_english.clone();
        jikan_data.mal_id = self.mal_id;
        jikan_data.synopsis = self.synopsis.clone();
        jikan_data.status = self.status.clone();
        jikan_data.images.jpg.large_image_url = self.image.clone();
        jikan_data.studios = self.studios.clone().into_iter().map(|studio| StudioData {name: studio}).collect();
        jikan_data.genres = self.genres.clone().into_iter().map(|genre| GenreData {name: genre}).collect();
        jikan_data.rating = self.rating.clone();

        jikan_data
    }
}
#[derive(Deserialize, Debug, Default, Clone, Serialize)]
pub struct AllanimeData {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    #[serde(rename = "availableEpisodes")]
    available_episodes: AvailableEpisode,
    #[serde(rename = "__typename")]
    typename: String,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
pub struct JikanData {
    pub mal_id: u32,
    pub title: String,
    pub title_english: Option<String>,
    pub status: String,
    pub rating: String,
    pub synopsis: Option<String>,
    pub studios: Vec<StudioData>,
    pub images: ImageData,
    pub genres: Vec<GenreData>,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
pub struct StudioData {
    pub name: String,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
pub struct ImageData {
    pub jpg: JpgShowImage,
}
#[derive(Deserialize, Debug, Default, Clone, Serialize)]

pub struct JpgShowImage {
    pub image_url: String,
    pub large_image_url: String,
    pub small_image_url: String,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
pub struct GenreData {
    pub name: String,
}
#[derive(Deserialize, Debug, Default, Clone, Serialize)]
pub struct AvailableEpisode {
    pub sub: u32,
    pub dub: u32,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeData {
    pub source_urls: Vec<SourceUrl>,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUrl {
    pub source_url: String,
    pub priority: f32,
    pub source_name: String,
    pub r#type: String,
    pub class_name: String,
    pub streamer_id: String,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkData {
    pub link: String,
    pub resolution_str: String,
}

pub async fn api_search(
    name: String,
    translation: String,
    sfw: bool,
) -> Result<Vec<AllanimeData>, Box<dyn Error>> {
    // let mut easy = Easy::new();
    let query =
    "query(        $search: SearchInput        $limit: Int        $page: Int        $translationType: VaildTranslationTypeEnumType        $countryOrigin: VaildCountryOriginEnumType    ) {    shows(        search: $search        limit: $limit        page: $page        translationType: $translationType        countryOrigin: $countryOrigin    ) {        edges {            _id name availableEpisodes __typename       }    }}";
    let variables =  format!("{{\"search\":{{\"allowAdult\":{},\"allowUnknown\":false,\"query\":\"{}\"}},\"limit\":40,\"page\":1,\"translationType\":\"{}\",\"countryOrigin\":\"ALL\"}}", !sfw, name, translation);
    let query_encoded: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let variables_encoded: String = form_urlencoded::byte_serialize(variables.as_bytes()).collect();

    let mut headers: header::HeaderMap = header::HeaderMap::new();
    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
    let client = reqwest::Client::builder()
        .referer(false)
        .default_headers(headers)
        .user_agent(API_USERAGENT)
        .build()?;

    let get = client
        .get(&format!(
            "{}?query={}&variables={}",
            API, query_encoded, variables_encoded
        ))
        .send()
        .await?
        .text()
        .await?;

    let search_value: serde_json::Value = serde_json::from_str(get.trim())?;
    let data_value = search_value.get("data").expect("couldn't find edges value");
    let shows_value = data_value.get("shows").expect("couldn't find edges value");
    let edges_value = shows_value.get("edges").expect("couldn't find edges value");
    let search_data: Vec<AllanimeData> = serde_json::from_value(edges_value.clone())?;

    Ok(search_data)
}

pub async fn api_get_episode(
    id: String,
    translation: String,
    episode_number: u32,
) -> Result<String, Box<dyn Error>> {
    let query = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {    episode(        showId: $showId        translationType: $translationType        episodeString: $episodeString    ) {        episodeString sourceUrls    }}";
    let variables = format!(
        "{{\"showId\":\"{}\",\"translationType\":\"{}\",\"episodeString\":\"{}\"}}",
        id, translation, episode_number
    );
    let query_encoded: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let variables_encoded: String = form_urlencoded::byte_serialize(variables.as_bytes()).collect();

    let mut headers: header::HeaderMap = header::HeaderMap::new();
    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
    let client = reqwest::Client::builder()
        .referer(false)
        .default_headers(headers)
        .user_agent(API_USERAGENT)
        .build()?;

    let get = client
        .get(&format!(
            "{}?query={}&variables={}",
            API, query_encoded, variables_encoded
        ))
        .send()
        .await?
        .text()
        .await?;

    let request_value: serde_json::Value = serde_json::from_str(get.trim())?;
    let data_value = request_value.get("data").expect("couldn't find data value");
    let episode_value = data_value
        .get("episode")
        .expect("couldn't find episode value");
    let sources_value = episode_value
        .get("sourceUrls")
        .expect("couldn't find sources value");
    let episode_data: Vec<SourceUrl> = serde_json::from_value(sources_value.clone())?;
    for source in &episode_data {
        if source.source_name == "Sak" || source.source_name == "S-mp4" {
            if source.source_url.get(0..2).unwrap() == "--".to_string() {
                let mut id = substitute_id(source.source_url.split_at(2).1.to_string());

                if let Some(index) = id.find("clock") {
                    id.insert_str(index + 5, ".json");

                    let mut headers: header::HeaderMap = header::HeaderMap::new();
                    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
                    let client = reqwest::Client::builder()
                        .referer(false)
                        .default_headers(headers)
                        .user_agent(API_USERAGENT)
                        .build()?;

                    let get = client
                        .get(&format!("https://{}{}", API_BASE, id))
                        .send()
                        .await?
                        .text()
                        .await?;

                    let value: serde_json::Value = serde_json::from_str(&get)?;
                    let links_value = value.get("links").expect("value doesn't exist");
                    let links_data: Vec<LinkData> = serde_json::from_value(links_value.clone())?;

                    if let Some(data) = links_data.get(0) {
                        return Ok(data.link.to_string());
                    }
                }
            }
        }
    }

    for source in &episode_data {
        if source.source_name == "Luf-mp4" {
            if source.source_url.get(0..2).unwrap() == "--".to_string() {
                let mut id = substitute_id(source.source_url.split_at(2).1.to_string());

                if let Some(index) = id.find("clock") {
                    id.insert_str(index + 5, ".json");

                    let mut headers: header::HeaderMap = header::HeaderMap::new();
                    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
                    let client = reqwest::Client::builder()
                        .referer(false)
                        .default_headers(headers)
                        .user_agent(API_USERAGENT)
                        .build()?;

                    let get = client
                        .get(&format!("https://{}{}", API_BASE, id))
                        .send()
                        .await?
                        .text()
                        .await?;

                    let value: serde_json::Value = serde_json::from_str(&get)?;
                    let links_value = value.get("links").expect("value doesn't exist");
                    let links_data: Vec<LinkData> = serde_json::from_value(links_value.clone())?;
                    if let Some(data) = links_data.get(0) {
                        return Ok(data.link.to_string());
                    }
                }
            }
        }
    }

    Err("couldn't get episode link".into())
}

pub fn api_get_episode_blocking(
    id: String,
    translation: String,
    episode_number: u32,
) -> Result<String, Box<dyn Error>> {
    let query = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {    episode(        showId: $showId        translationType: $translationType        episodeString: $episodeString    ) {        episodeString sourceUrls    }}";
    let variables = format!(
        "{{\"showId\":\"{}\",\"translationType\":\"{}\",\"episodeString\":\"{}\"}}",
        id, translation, episode_number
    );
    let query_encoded: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let variables_encoded: String = form_urlencoded::byte_serialize(variables.as_bytes()).collect();

    let mut headers: header::HeaderMap = header::HeaderMap::new();
    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
    let client = reqwest::blocking::Client::builder()
        .referer(false)
        .default_headers(headers)
        .user_agent(API_USERAGENT)
        .build()?;

    let get = client
        .get(&format!(
            "{}?query={}&variables={}",
            API, query_encoded, variables_encoded
        ))
        .send()?
        .text()?;

    let request_value: serde_json::Value = serde_json::from_str(get.trim())?;
    let data_value = request_value.get("data").expect("couldn't find data value");
    let episode_value = data_value
        .get("episode")
        .expect("couldn't find episode value");
    let sources_value = episode_value
        .get("sourceUrls")
        .expect("couldn't find sources value");
    let episode_data: Vec<SourceUrl> = serde_json::from_value(sources_value.clone())?;
    for source in &episode_data {
        if source.source_name == "Sak" || source.source_name == "S-mp4" {
            if source.source_url.get(0..2).unwrap() == "--".to_string() {
                let mut id = substitute_id(source.source_url.split_at(2).1.to_string());

                if let Some(index) = id.find("clock") {
                    id.insert_str(index + 5, ".json");

                    let mut headers: header::HeaderMap = header::HeaderMap::new();
                    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
                    let client = reqwest::blocking::Client::builder()
                        .referer(false)
                        .default_headers(headers)
                        .user_agent(API_USERAGENT)
                        .build()?;

                    let get = client
                        .get(&format!("https://{}{}", API_BASE, id))
                        .send()?
                        .text()?;

                    let value: serde_json::Value = serde_json::from_str(&get)?;
                    let links_value = value.get("links").expect("value doesn't exist");
                    let links_data: Vec<LinkData> = serde_json::from_value(links_value.clone())?;

                    if let Some(data) = links_data.get(0) {
                        return Ok(data.link.to_string());
                    }
                }
            }
        }
    }

    for source in &episode_data {
        if source.source_name == "Luf-mp4" {
            if source.source_url.get(0..2).unwrap() == "--".to_string() {
                let mut id = substitute_id(source.source_url.split_at(2).1.to_string());

                if let Some(index) = id.find("clock") {
                    id.insert_str(index + 5, ".json");

                    let mut headers: header::HeaderMap = header::HeaderMap::new();
                    headers.insert("Referer", header::HeaderValue::from_static(API_REFERER));
                    let client = reqwest::blocking::Client::builder()
                        .referer(false)
                        .default_headers(headers)
                        .user_agent(API_USERAGENT)
                        .build()?;

                    let get = client
                        .get(&format!("https://{}{}", API_BASE, id))
                        .send()?
                        .text()?;

                    let value: serde_json::Value = serde_json::from_str(&get)?;
                    let links_value = value.get("links").expect("value doesn't exist");
                    let links_data: Vec<LinkData> = serde_json::from_value(links_value.clone())?;
                    if let Some(data) = links_data.get(0) {
                        return Ok(data.link.to_string());
                    }
                }
            }
        }
    }

    Err("couldn't get episode link".into())
}

pub fn substitute_id(input_id: String) -> String {
    let mut output_id = String::new();

    let rules = vec![
        ("01", '9'),
        ("08", '0'),
        ("05", '='),
        ("0a", '2'),
        ("0b", '3'),
        ("0c", '4'),
        ("07", '?'),
        ("00", '8'),
        ("5c", 'd'),
        ("0f", '7'),
        ("5e", 'f'),
        ("17", '/'),
        ("54", 'l'),
        ("09", '1'),
        ("48", 'p'),
        ("4f", 'w'),
        ("0e", '6'),
        ("5b", 'c'),
        ("5d", 'e'),
        ("0d", '5'),
        ("53", 'k'),
        ("1e", '&'),
        ("5a", 'b'),
        ("59", 'a'),
        ("4a", 'r'),
        ("4c", 't'),
        ("4e", 'v'),
        ("57", 'o'),
        ("51", 'i'),
    ];

    for i in 0..(input_id.len() / 2) {
        let start = i * 2;
        let end = i * 2 + 2;
        let sub_str = input_id.get(start..end).expect("failed to get string");

        for rule in &rules {
            if sub_str == rule.0 {
                output_id.push(rule.1);
            }
        }
    }

    output_id
}

pub async fn jikan_search(name: String) -> Result<Vec<JikanData>, Box<dyn Error>> {
    let client = Client::new();
    // Define query and variables
    // Make HTTP post request
    let name_encoded: String = form_urlencoded::byte_serialize(name.as_bytes()).collect();
    let resp = client
        .get(format!(
            "https://api.jikan.moe/v4/anime?q={}&limit=10&sfw&type=tv",
            &name_encoded
        ))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await;
    // Get json output
    let result: serde_json::Value = serde_json::from_str(&resp.unwrap())?;
    // println!("{}", result);
    let data_value = result.get("data").expect("failed to get data");
    let jikan_data: Vec<JikanData> = serde_json::from_value(data_value.clone())?;
    Ok(jikan_data)
}
