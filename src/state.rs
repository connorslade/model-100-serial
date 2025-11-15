use std::{
    collections::BinaryHeap,
    mem,
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::Result;
use tokio::{
    io::{AsyncWriteExt, BufWriter, WriteHalf},
    time::Instant,
};
use tokio_serial::SerialStream;

use crate::screen::Screen;

type SerialWriter = BufWriter<WriteHalf<SerialStream>>;

pub struct State {
    screen: Screen,
    writer: SerialWriter,

    pub(super) timeouts: BinaryHeap<Timeout>,
    exit: bool,
}

pub(super) struct Timeout {
    pub time: Instant,
    pub kind: u32,
}

impl State {
    pub async fn new(mut writer: SerialWriter) -> Result<Self> {
        // Reset screen, disable scroll, hide cursor
        writer.write_all(b"\x0C\x1Bq\x1BV\x1BQ").await?;

        Ok(Self {
            screen: Screen::new(),
            writer,

            timeouts: BinaryHeap::new(),
            exit: false,
        })
    }

    pub async fn draw(&mut self) -> Result<()> {
        self.screen.draw(&mut self.writer).await
    }

    pub fn schedule(&mut self, duration: Duration, kind: u32) {
        self.timeouts.push(Timeout {
            time: Instant::now() + duration,
            kind,
        });
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

impl PartialEq for Timeout {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for Timeout {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for Timeout {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl Eq for Timeout {}
