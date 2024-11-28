use teloxide::net::Download;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Request, Requester};
use teloxide::types::{
    ChatKind, InlineKeyboardButton, InlineKeyboardMarkup, LinkPreviewOptions, Me, ParseMode,
    Recipient, ReplyMarkup, UserId,
};
use teloxide::Bot;
use uuid::Uuid;

use crate::models::users::SavedUser;
use crate::utils::api_errors::ApiError;

pub struct TeloxideTelegramBotApi {
    bot: Bot,
    pub bot_info: Option<Me>,
}

impl TeloxideTelegramBotApi {
    pub async fn new(bot: Bot) -> Result<Self, ApiError> {
        let bot_info = bot.get_me().await.ok();
        Ok(Self { bot, bot_info })
    }
}
impl TeloxideTelegramBotApi {
    pub async fn send_message<'a>(
        &'a self,
        telegram_id: u64,
        message: &'a str,
    ) -> Result<(), ApiError> {
        self.bot
            .send_message(Recipient::from(UserId(telegram_id)), message)
            .parse_mode(ParseMode::Html)
            .link_preview_options(LinkPreviewOptions {
                is_disabled: true,
                url: None,
                prefer_small_media: false,
                prefer_large_media: false,
                show_above_text: false,
            })
            .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
                vec![vec![InlineKeyboardButton::callback("Back", "start")]],
            )))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_user_by_telegram_id(
        &self,
        telegram_id: i64,
    ) -> Result<(SavedUser, Option<String>), ApiError> {
        let user = self
            .bot
            .get_chat(Recipient::from(UserId(telegram_id as u64)))
            .await?;
        let bio = user.bio().map(|bio| bio.to_string());
        let username = match user.kind {
            ChatKind::Public(public) => public.title,
            ChatKind::Private(group) => group.username,
        }
        .unwrap_or_default();
        let photo = user.photo.map(|p| p.small_file_id);
        let user = SavedUser {
            id: Uuid::new_v4(),
            username,
            telegram_id: telegram_id,
            selected_wallet_id: None,
            waitlisted: false,
            image_uri: None,
            bio,
        };

        Ok((user, photo))
    }

    pub async fn get_user_avatar_by_id(&self, telegram_id: i64) -> Result<Vec<u8>, ApiError> {
        let user = self
            .bot
            .get_user_profile_photos(UserId(telegram_id as u64))
            .await?;

        let photo = user
            .photos
            .first()
            .and_then(|photos| photos.last())
            .ok_or(ApiError::NotFound("User has no profile photo".into()))?;

        let photo_url = photo.file.id.clone();
        let file = self.bot.get_file(photo_url).await?;

        let mut image_data = Vec::new();
        self.bot.download_file(&file.path, &mut image_data).await?;
        Ok(image_data)
    }

    pub async fn get_user_avatar_by_file_id(&self, file_id: &str) -> Result<Vec<u8>, ApiError> {
        let file = self.bot.get_file(file_id).await?;

        let mut image_data = Vec::new();
        self.bot.download_file(&file.path, &mut image_data).await?;
        Ok(image_data)
    }
}
