use failure::Error;

#[derive(Debug, Fail)]
pub enum AuthorizationError {
    #[fail(display = "Missing user_id")]
    Missing,
    #[fail(display = "Failed to parse user_id: {}, {}", raw, error)]
    Parse { raw: String, error: Error },
}
