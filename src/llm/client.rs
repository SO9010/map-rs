use crate::llm::Request;

// https://openrouter.ai/docs/api-reference/overview
use super::OpenrouterClient;
// ============================================================================
// LLM Request Command Specification
// ============================================================================
//
// Use "rq:" for data requests. Format:
//     rq: <cmd> <params>
//
// Commands:
//   nb   : Nearby features      ex: rq: nb {51.5,-0.09} r500
//   cnt  : Count features       ex: rq: cnt {51.5,-0.09} r500
//   sm   : Summarize features   ex: rq: sm {51.5,-0.09} r500
//   gt   : Feature details      ex: rq: gt 123456
//   t    : Feature tags         ex: rq: t 123456
//   bb   : Features in bbox     ex: rq: bb {51.4,-0.1,51.6,-0.08}
//   d    : Distance             ex: rq: d {51.5,-0.09} {51.6,-0.10}
//   n    : Nearest feature      ex: rq: n {51.5,-0.09}
//   ply  : Features in polygon  ex: rq: ply {[51.5,-0.1],[51.6,-0.1],[51.6,-0.09]}
//   i    : General info/stats   ex: rq: i {51.5,-0.09} r500
//
// Rules:
// - Points as {lat,lon}, radius as r<meters> (e.g., r200).
// - bbox: {minLat,minLon,maxLat,maxLon}.
// - polygons: {[lat,lon],...}.
// - Always prefix data requests with "rq:".
//
// Example:
//   LLM -> rq: nb {51.5,-0.09} r200
//   App -> Returns JSON of features.
//
// ============================================================================

pub const LLM_PROMPT: &str = r#"
You are a geo-analysis assistant.

Answer user queries (e.g., "What is the population density here, and what if I build 200 more houses?").
If data is required, use an rq: command.

Commands:
// nb   : Nearby features.      ex: rq: nb {51.5,-0.09} r500
// cnt  : Count features.        ex: rq: cnt {51.5,-0.09} r500
// sm   : Summarize features.    ex: rq: sm {51.5,-0.09} r500
// gt   : Feature details by ID. ex: rq: gt 123456
// t    : Tags for feature.      ex: rq: t 123456
// bb   : Features in bbox.      ex: rq: bb {51.4,-0.1,51.6,-0.08}
// d    : Distance between two.  ex: rq: d {51.5,-0.09} {51.6,-0.10}
// n    : Nearest feature.       ex: rq: n {51.5,-0.09}
// ply  : Features in polygon.   ex: rq: ply {[51.5,-0.1],[51.6,-0.1],[51.6,-0.09]}
// i    : General info/stats.    ex: rq: i {51.5,-0.09} r500

Rules:
- Only use rq: commands to request data.
- Interpret returned JSON and respond concisely.

Example:
User: What is the population density here?
Assistant:
rq: sm {51.5,-0.09} r500
"#;

impl OpenrouterClient {
    // We want to also parse in the get_data_from_string_osm but maybe we want to format it differently (we should try that first)
    // We also need to parse through the selection data like size and location!
    /*
    pub fn send_openrouter_chat_string(&self, request: Request) -> Result<String, ureq::Error> {
        // This will be a put request
        // want to parse throguh the request in the body as a json
        // Remember hearder needs bearer token
        // Would be good to find out how to do the streaing but first just await response.
        let mut status = 429;
        while status == 429 {
            if let Ok(mut response) = self.agent.post(&self.url).send(&request) {
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
    */
}
