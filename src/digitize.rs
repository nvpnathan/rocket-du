use std::{fs::File, io::Read, path::PathBuf};

use reqwest::{header::AUTHORIZATION, Client};
use serde::{Deserialize, Serialize};

pub struct Digitize {
    base_url: String,
    project_id: String,
    bearer_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    #[serde(rename = "documentId")]
    document_id: String,
}

impl Digitize {
    pub fn new(base_url: &str, project_id: &str, bearer_token: &str) -> Digitize {
        Digitize {
            base_url: base_url.to_string(),
            project_id: project_id.to_string(),
            bearer_token: bearer_token.to_string(),
        }
    }

    pub async fn start(&self, document_path: &PathBuf) -> Option<String> {
        // Define the API endpoint for digitization
        let api_url = format!("{}/{}/digitization/start?api-version=1", self.base_url, self.project_id);

        // Read file bytes
        let mut file = match File::open(document_path) {
            Ok(file) => file,
            Err(e) => {
                println!("Error opening file: {}", e);
                return None;
            }
        };
        let mut file_content = Vec::new();
        if let Err(e) = file.read_to_end(&mut file_content) {
            println!("Error reading file: {}", e);
            return None;
        }

        // Get MIME type
        let mime_type = mime_guess::from_path(document_path).first_or_octet_stream().to_string();

        // Prepare request
        let client = Client::new();
        let response = client
            .post(&api_url)
            .header(AUTHORIZATION, format!("Bearer {}", self.bearer_token))
            .header(reqwest::header::ACCEPT, "text/plain")
            .body(file_content)
            .header(reqwest::header::CONTENT_TYPE, mime_type)
            .send()
            .await;

        // Process response
        match response {
            Ok(response) => match response.status() {
                reqwest::StatusCode::ACCEPTED => {
                    println!("Document successfully digitized!");
                    let response_data: Document = response.json().await.unwrap();
                    let document_id = response_data.document_id;
                    println!("Document ID: {}", document_id);
                    Some(document_id)
                }
                _ => {
                    println!("Error: {} - {}", response.status(), response.text().await.unwrap());
                    None
                }
            },
            Err(e) => {
                println!("An error occurred: {}", e);
                None
            }
        }
    }
}