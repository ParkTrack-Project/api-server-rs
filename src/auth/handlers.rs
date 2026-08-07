use axum::Json;
use axum_valid::Valid;

use super::requests;
use super::responses;

pub async fn register(Valid(Json(body)): Valid<Json<requests::Register>>) {
    todo!()
}

pub async fn login(Valid(Json(body)): Valid<Json<requests::Login>>) {
    todo!()
}

pub async fn reset_password_request(Valid(Json(body)): Valid<Json<requests::PasswordReset>>) {
    todo!()
}

pub async fn reset_password_confirm(
    Valid(Json(body)): Valid<Json<requests::PasswordResetConfirm>>,
) {
    todo!()
}

pub async fn logout() {
    todo!()
}

pub async fn get_me() {
    todo!()
}
