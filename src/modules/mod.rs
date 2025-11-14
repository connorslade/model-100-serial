use anyhow::Result;
use async_trait::async_trait;

use crate::state::State;

pub mod chatgpt;
pub mod menu;

#[async_trait]
pub trait Module {
    async fn init(&mut self, screen: &mut State) -> Result<()>;
    async fn on_key(&mut self, key: u8, screen: &mut State) -> Result<()>;
}
