use std::io::Error;

use super::OverpassClient;

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
