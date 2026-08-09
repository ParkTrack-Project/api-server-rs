mod repository {
    use api_server_rust::{cameras::{repository::CameraAccess, *}, error::{ApiError, ApiResult}, types::Bbox};
    use sqlx::PgPool;

    const FIXTURE_CAMERAS_COUNT: u32 = 7;

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn list_cameras_with_title_single_match(pool: PgPool) -> ApiResult<()> {
        // Arrange
        let q = requests::ListCamerasQuery {
            q: "Проспект".to_string().into(),
            is_active: None,
            bbox: None,
            view: api_server_rust::types::CameraView::Full,
        };

        let scope = repository::CameraAccess::All;

        // Act
        let res = repository::list_cameras(&pool, q, scope).await?;

        // Assert
        assert_ne!(res.len(), 0);
        assert!(res.iter().all(|c| c.title.contains("Проспект")));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn list_cameras_with_inactive_ok(pool: PgPool) -> ApiResult<()> {
        // Arrange
        let q = requests::ListCamerasQuery {
            q: None,
            is_active: false.into(),
            bbox: None,
            view: api_server_rust::types::CameraView::Full,
        };

        let scope = repository::CameraAccess::All;

        // Act
        let res = repository::list_cameras(&pool, q, scope).await?;

        // Assert
        assert_ne!(res.len(), 0);
        assert!(res.iter().all(|c| c.is_active == false));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn list_cameras_with_bbox_ok(pool: PgPool) -> ApiResult<()> {
        // Arrange
        let q = requests::ListCamerasQuery {
            q: None,
            is_active: None,
            bbox: Bbox::new(-120., 30., -100., 40.).into(),
            view: api_server_rust::types::CameraView::Full,
        };

        let scope = repository::CameraAccess::All;

        // Act
        let res = repository::list_cameras(&pool, q, scope).await?;

        // Assert
        assert_ne!(res.len(), 0);
        assert!(res.iter().all(
            |c| (30.0..=40.0).contains(&c.latitude) && (-120.0..=-100.0).contains(&c.longitude)
        ));

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn list_cameras_as_map_ok(pool: PgPool) -> ApiResult<()> {
        // Arrange
        let q = requests::ListCamerasQuery {
            q: None,
            is_active: None,
            bbox: None,
            view: api_server_rust::types::CameraView::Map,
        };

        let scope = repository::CameraAccess::All;

        // Act
        let res = repository::list_cameras(&pool, q, scope).await?;

        // Assert
        assert_eq!(res.len(), FIXTURE_CAMERAS_COUNT as usize);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn create_camera_ok(pool: PgPool) -> ApiResult<()> {
        // Arrange
            let q = requests::CreateCamera {
                title: String::new(),
                source: "popa".to_string(),
                image_width: 1,
                image_height: 1,
                calib: None,
                latitude: 0.,
                longitude: 0.
            };
        // Act

        let _ = repository::create_camera(&pool, q, None).await?;

        // Assert
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn delete_camera_with_bad_id_returns_not_found(pool: PgPool) -> ApiResult<()> {
        // Arrange

        let scope = CameraAccess::All;

        // Act
        let res = repository::delete_camera(&pool, 67, scope).await;

        // Assert
        assert!(if let Err(ApiError::NotFound(_)) = res {true} else {false});
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn delete_camera_with_bad_scope_returns_not_found(pool: PgPool) -> ApiResult<()> {
        // Arrange

        let scope = CameraAccess::Public;

        // Act
        let res = repository::delete_camera(&pool, 7, scope).await;
        
        // Assert
        assert!(if let Err(ApiError::NotFound(_)) = res {true} else {false});
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("cameras")))]
    fn get_camera_with_bad_scope_returns_not_found(pool: PgPool) -> ApiResult<()> {
        // Arrange

        let scope = CameraAccess::Public;

        // Act
        let res = repository::get_camera(&pool, 7, scope).await;
        
        // Assert
        assert!(if let Err(ApiError::NotFound(_)) = res {true} else {false});
        Ok(())
    }
}
