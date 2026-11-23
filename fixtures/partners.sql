INSERT INTO users (user_id,full_name,email,hashed_password,global_role)
VALUES (101,'Partner Manager','manager@example.com','unused','user'),
       (102,'Platform Admin','admin@example.com','unused','admin'),
       (103,'Other User','other@example.com','unused','user');
INSERT INTO partners (partner_id,legal_name,slug,contact_email,contact_phone)
VALUES (101,'First Partner','first-partner','first@example.com','+79990000001'),
       (102,'Second Partner','second-partner','second@example.com','+79990000002');
SELECT setval(pg_get_serial_sequence('partners','partner_id'),102);
INSERT INTO partner_memberships (user_id,partner_id,user_role,read_scope,write_scope,delete_scope)
VALUES (101,101,'manager','partner_all','own','own');
INSERT INTO partner_member_permissions(user_id,partner_id,permission)
VALUES (101,101,'partner_members.view'),(101,101,'partner_members.write'),(101,101,'partner_members.disable');
