use crate::entities::{user_follows, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect,
    RelationTrait, Set,
};
use uuid::Uuid;

pub async fn follow_user(
    db: &DatabaseConnection,
    follower_id: Uuid,
    followed_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let follow = user_follows::ActiveModel {
        follower_id: Set(follower_id),
        followed_id: Set(followed_id),
        created_at: Set(chrono::Utc::now().into()),
    };

    user_follows::Entity::insert(follow).exec(db).await?;
    Ok(())
}

pub async fn unfollow_user(
    db: &DatabaseConnection,
    follower_id: Uuid,
    followed_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    user_follows::Entity::delete_many()
        .filter(user_follows::Column::FollowerId.eq(follower_id))
        .filter(user_follows::Column::FollowedId.eq(followed_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn get_followers(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<users::Model>, Box<dyn std::error::Error>> {
    let followers = users::Entity::find()
        .join(
            sea_orm::JoinType::InnerJoin,
            users::Relation::UserFollows.def().rev(),
        )
        .filter(user_follows::Column::FollowedId.eq(user_id))
        .all(db)
        .await?;

    Ok(followers)
}

pub async fn get_following(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<users::Model>, Box<dyn std::error::Error>> {
    let following = users::Entity::find()
        .join(
            sea_orm::JoinType::InnerJoin,
            users::Relation::UserFollows.def().rev(),
        )
        .filter(user_follows::Column::FollowerId.eq(user_id))
        .all(db)
        .await?;

    Ok(following)
}
