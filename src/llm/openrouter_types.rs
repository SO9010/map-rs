use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct AnalysisRequestType {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Request {
    pub messages: Option<Vec<Message>>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub response_format: Option<ResponseFormat>,
    pub stop: Option<Stop>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub seed: Option<u64>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub repetition_penalty: Option<f32>,
    pub logit_bias: Option<HashMap<i32, f32>>,
    pub top_logprobs: Option<u32>,
    pub min_p: Option<f32>,
    pub top_a: Option<f32>,
    pub prediction: Option<Prediction>,
    pub transforms: Option<Vec<String>>,
    pub models: Option<Vec<String>>,
    pub route: Option<Route>,
    pub user: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResponseFormat {
    pub r#type: String, // "json_object"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Stop {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContentPart),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextContent {
    pub r#type: String, // "text"
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageContentPart {
    pub r#type: String, // "image_url"
    pub image_url: ImageUrl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    UserAssistantSystem {
        role: Role,
        content: MessageContent,
        name: Option<String>,
    },
    Tool {
        role: Role, // must be Tool
        content: String,
        tool_call_id: String,
        name: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: String, // "function"
    pub function: FunctionDescription,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDescription {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value, // JSON Schema
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    None,
    Auto,
    Function {
        r#type: String, // "function"
        function: ToolFunctionChoice,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunctionChoice {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prediction {
    pub r#type: String, // "content"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Fallback,
}

// pub fn Addget_data_from_string_osm
// add workspace info!
