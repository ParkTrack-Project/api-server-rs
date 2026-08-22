CREATE TABLE IF NOT EXISTS partner_member_permissions (
    user_id INTEGER NOT NULL,
    partner_id INTEGER NOT NULL,
    permission VARCHAR(100) NOT NULL,
    granted_at TIMESTAMP DEFAULT NOW(),
    granted_by INTEGER REFERENCES users(user_id) ON DELETE SET NULL,
    PRIMARY KEY (user_id, permission, partner_id),
    FOREIGN KEY (user_id, partner_id)
        REFERENCES partner_memberships (user_id, partner_id)
        ON DELETE CASCADE
);