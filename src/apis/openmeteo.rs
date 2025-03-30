use bevy::text::cosmic_text::rustybuzz::Language;
use serde::Deserialize;

use crate::types::Coord;

// https://geocoding-api.open-meteo.com/v1/search?name=Cambridge&count=1&language=en&format=json

//------------------------------------------------------------------------------
// Functions
//------------------------------------------------------------------------------ 

/// This function sends a geocode request to the Open Meteo API.
/// If no langauge is provided, it defaults to English.
pub fn send_geocode_request(name: String, count: u64, language: Option<String>) -> Coord {
    let lang = language.unwrap_or("en".to_string());

    let mut status = 429;
    while status == 429 {
        if let Ok(response) = ureq::get(format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count={}&language={}&format=json", name, count, lang).as_str()).call() {
            if response.status() == 200 {
                serde_json::from_str(response.into_body().read_to_string().unwrap().as_str()).expect("JSON was not well-formatted")
            } else if response.status() == 429 {
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                status = 0;
            }
        }
    }
    Coord::default()
}


//------------------------------------------------------------------------------
// Types
//------------------------------------------------------------------------------ 


#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub results: Vec<Result>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    #[serde(rename = "feature_code")]
    pub feature_code: String,
    #[serde(rename = "country_code")]
    pub country_code: String,
    #[serde(rename = "admin1_id")]
    pub admin1_id: i64,
    #[serde(rename = "admin2_id")]
    pub admin2_id: i64,
    #[serde(rename = "admin3_id")]
    pub admin3_id: i64,
    #[serde(rename = "admin4_id")]
    pub admin4_id: i64,
    pub timezone: String,
    pub population: i64,
    pub postcodes: Vec<String>,
    #[serde(rename = "country_id")]
    pub country_id: i64,
    pub country: String,
    pub admin1: String,
    pub admin2: String,
    pub admin3: String,
    pub admin4: String,
}
