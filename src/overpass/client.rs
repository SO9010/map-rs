use std::{io::Error, time::Duration};

use bevy_map_viewer::DistanceType;
use ureq::Agent;

use crate::workspace::{Selection, SelectionType};

use super::{OverpassClient, Settings};

impl OverpassClient {
    pub fn send_overpass_query_string(&self, query: String) -> Result<String, ureq::Error> {
        let mut status = 429;
        while status == 429 {
            if let Ok(mut response) = self.agent.post(&self.url).send(&query) {
                if response.status() == 200 {
                    return response.body_mut().read_to_string();
                } else if response.status() == 429 {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                } else {
                    status = 0;
                }
            }
        }
        Err(ureq::Error::BadUri(
            "Error sending/making request!".to_string(),
        ))
    }

    pub fn send_overpass_query(&self) -> Result<String, ureq::Error> {
        if let Ok(query) = self.build_overpass_query_string() {
            if query.is_empty() {
                return Err(ureq::Error::BadUri("Empty query".into()));
            }
            let mut status = 429;
            while status == 429 {
                if let Ok(mut response) = self.agent.post(&self.url).send(&query) {
                    if response.status() == 200 {
                        return response.body_mut().read_to_string();
                    } else if response.status() == 429 {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    } else {
                        status = 0;
                    }
                }
            }
            return Err(ureq::Error::BadUri(
                "Error sending/making request!".to_string(),
            ));
        }
        Err(ureq::Error::BadUri(
            "Error sending/making request!".to_string(),
        ))
    }

    /// This function builds an Overpass query string based on the provided bounds and settings.
    /// For its bounds it wants something like this:
    /// "poly:\"{} {} {} {} {} {} {} {}\"".to_string()
    pub fn build_overpass_query_string(&self) -> Result<String, Error> {
        let mut query = String::default();
        let opening = "[out:json];";
        let closing = "\nout body geom;";

        for (category, key) in self.settings.get_true_keys_with_category() {
            if key == "n/a" {
                continue;
            } else if key == "*" {
                query.push_str(&format!(
                    r#"
                        (
                        way["{}"]({});
                        node["{}"]({});
                        relation["{}"]({});
                        );
                        "#,
                    category.to_lowercase(),
                    self.bounds,
                    category.to_lowercase(),
                    self.bounds,
                    category.to_lowercase(),
                    self.bounds,
                ));
            } else {
                query.push_str(&format!(
                    r#"
                        (
                        way["{}"="{}"]({});
                        node["{}"="{}"]({});
                        relation["{}"="{}"]({});
                        );
                        "#,
                    category.to_lowercase(),
                    key.to_lowercase(),
                    self.bounds,
                    category.to_lowercase(),
                    key.to_lowercase(),
                    self.bounds,
                    category.to_lowercase(),
                    key.to_lowercase(),
                    self.bounds,
                ));
            }
        }

        if !query.is_empty() {
            query.insert_str(0, opening);
            query.push_str(closing);
            Ok(query)
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "No valid settings provided",
            ))
        }
    }
}

pub fn get_bounds(selection: Selection) -> String {
    if selection.points.is_some() {
        if selection.selection_type == SelectionType::POLYGON {
            let points_string = selection
                .points
                .unwrap()
                .iter()
                .map(|point| format!("{} {}", point.lat, point.long))
                .collect::<Vec<String>>()
                .join(" ");
            format!("poly:\"{points_string}\"")
        } else {
            String::default()
        }
    } else if selection.start.is_some() && selection.end.is_some() {
        match selection.selection_type {
            SelectionType::RECTANGLE => {
                let start = selection.start.unwrap();
                let end = selection.end.unwrap();
                format!(
                    "poly:\"{} {} {} {} {} {} {} {}\"",
                    start.lat,
                    start.long,
                    start.lat,
                    end.long,
                    end.lat,
                    end.long,
                    end.lat,
                    start.long
                )
            }
            SelectionType::CIRCLE => {
                let start = selection.start.unwrap();
                let end = selection.end.unwrap();
                let (mut dist, dist_type) = start.distance(&end);
                match dist_type {
                    DistanceType::Km => dist *= 1000.0,
                    DistanceType::M => {}
                    DistanceType::CM => dist /= 100.0,
                }
                format!("around:{}, {}, {}", dist, start.lat, start.long)
            }
            // TODO: Add support to polygon.
            SelectionType::POLYGON => {
                if let Some(points) = &selection.points {
                    let points_string = points
                        .iter()
                        .map(|point| format!("{} {}", point.lat, point.long))
                        .collect::<Vec<String>>()
                        .join(" ");
                    format!("poly:\"{points_string}\"")
                } else {
                    String::new() // Return an empty string if no points are provided
                }
            }

            _ => {
                return String::new();
            }
        }
    } else {
        return String::new();
    }
}

impl Default for OverpassClient {
    fn default() -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let agent: Agent = config.into();
        OverpassClient {
            agent,
            url: "https://overpass-api.de/api/interpreter".to_string(),
            bounds: String::new(),
            settings: Settings::default(),
        }
    }
}

impl OverpassClient {
    pub fn new(url: &str) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let agent: Agent = config.into();
        OverpassClient {
            agent,
            url: url.to_string(),
            bounds: String::new(),
            settings: Settings::default(),
        }
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }
}

pub fn build_overpass_query_string(bounds: String, settings: Settings) -> Result<String, Error> {
    let mut query = String::default();
    let opening = "[out:json];";
    let closing = "\nout body geom;";

    for (category, key) in settings.get_true_keys_with_category() {
        if key == "n/a" {
            continue;
        } else if key == "*" {
            query.push_str(&format!(
                r#"
                    (
                    way["{}"]({});
                    node["{}"]({});
                    relation["{}"]({});
                    );
                    "#,
                category.to_lowercase(),
                bounds,
                category.to_lowercase(),
                bounds,
                category.to_lowercase(),
                bounds,
            ));
        } else {
            query.push_str(&format!(
                r#"
                    (
                    way["{}"="{}"]({});
                    node["{}"="{}"]({});
                    relation["{}"="{}"]({});
                    );
                    "#,
                category.to_lowercase(),
                key.to_lowercase(),
                bounds,
                category.to_lowercase(),
                key.to_lowercase(),
                bounds,
                category.to_lowercase(),
                key.to_lowercase(),
                bounds,
            ));
        }
    }

    if !query.is_empty() {
        query.insert_str(0, opening);
        query.push_str(closing);
        Ok(query)
    } else {
        Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "No valid settings provided",
        ))
    }
}
