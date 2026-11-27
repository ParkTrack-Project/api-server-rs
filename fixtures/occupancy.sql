INSERT INTO parking_zones (
    parking_zone_id, camera_id, zone_type, capacity, geometry, image_polygon,
    partner_id, location_type, pay, is_accessible, centroid_longitude, centroid_latitude
) VALUES
    (1, 1, 'parallel', 10, '{"type":"Polygon","coordinates":[[[30,60],[31,60],[31,61],[30,60]]]}', '[]',
     NULL, 'street', 20, TRUE, 30, 60),
    (2, 7, 'standard', 4, '{"type":"Polygon","coordinates":[[[35,55],[36,55],[36,56],[35,55]]]}', '[]',
     1, 'yard', 0, FALSE, 35, 55);
