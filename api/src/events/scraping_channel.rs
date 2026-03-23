use htc::models::scrape_batch::ScrapeBatch;
use tokio::sync::broadcast;
use tracing::info;

use crate::events::EventHandler;

pub struct ScrapingChannel {
    pub sender: broadcast::Sender<ScrapeBatch>,
}

#[derive(thiserror::Error, Debug)]
pub enum ScrapingChannelError {}

impl EventHandler for ScrapingChannel {
    type Input = ScrapeBatch;
    type Rejection = ScrapingChannelError;

    async fn handle(&self, input: Self::Input) -> Result<(), Self::Rejection> {
        info!("FOUND NEW BATCH : {:#?}", input.scraped_at);
        let _ = self.sender.send(input);
        Ok(())
    }
}
