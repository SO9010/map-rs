use serde_json::json;

use crate::llm::LlmResponse;

// https://openrouter.ai/docs/api-reference/overview
use super::{Message, OpenrouterClient};
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

// We should add a populiation decnity command
pub const LLM_PROMPT: &str = r#"
You are a geo-analysis assistant.

Your job is to answer user questions about geographic areas. Always follow this strict process:

1. Use an `rq:` command to request data DO NOT ADD ANY OTHER TEXT TO THIS MESSAGE.
2. Wait for JSON data to be returned.
3. Interpret the data and give a clear, concise answer.
4. Do NOT guess or assume anything without data.
5. When words like here are used assume it means workspace and data can be gotten through commands

--- Commands ---

Needs specific parameters:
- rq: nb {lat,lon} r{radius} → Nearby features
- rq: bb {lat1,lon1,lat2,lon2} → Features in bounding box
- rq: d {lat1,lon1} {lat2,lon2} → Distance between two points
- rq: n {lat,lon} → Nearest feature
- rq: ply {[lat,lon],[lat,lon],...} → Features inside polygon

Context from workspace (no parameters)

- rq: s → Get the selection information
- rq: i → General summary for an area
- rq: cnt → Count features
- rq: sm → Summarize attributes (e.g. population, area)

Id parameter
- rq: gt <id> → Get full details for a feature
- rq: t <id> → Get tag metadata

Points must be in {lat,lon} format. Distances like r500 mean 500 meters.

--- Examples ---

User: What's nearby at {51.5,-0.09}?

Assistant: rq: nb {51.5,-0.09} r500

{Wait wait for data}

{Analyse data or request more}

Assistant: There are 18 nearby features within 500 meters, including residential buildings, a park, and a school.

---

User: What's the population density at {51.5,-0.09}?

Assistant: rq: sm {51.5,-0.09} r500

{Wait wait for data}

{Analyse data or request more}

Assistant: The population density is 15 people per 100 m² (or 15,000 per km²).

---

User: What happens if I add 200 houses here?

Assistant: rq: sm {51.5,-0.09} r500

{Wait wait for data}

{Analyse data or request more}

Assistant: Currently, there are 500 households. Adding 200 would increase that by 40%. If average household size stays the same, population would increase from 1500 to ~2100.

---

Rules Recap:
- Only use `rq:` to request data DO NOT ADD ANY OTHER TEXT TO THIS MESSAGE.
- Don’t guess — answer only after data is returned.
- Be concise and directly answer the user's question.

"#;

// I really should make my own llm api using ureq!
impl OpenrouterClient {
    fn build_messages_json(&self, mess: &Vec<Message>) -> serde_json::Value {
        let mut messages = vec![json!({
            "role": "system",
            "content": self.prompt
        })];

        for message in mess {
            messages.push(json!({
                "role": message.role,
                "content": message.content
            }));
        }

        json!(messages)
    }

    pub fn send_openrouter_chat(
        &self,
        messages: &Vec<Message>,
    ) -> Result<LlmResponse, ureq::Error> {
        let body = json!({
            "temperature": 0.0,
            "model": "deepseek/deepseek-chat-v3-0324:free",
            "messages": self.build_messages_json(messages)
        });

        let mut status = 429;
        while status == 429 {
            let body_clone = body.clone(); // Clone the body for each request attempt
            if let Ok(mut response) = self
                .agent
                .post(&self.url)
                .header(
                    "Authorization",
                    format!("Bearer {}", self.token.as_ref().unwrap().to_string()),
                )
                .send_json(body_clone)
            {
                if response.status() == 200 {
                    let res: LlmResponse = response.body_mut().read_json()?;
                    return Ok(res);
                } else if response.status() == 429 {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                } else {
                    status = 0;
                }
            }
        }
        Err(ureq::Error::ConnectionFailed)
    }
}
