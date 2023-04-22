use std::{collections::HashMap, env};
use uuid::Uuid;

use color_eyre::eyre::{eyre, Result};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let api_token = env::var("api_token")?;
    let account_id = env::var("account_id")?;

    let client = reqwest::Client::new();

    let url = env::args()
        .nth(1)
        .ok_or_else(|| eyre!("missing argument"))?;
    let url = Url::parse(&url)?;
    let domain = format!(
        "{}://{}",
        url.scheme(),
        url.domain().ok_or_else(|| eyre!("missing domain"))?
    );

    let req_id = Uuid::new_v4();
    let create_id = Uuid::new_v4();

    let res = client
        .post("https://api.fastmail.com/jmap/api")
        .bearer_auth(api_token)
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core", "https://www.fastmail.com/dev/maskedemail"],
            "methodCalls": [
                [
                    "MaskedEmail/set",
                    {
                        "accountId": account_id,
                        "create": {
                            create_id.to_string(): {
                                "forDomain": domain,
                                "state": "enabled",
                            },
                        },
                    },
                    req_id.to_string(),
                ],
            ],
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?;

    let method_response = &res
        .method_responses
        .iter()
        .find(|x| x.2 == req_id.to_string())
        .ok_or_else(|| eyre!("missing method response"))?
        .1;
    let created_response = method_response
        .created
        .get(&create_id.to_string())
        .ok_or_else(|| eyre!("missing created response"))?;
    let email = &created_response.email;

    println!("{email}");

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "methodResponses")]
    method_responses: Vec<(String, MethodResponse, String)>,
}

#[derive(Debug, Deserialize)]
struct MethodResponse {
    created: HashMap<String, CreatedResponse>,
}

#[derive(Debug, Deserialize)]
struct CreatedResponse {
    #[allow(dead_code)]
    id: String,
    email: String,
}
