use anyhow::Result;
use async_trait::async_trait;
use nalgebra::Vector2;

use crate::{
    modules::{Module, chatgpt::ChatGpt},
    state::State,
};

pub struct Menu {
    selection: u8,
    module: Option<Box<dyn Module + Send>>,
}

impl Menu {
    async fn draw(&mut self, screen: &mut State) -> Result<()> {
        const OPTIONS: &[&[u8]] = &[b"Chat-GPT", b"About", b"Exit"];

        for (i, option) in OPTIONS.iter().enumerate() {
            screen.write_string_inverted(
                Vector2::new(20 - option.len() / 2, i + 1),
                option,
                self.selection == i as u8,
            );
        }

        screen.draw().await?;
        Ok(())
    }
}

#[async_trait]
impl Module for Menu {
    async fn init(&mut self, screen: &mut State) -> Result<()> {
        self.draw(screen).await?;
        Ok(())
    }

    async fn on_key(&mut self, key: u8, screen: &mut State) -> Result<()> {
        if let Some(module) = &mut self.module {
            module.on_key(key, screen).await?;

            if screen.take_exit() {
                screen.clear();
                self.draw(screen).await?;
                self.module = None;
            }
            return Ok(());
        }

        match key {
            0x1E => self.selection = self.selection.saturating_sub(1),
            0x1F => self.selection += 1,
            0x0D => {
                let mut module = match self.selection {
                    0 => Box::new(ChatGpt::default()),
                    _ => unreachable!(),
                };

                screen.clear();
                module.init(screen).await?;
                self.module = Some(module);
                return Ok(());
            }
            _ => {}
        };
        self.draw(screen).await?;

        Ok(())
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            selection: 0,
            module: None,
        }
    }
}
