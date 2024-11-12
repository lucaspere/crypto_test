use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Request, Requester};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, Me, ParseMode, Recipient, ReplyMarkup, UserId,
};
use teloxide::Bot;

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
            .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
                vec![vec![InlineKeyboardButton::callback("Back", "start")]],
            )))
            .send()
            .await?;

        Ok(())
    }
}
