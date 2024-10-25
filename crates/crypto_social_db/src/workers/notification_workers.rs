use crate::services::notification_service::{Notification, NotificationService};
use std::sync::Arc;
use tokio::time::{self, Duration};

pub struct NotificationWorker {
    notification_service: Arc<NotificationService>,
}

impl NotificationWorker {
    pub fn new(notification_service: Arc<NotificationService>) -> Self {
        Self {
            notification_service,
        }
    }

    pub async fn run(&self) {
        loop {
            match self.process_notifications().await {
                Ok(_) => {}
                Err(e) => eprintln!("Error processing notifications: {:?}", e),
            }
            time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn process_notifications(&self) -> Result<(), Box<dyn std::error::Error>> {
        let notifications = self.notification_service.get_notifications().await?;

        for notification in notifications {
            self.send_notification(&notification).await?;
        }

        Ok(())
    }

    async fn send_notification(
        &self,
        notification: &Notification,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Implement the logic to send notifications to the webapp and Telegram
        println!("Sending notification: {:?}", notification);
        // TODO: Implement webapp notification (e.g., through WebSockets)
        // TODO: Implement Telegram notification
        Ok(())
    }
}
