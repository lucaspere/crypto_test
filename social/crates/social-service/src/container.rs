use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    external_services::{
        birdeye::BirdeyeService, cielo::CieloService, rust_monorepo::RustMonorepoService,
    },
    repositories::{
        group_repository::GroupRepository, token_repository::TokenRepository,
        user_repository::UserRepository,
    },
    services::{
        group_service::GroupService, profile_service::ProfileService, redis_service::RedisService,
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
}

impl ServiceContainer {
    pub async fn new(
        settings: &Settings,
        db: Arc<PgPool>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let user_repository = Arc::new(UserRepository::new(db.clone()));
        let token_repository = Arc::new(TokenRepository::new(db.clone()));
        let redis_service = Arc::new(RedisService::new(&settings.redis_url).await?);

        let user_service = Arc::new(UserService::new(user_repository.clone()));
        let group_service = Arc::new(GroupService::new(
            Arc::new(GroupRepository::new(db)),
            user_service.clone(),
            Arc::new(None),
        ));
        let token_service = Arc::new(TokenService::new(
            token_repository.clone(),
            Arc::new(RustMonorepoService::new(settings.rust_monorepo_url.clone())),
            user_service.clone(),
            redis_service.clone(),
            Arc::new(BirdeyeService::new(settings.birdeye_api_key.clone())),
            group_service.clone(),
        ));
        let profile_service = Arc::new(ProfileService::new(
            user_repository,
            token_repository,
            Arc::new(RustMonorepoService::new(settings.rust_monorepo_url.clone())),
            Arc::new(BirdeyeService::new(settings.birdeye_api_key.clone())),
            redis_service.clone(),
            token_service.clone(),
            Arc::new(CieloService::new(
                settings.cielo_api_key.clone(),
                redis_service.clone(),
            )),
        ));

        Ok(Self {
            user_service,
            profile_service,
            token_service,
            group_service,
            redis_service,
        })
    }
}
