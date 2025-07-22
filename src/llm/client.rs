use crate::llm::Request;

// https://openrouter.ai/docs/api-reference/overview
use super::OpenrouterClient;

impl OpenrouterClient {
    // We want to also parse in the get_data_from_string_osm but maybe we want to format it differently (we should try that first)
    // We also need to parse through the selection data like size and location!
    pub fn send_openrouter_chat_string(&self, request: Request) -> Result<String, ureq::Error> {
        // This will be a put request
        // want to parse throguh the request in the body as a json
        // Remember hearder needs bearer token
        // Would be good to find out how to do the streaing but first just await response.
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
}
