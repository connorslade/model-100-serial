use anyhow::Result;
use async_trait::async_trait;

use crate::state::State;

pub mod chatgpt;
pub mod menu;
pub mod printer;

#[async_trait]
pub trait Module {
    async fn init(&mut self, screen: &mut State) -> Result<()>;
    async fn on_key(&mut self, screen: &mut State, key: u8) -> Result<()>;
    async fn callback(&mut self, screen: &mut State, kind: u32) -> Result<()> {
        let _ = (screen, kind);
        Ok(())
    }
}
