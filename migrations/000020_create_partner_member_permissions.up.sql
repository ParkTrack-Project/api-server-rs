CREATE TABLE IF NOT EXISTS partner_member_permissions (
    partner_member_permission_id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    partner_id INTEGER NOT NULL REFERENCES partners(partner_id) ON DELETE CASCADE,
    permission VARCHAR(100) NOT NULL,
    granted_at TIMESTAMP DEFAULT NOW(),
    granted_by INTEGER REFERENCES users(user_id) ON DELETE SET NULL,
    UNIQUE(user_id, permission, partner_id)
);