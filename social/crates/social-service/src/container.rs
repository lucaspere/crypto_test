use std::sync::Arc;

use sqlx::PgPool;
use teloxide::Bot;

use crate::{
    external_services::{
        birdeye::BirdeyeService, cielo::CieloService,
        ext_data_services_v1::token_data::TokenDataService, rust_monorepo::RustMonorepoService,
        usergate::UserGateService,
    },
    repositories::{
        group_repository::GroupRepository, token_repository::TokenRepository,
        user_repository::UserRepository,
    },
    services::{
        group_service::GroupService, profile_service::ProfileService, redis_service::RedisService,
        s3_service::S3Service, telegram_service::TeloxideTelegramBotApi,
        token_service::TokenService, user_service::UserService,
    },
    settings::Settings,
};

pub struct ServiceContainer {
    pub user_service: Arc<UserService>,
    pub profile_service: Arc<ProfileService>,
    pub token_service: Arc<TokenService>,
    pub group_service: Arc<GroupService>,
    pub redis_service: Arc<RedisService>,
    pub telegram_service: Arc<TeloxideTelegramBotApi>,
    pub rust_monorepo_service: Arc<RustMonorepoService>,
    pub token_data_service: Option<Arc<TokenDataService>>,
    pub s3_service: Arc<S3Service>,
    pub birdeye_service: Arc<BirdeyeService>,
    pub environment: String,
}

impl ServiceContainer {
    pub async fn new(
        settings: &Settings,
        db: Arc<PgPool>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let user_repository = Arc::new(UserRepository::new(db.clone()));
        let token_repository = Arc::new(TokenRepository::new(db.clone()));
        let redis_service = Arc::new(RedisService::new(&settings.redis_url).await?);
        let rust_monorepo_service = Arc::new(RustMonorepoService::new(
            settings.rust_monorepo_url.clone(),
            settings.rust_monorepo_api_key.clone(),
        ));
        let usergate_service = Arc::new(UserGateService::new(
            settings.usergate_url.clone(),
            settings.usergate_api_key.clone(),
        ));
        let birdeye_service = Arc::new(BirdeyeService::new(settings.birdeye_api_key.clone()));
        let s3_service = Arc::new(
            S3Service::new(
                settings.s3_bucket.clone(),
                settings.aws_access_key_id.clone(),
                settings.aws_secret_access_key.clone(),
                settings
                    .aws_region
                    .clone()
                    .unwrap_or_else(|| "us-east-1".to_string()),
            )
            .await?,
        );
        let token_data_service =
            if let Some(api_key) = settings.ext_data_services_v1_api_key.clone() {
                Some(Arc::new(TokenDataService::new(
                    api_key,
                    redis_service.clone(),
                )))
            } else {
                None
            };

        let bot = Bot::new(settings.telegram_bot_token.clone());
        let telegram_service = Arc::new(TeloxideTelegramBotApi::new(bot).await?);
        let user_service = Arc::new(UserService::new(
            user_repository.clone(),
            telegram_service.clone(),
            s3_service.clone(),
        ));
        let group_service = Arc::new(GroupService::new(
            Arc::new(GroupRepository::new(db.clone())),
            user_service.clone(),
            Arc::new(None),
            telegram_service.clone(),
            s3_service.clone(),
        ));
        let token_service = Arc::new(TokenService::new(
            token_repository.clone(),
            rust_monorepo_service.clone(),
            user_service.clone(),
            redis_service.clone(),
            birdeye_service.clone(),
            group_service.clone(),
        ));

        let profile_service = ProfileService::new(
            user_repository,
            token_repository,
            rust_monorepo_service.clone(),
            birdeye_service.clone(),
            redis_service.clone(),
            token_service.clone(),
            Arc::new(CieloService::new(
                settings.cielo_api_key.clone(),
                redis_service.clone(),
            )),
            usergate_service.clone(),
            s3_service.clone(),
        );
        let profile_group = Arc::new(Some(profile_service.clone()));
        let group_service = Arc::new(GroupService::new(
            Arc::new(GroupRepository::new(db)),
            user_service.clone(),
            profile_group,
            telegram_service.clone(),
            s3_service.clone(),
        ));
        let profile_service = Arc::new(profile_service);

        Ok(Self {
            user_service,
            profile_service,
            token_service,
            group_service,
            redis_service,
            telegram_service,
            rust_monorepo_service,
            token_data_service,
            s3_service,
            birdeye_service,
            environment: settings.environment.clone().unwrap_or("prod".to_string()),
        })
    }
}
