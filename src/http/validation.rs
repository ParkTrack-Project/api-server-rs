use validator::ValidationError;

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    phonenumber::parse(None, phone)
        .map_err(|_| ValidationError::new("invalid_phone"))?
        .is_valid()
        .then_some(())
        .ok_or_else(|| ValidationError::new("invalid_phone"))
}
