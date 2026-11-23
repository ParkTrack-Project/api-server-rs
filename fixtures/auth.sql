INSERT INTO users (user_id, full_name, email, hashed_password, global_role, is_active)
VALUES
    (101, 'Test User', 'user@example.com', 'replaced-by-test-setup', 'user', TRUE),
    (102, 'Test Admin', 'admin@example.com', 'replaced-by-test-setup', 'admin', TRUE),
    (103, 'Disabled User', 'disabled@example.com', 'replaced-by-test-setup', 'user', FALSE);

INSERT INTO partners (partner_id, legal_name, slug, contact_email, contact_phone)
VALUES (101, 'Auth Test Partner', 'auth-test-partner', 'partner@example.com', '88005553535');

INSERT INTO partner_memberships (user_id, partner_id, user_role, read_scope, write_scope, delete_scope)
VALUES (101, 101, 'member', 'partner_all', 'own', 'own');

INSERT INTO partner_member_permissions (user_id, partner_id, permission)
VALUES (101, 101, 'cameras:read');
