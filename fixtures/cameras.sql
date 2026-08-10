INSERT INTO partners (
    partner_id,
    legal_name,
    slug,
    contact_email,
    contact_phone
) VALUES (
    1,
    'legal_name_1',
    'slug_1',
    'contact@mail.example',
    '88005553535'
);

INSERT INTO cameras (
    title, source, image_width, image_height, 
    calib, latitude, longitude, partner_id, created_by_user_id,
    is_active, last_snapshot_at
) VALUES
    -- База
    ('Улица абоба', 'rtsp://cam1.example.com/stream', 1920, 1080,
     NULL, 40.7580, -73.9855, NULL, NULL,
     TRUE, NOW()),

    -- Тест на неактив
    ('Улица сдоба', 'rtsp://cam2.example.com/stream', 1920, 1080,
     NULL, 40.7489, -73.9680, NULL, NULL,
     FALSE, NOW()),

    -- Прописанный калиб
    ('Улица гроба', 'rtsp://cam3.example.com/stream', 1280, 720,
     '{
        "image_width": 1920,
        "image_height": 1080,
        "K": [
        [1739.237279181759, 0.0, 947.5335576199107],
        [0.0, 2244.705015334057, 564.6946579168148],
        [0.0, 0.0, 1.0]
        ],
        "D": [-0.37062084436192333, 0.05057465862770827, 0.033198096980616335, 0.012812747166936252],
        "balance": 0.0,
        "model": "opencv_fisheye_k1k2k3k4"
     }'::jsonb,
     40.7061, -74.0087, NULL, NULL,
     TRUE, NOW()),

    -- Тест на фильтр по титлу
    ('Проспект абоба', 'rtsp://cam4.example.com/stream', 1280, 720,
     NULL, 40.7050, -74.0130, NULL, NULL,
     TRUE, NOW()),

    -- Тест на фильтр по ббокс
    ('Улица жлоба', 'rtsp://cam5.example.com/stream', 1920, 1080,
     NULL, 34.0522, -118.2437, NULL, NULL,
     TRUE, NOW()),

    -- Тест на отсутствие снапшота
    ('Улица амеба', 'rtsp://cam6.example.com/stream', 1920, 1080,
     NULL, 40.7300, -73.9950, NULL, NULL,
     TRUE, NULL),

     -- Тест на партнерский скоуп
     ('Улица злоба', 'rtsp://cam7.example.com/stream', 1920, 1080,
     NULL, 40.7320, -73.9930, 1, NULL,
     TRUE, NULL);
