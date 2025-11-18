use anyhow::Result;
use async_trait::async_trait;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use nalgebra::Vector2;

use crate::{modules::Module, state::State};

pub struct KeyboardModule {
    enigo: Enigo,
}

#[async_trait]
impl Module for KeyboardModule {
    async fn init(&mut self, screen: &mut State) -> Result<()> {
        screen.write_string(Vector2::new(0, 0), b"Keyboard mode.");
        screen.write_string(Vector2::new(0, 1), b"Press GRAPH+Q to exit.");
        screen.draw().await?;
        Ok(())
    }

    async fn on_key(&mut self, screen: &mut State, key: u8) -> Result<()> {
        dbg!(key);

        if key == 0x93 {
            screen.exit();
            return Ok(());
        }

        if matches!(key, 1..=7 | 9..=12 | 14..=26) {
            let _ = self.enigo.key(Key::Control, Direction::Press);
            let _ = self
                .enigo
                .key(Key::Unicode((key - 1 + b'a') as char), Direction::Click);
            let _ = self.enigo.key(Key::Control, Direction::Release);
        } else {
            let key = match key {
                0x1B => Key::Escape,
                0x1D => Key::LeftArrow,
                0x1C => Key::RightArrow,
                0x1E => Key::UpArrow,
                0x1F => Key::DownArrow,
                _ => Key::Unicode(key as char),
            };

            let _ = self.enigo.key(key, Direction::Click);
        }

        Ok(())
    }
}

impl Default for KeyboardModule {
    fn default() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap(),
        }
    }
}
