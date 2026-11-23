INSERT INTO users (user_id,full_name,email,hashed_password,global_role,is_active)
SELECT id, 'Resource User', 'resource-' || id || '@example.com', 'unused',
       (CASE WHEN id=102 THEN 'admin' ELSE 'user' END)::global_roles, id<>108
FROM generate_series(101,108) id;
INSERT INTO partners (partner_id,legal_name,slug,contact_email,contact_phone,is_active)
VALUES (101,'Resource Partner One','resource-one','resource-one@example.com','+79990000101',TRUE),
       (102,'Resource Partner Two','resource-two','resource-two@example.com','+79990000102',TRUE),
       (103,'Disabled Partner','resource-disabled','resource-disabled@example.com','+79990000103',FALSE);
INSERT INTO partner_memberships(user_id,partner_id,user_role,read_scope,write_scope,delete_scope,is_active)
VALUES (101,101,'operator','own','own','own',TRUE),
       (103,101,'operator','partner_all','partner_all','partner_all',TRUE),
       (105,101,'operator','none','none','none',TRUE),
       (106,101,'viewer','partner_all','partner_all','partner_all',TRUE),
       (107,101,'operator','partner_all','partner_all','partner_all',FALSE),
       (108,101,'operator','partner_all','partner_all','partner_all',TRUE),
       (101,103,'operator','partner_all','partner_all','partner_all',TRUE);
INSERT INTO partner_member_permissions(user_id,partner_id,permission)
SELECT user_id,partner_id,permission FROM partner_memberships
CROSS JOIN (VALUES ('cameras.view'),('cameras.write'),('cameras.delete'),('zones.view'),('zones.write'),('zones.delete')) permissions(permission)
WHERE user_id<>106 OR permission IN ('cameras.view','zones.view');
INSERT INTO cameras(camera_id,title,source,image_width,image_height,latitude,longitude,partner_id,created_by_user_id)
VALUES (101,'Own Camera','rtsp://own.example.com',1920,1080,60,30,101,101),
       (102,'Other Camera','rtsp://other.example.com',1920,1080,60,30,101,103),
       (103,'Foreign Camera','rtsp://foreign.example.com',1920,1080,60,30,102,101),
       (104,'Public Camera','rtsp://public.example.com',1920,1080,60,30,NULL,101),
       (105,'Unowned Camera','rtsp://unowned.example.com',1920,1080,60,30,101,NULL);
SELECT setval(pg_get_serial_sequence('cameras','camera_id'),105);
INSERT INTO parking_zones(parking_zone_id,camera_id,zone_type,capacity,geometry,image_polygon,partner_id,created_by_user_id,centroid_latitude,centroid_longitude)
SELECT camera_id,camera_id,'standard',10,
       '{"type":"Polygon","coordinates":[[[30,60],[31,60],[31,61],[30,60]]]}',
       '[[0,0],[10,0],[10,10],[0,10]]',partner_id,created_by_user_id,60.25,30.5 FROM cameras;
SELECT setval(pg_get_serial_sequence('parking_zones','parking_zone_id'),105);
