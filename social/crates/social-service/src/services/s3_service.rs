use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{config::Credentials, primitives::ByteStream, Client, Config};
use bytes::Bytes;
use image::{imageops::FilterType, ImageFormat};
use std::io::Cursor;
use tracing::error;

use crate::utils::errors::app_error::AppError;

const PROFILE_AVATARS_PATH: &str = "profile-avatars";
const AVATAR_SIZE: u32 = 400; // 400x400 pixels for avatars

pub struct S3Service {
    client: Client,
    bucket: String,
}

impl S3Service {
    pub async fn new(
        bucket: String,
        access_key: String,
        secret_key: String,
        region: String,
    ) -> Result<Self, AppError> {
        let credentials = Credentials::new(access_key, secret_key, None, None, "env");

        let config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region))
            .credentials_provider(credentials)
            .build();

        let client = Client::from_conf(config);
        Ok(Self { client, bucket })
    }

    async fn process_image(&self, image_data: Bytes) -> Result<(Bytes, &str), AppError> {
        let img = image::load_from_memory(&image_data).map_err(|e| {
            error!("Error loading image: {}", e);
            AppError::InvalidFileType
        })?;

        let resized = img.resize(AVATAR_SIZE, AVATAR_SIZE, FilterType::Lanczos3);

        let mut buffer = Cursor::new(Vec::new());
        resized
            .write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| AppError::S3Error(e.to_string()))?;

        Ok((Bytes::from(buffer.into_inner()), "image/png"))
    }

    pub async fn upload_profile_image(
        &self,
        user_telegram_id: &i64,
        image_data: Bytes,
        _content_type: &str,
    ) -> Result<String, AppError> {
        let (processed_image, content_type) = self.process_image(image_data).await?;

        let key = format!("{}/{}.png", PROFILE_AVATARS_PATH, user_telegram_id);

        let body = ByteStream::from(processed_image);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::S3Error(e.to_string()))?;

        Ok(format!("https://{}.s3.amazonaws.com/{}", self.bucket, key))
    }

    pub async fn delete_profile_image(&self, user_telegram_id: &i64) -> Result<(), AppError> {
        let objects = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(format!("{}/{}", PROFILE_AVATARS_PATH, user_telegram_id))
            .send()
            .await
            .map_err(|e| AppError::S3Error(e.to_string()))?;

        for object in objects.contents() {
            if let Some(key) = &object.key {
                self.client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| AppError::S3Error(e.to_string()))?;
            }
        }

        Ok(())
    }

    pub fn get_profile_image_url(&self, user_telegram_id: i64) -> String {
        format!(
            "https://{}.s3.amazonaws.com/{}/{}.png",
            self.bucket, PROFILE_AVATARS_PATH, user_telegram_id
        )
    }
}
