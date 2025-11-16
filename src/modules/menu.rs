use anyhow::Result;
use async_trait::async_trait;
use nalgebra::Vector2;

use crate::{
    modules::{Module, chatgpt::ChatGptModule, printer::PrinterModule},
    state::State,
};

pub struct Menu {
    selection: u8,
    module: Option<Box<dyn Module + Send>>,
}

impl Menu {
    async fn draw(&mut self, screen: &mut State) -> Result<()> {
        const OPTIONS: &[&[u8]] = &[b"Chat-GPT", b"Printer", b"Exit"];

        screen.clear();
        for (i, option) in OPTIONS.iter().enumerate() {
            if self.selection == i as u8 {
                screen.put(Vector2::new(18 - option.len() / 2, i + 1), b'>'.into());
                screen.put(
                    Vector2::new(21 - option.len() / 2 + option.len(), i + 1),
                    b'<'.into(),
                );
            }

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

    async fn on_key(&mut self, screen: &mut State, key: u8) -> Result<()> {
        if key == 0x12 {
            screen.redraw().await?;
            return Ok(());
        }

        if let Some(module) = &mut self.module {
            module.on_key(screen, key).await?;

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
                let mut module: Box<dyn Module + Send> = match self.selection {
                    0 => Box::new(ChatGptModule::default()),
                    1 => Box::new(PrinterModule::default()),
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

    async fn callback(&mut self, screen: &mut State, kind: u32) -> Result<()> {
        if let Some(module) = &mut self.module {
            module.callback(screen, kind).await?;
        }

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
