use rocket::{Request, Data, Outcome};
use rocket::http::{Status, ContentType};
use rocket::response::status::{Custom};
use rocket::data;
use serde::de::DeserializeOwned;
use std::io::Read;
use validator::{ValidationError, ValidationErrors, Validate};

const LIMIT: u64 = 256; // always use a limit to prevent DoS attacks

pub fn validate_json_request<T>(req: &Request, data: Data) -> data::Outcome<T, Custom<String>>
    where T: DeserializeOwned 
{
    // Ensure the content type is JSON before opening the data.
    if req.content_type() != Some(&ContentType::JSON) {
        return Outcome::Forward(data);
    }

    let mut body = String::new();
    if let Err(e) = data.open().take(LIMIT).read_to_string(&mut body) {
        return Outcome::Failure((Status::InternalServerError, Custom(Status::InternalServerError, format!("{:?}", e))));
    }

    let result: T = match serde_json::from_str(&body) {
        Ok(data) => data,
        Err(error) => {
            let status = match error.classify() {
                serde_json::error::Category::Eof => Status::BadRequest,
                serde_json::error::Category::Syntax => Status::BadRequest,
                serde_json::error::Category::Io => Status::InternalServerError,
                serde_json::error::Category::Data => Status::UnprocessableEntity
            };
            return Outcome::Failure((status, Custom(status, error.to_string())));
        }
    };

    Outcome::Success(result)
}

pub fn perform_custom_validation<T>(resource: T) -> data::Outcome<T, Custom<String>>
    where T: Validate 
{
    match resource.validate() {
        Ok(_) => Outcome::Success(resource),
        Err(e) => {
            let error_messages = handle_validation_errors(e);
            let error_message = format!("The following validation errors occurred: {}", error_messages.join(";"));
            Outcome::Failure((Status::BadRequest, Custom(Status::BadRequest, error_message)))
        }
    }
}

pub fn handle_validation_errors(validation_errors: ValidationErrors) -> Vec<String> {
    let mut messages = Vec::new();

    for (_, val) in validation_errors.field_errors().iter() {
        for validation_error in *val {
            let message = handle_field_validation_error(validation_error);
            messages.push(message);
        }
    }

    messages
}

pub fn handle_field_validation_error(validation_error: &ValidationError) -> String {
    match &validation_error.message {
        Some(message) => format!("'{}'",message.to_string()),
        None => String::from("")
    }
}