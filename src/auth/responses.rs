use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PasswordResetRequest {
    ok: bool,
    reset_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetConfirm {
    ok: bool,
}

#[derive(Debug, Serialize)]
pub struct PartnerMembershipInfo {
    partner_id: u32,
    role: String,
    permissions: Vec<String>,
    read_scope: String,
    write_scope: String,
    delete_scope: String,
    is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthUserInfo {
    user_id: u32,
    email: String,
    full_name: Option<String>,
    global_role: String,
    permissions: Vec<String>,
    partner_memberships: Vec<PartnerMembershipInfo>,
}

#[derive(Debug, Serialize)]
pub struct Token {
    access_token: String,
    token_type: String,
    expires_in: u32,
    user: AuthUserInfo,
}

#[derive(Debug, Serialize)]
pub struct Me {
    user_id: u32,
    email: String,
    full_name: Option<String>,
    global_role: String,
    permissions: Vec<String>,
    partner_memberships: Vec<PartnerMembershipInfo>,
}
