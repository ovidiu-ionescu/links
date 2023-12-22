use std::time::{SystemTime, UNIX_EPOCH};

use hyper::{Body, Request};
use lib_hyper_organizator::{authentication::check_security::UserId, typedef::GenericError};

pub type Result<T> = std::result::Result<T, GenericError>;

/** Equivalent of System.currentTimeMillis */
pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn get_user_name(request: &Request<Body>) -> Result<&str> {
    let Some(user_id) = request.extensions().get::<UserId>() else {
        return Err(GenericError::from(
            "No user found in request, this should not happen",
        ));
    };
    let user = &user_id.0;
    Ok(user)
}
