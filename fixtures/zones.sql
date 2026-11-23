INSERT INTO users (user_id, full_name, email, hashed_password, global_role)
VALUES (101, 'Zone User', 'zone-user@example.com', 'unused', 'user'),
       (102, 'Zone Admin', 'zone-admin@example.com', 'unused', 'admin');
INSERT INTO partners (partner_id, legal_name, slug, contact_email, contact_phone)
VALUES (1, 'Zone Partner', 'zone-partner', 'zone-partner@example.com', '88005553536');
INSERT INTO cameras (camera_id, title, source, image_width, image_height, latitude, longitude, partner_id)
VALUES (1, 'Public Camera', 'rtsp://public.example.com', 1920, 1080, 60, 30, NULL),
       (7, 'Partner Camera', 'rtsp://partner.example.com', 1920, 1080, 60, 30, 1);
