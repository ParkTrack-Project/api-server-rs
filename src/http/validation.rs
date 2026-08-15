use validator::ValidationError;

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    phonenumber::parse(None, phone)
        .map_err(|_| ValidationError::new("invalid_phone"))?
        .is_valid()
        .then_some(())
        .ok_or_else(|| ValidationError::new("invalid_phone"))
}

pub fn validate_source(source: &str) -> Result<(), ValidationError> {
    if source.ends_with(".m3u8") { return Ok(()) }

    if source.starts_with("rtsp://") { return Ok(()) }

    Err(ValidationError::new("unsupported_format"))
}