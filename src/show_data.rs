use std::vec;

#[derive(Default, Debug, Clone)]
pub struct ShowData {
    pub name: String,
    pub description: String,
    pub image: String,
    pub airing: bool,
    pub rating: f32,
    pub episodes: vec::Vec<EpisodeData>,
    pub in_library: bool,
}

#[derive(Default, Debug, Clone)]
pub struct EpisodeData {
    pub number: u32,
    pub stream_url: String,
}
