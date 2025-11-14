use std::{
    mem,
    ops::{Deref, DerefMut},
};

use anyhow::Result;
use tokio::io::{AsyncWriteExt, BufWriter, WriteHalf};
use tokio_serial::SerialStream;

use crate::screen::Screen;

type SerialWriter = BufWriter<WriteHalf<SerialStream>>;

pub struct State {
    screen: Screen,
    writer: SerialWriter,

    exit: bool,
}

impl State {
    pub async fn new(mut writer: SerialWriter) -> Result<Self> {
        // Reset screen, disable scroll, hide cursor
        writer.write_all(b"\x0C\x1Bq\x1BV\x1BQ").await?;

        Ok(Self {
            screen: Screen::new(),
            writer,
            exit: false,
        })
    }

    pub async fn draw(&mut self) -> Result<()> {
        self.screen.draw(&mut self.writer).await
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub(super) fn take_exit(&mut self) -> bool {
        mem::take(&mut self.exit)
    }
}

impl Deref for State {
    type Target = Screen;

    fn deref(&self) -> &Self::Target {
        &self.screen
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.screen
    }
}
