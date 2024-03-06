use rocket::data::ByteUnit;
use rocket::http::Status;
use rocket::response::Response;
use rocket::tokio::fs::File;
use rocket::{
    data::Data,
    error,
    launch, post, routes,
    serde::{Deserialize, Serialize},
};
use std::env;
use std::io::BufWriter;
use std::path::PathBuf;
use rocket::serde::json::json;

// Load environment variables
fn load_env_vars() -> (String, String, String, String, String) {
    (
        env::var("APP_ID").unwrap(),
        env::var("APP_SECRET").unwrap(),
        env::var("AUTH_URL").unwrap(),
        env::var("BASE_URL").unwrap(),
        env::var("PROJECT_ID").unwrap(),
    )
}

// Trait to define functionality for different client services (Digitize, Classify, etc.)
trait Client {
    fn digitize(&self, document_path: &str) -> Option<String>;
    fn classify_document(&self, document_id: &str, classification_type: &str) -> Option<String>;
}

// Mock implementations for clients (replace with actual implementations)
struct MockDigitize;

impl Client for MockDigitize {
    fn digitize(&self, _document_path: &str) -> Option<String> {
        Some("mock_document_id".to_string())
    }

    fn classify_document(&self, _document_id: &str, _classification_type: &str) -> Option<String> {
        Some("mock_document_type_id".to_string())
    }
}

// Struct to represent the API response
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    result: Option<String>,
    error: Option<String>,
}

// Function to process the uploaded document
async fn process_document(document_path: &str, client: &impl Client) -> Option<String> {
    let document_id = client.digitize(document_path)?;
    if document_id.is_empty() {
        return None;
    }

    let document_type_id = client.classify_document(&document_id, "ml-classification")?;
    if document_type_id.is_empty() {
        return None;
    }

    Some(format!(
        "Extracted data for document type: {}",
        document_type_id
    ))
}

// Function to handle file upload
#[post("/upload", data = "<file>")]
async fn upload(file: Data<'_>) -> Result<Status, Status> {
    let mut filename = PathBuf::from("uploaded_file");

    // Open a file for writing
    let uploaded_file = File::create(&filename).await.map_err(|err| {
        error!("Error creating file: {}", err);
        Status::InternalServerError
    })?;

    // Create a buffer for efficient writing
    let mut writer = BufWriter::new(uploaded_file);

    // Read from the Data stream and write it to the file
    file.stream_to(&mut writer, ByteUnit::max_value())
        .await
        .map_err(|err| {
            error!("Error writing to file: {}", err);
            Status::InternalServerError
        })?;

    // Process the uploaded file
    let result = process_document(filename.to_str().unwrap(), &(MockDigitize)).await;

    // Remove the temporary file
    if let Err(err) = std::fs::remove_file(&filename) {
        error!("Error removing temporary file: {}", err);
    }

    // Return the response based on the processing result
    let response = if let Some(extracted_data) = result {
        json!({ "result": extracted_data })
    } else {
        json!({ "error": "Document processing failed" })
    };

    Ok(Response::build()
           .status(if result.is_some() { Status::Ok } else { Status::InternalServerError })
        .status(Default::default())
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![upload])
}
