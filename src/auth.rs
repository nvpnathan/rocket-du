use reqwest::Client;

pub struct Authentication {
    client_id: String,
    client_secret: String,
    token_url: String,
}

impl Authentication {
    pub fn new(client_id: &str, client_secret: &str, token_url: &str) -> Authentication {
        Authentication {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            token_url: token_url.to_string(),
        }
    }

    pub async fn get_bearer_token(&self) -> Option<String> {
        let client = Client::new();
        let data = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("grant_type", &"client_credentials".to_string()),
            (
                "scope",
                &"Du.DocumentManager.Document Du.Classification.Api Du.Digitization.Api Du.Extraction.Api Du.Validation.Api"
                    .to_string(),
            ),
        ];

        return match client.post(&self.token_url).form(&data).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(token_data) => {
                            if let Some(access_token) = token_data.get("access_token") {
                                println!("Authenticated!\n");
                                Some(access_token.as_str().unwrap().to_string())
                            } else {
                                println!("Error: No access token received");
                                None
                            }
                        }
                        Err(e) => {
                            println!("Error parsing token data: {}", e);
                            None
                        }
                    }
                } else {
                    println!("Error: HTTP request failed with status code {}", response.status());
                    None
                }
            }
            Err(e) => {
                println!("Error fetching token: {}", e);
                None
            }
        };
    }
}