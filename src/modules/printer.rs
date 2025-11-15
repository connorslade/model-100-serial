use std::{mem, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use nalgebra::Vector2;
use printers::{
    common::base::{job::PrinterJobOptions, printer::Printer},
    get_printers,
};
use tokio::time::Instant;

use crate::{modules::Module, state::State};

pub struct PrinterModule {
    printers: Vec<Printer>,
    state: StateMachine,
}

enum StateMachine {
    SelectPrinter {
        selected: usize,
    },
    Uploading {
        printer: usize,
        file: Vec<u8>,

        last_size: usize,
        last_update: Instant,
    },
    Printing {
        printer: usize,
        job_id: u64,
    },
}

#[async_trait]
impl Module for PrinterModule {
    async fn init(&mut self, screen: &mut State) -> Result<()> {
        self.draw(screen).await?;

        Ok(())
    }

    async fn on_key(&mut self, screen: &mut State, key: u8) -> Result<()> {
        if key == 0x1B {
            screen.exit();
            return Ok(());
        }

        match &mut self.state {
            StateMachine::SelectPrinter { selected } => match key {
                0x1D => *selected = selected.saturating_sub(1),
                0x1C => *selected = (*selected + 1).min(self.printers.len() - 1),
                0x0D => {
                    self.state = StateMachine::Uploading {
                        printer: *selected,
                        file: Vec::new(),
                        last_size: 0,
                        last_update: Instant::now(),
                    }
                }
                _ => {}
            },
            StateMachine::Uploading { file, .. } => {
                if file.is_empty() {
                    screen.schedule(Duration::from_millis(250), 0);
                }

                file.push(key);
            }
            _ => {}
        }

        self.draw(screen).await?;

        Ok(())
    }

    async fn callback(&mut self, screen: &mut State, kind: u32) -> Result<()> {
        if kind != 0 {
            return Ok(());
        }

        match &mut self.state {
            StateMachine::Uploading {
                printer,
                file,
                last_size,
                ..
            } => {
                if mem::replace(last_size, file.len()) == file.len() {
                    for byte in file.iter_mut() {
                        (*byte == b'\r').then(|| *byte = b'\n');
                    }

                    let job_id = self.printers[*printer]
                        .print(file, PrinterJobOptions::none())
                        .unwrap();
                    self.state = StateMachine::Printing {
                        printer: *printer,
                        job_id,
                    };
                    self.draw(screen).await?;
                }
                screen.schedule(Duration::from_millis(250), 0);
            }
            StateMachine::Printing { printer, job_id } => {
                let jobs = self.printers[*printer].get_active_jobs();

                // todo: dont edit screen directly in callback
                let Some(job) = jobs.iter().find(|x| x.id == *job_id) else {
                    screen.write_string(Vector2::new(0, 3), b"Print successful");
                    screen.draw().await?;
                    return Ok(());
                };

                let message = format!("Print Status: {:?}", job.state);
                screen.write_string(Vector2::new(0, 3), message.as_bytes());
                screen.draw().await?;
                screen.schedule(Duration::from_millis(250), 0);
            }
            _ => {}
        }

        Ok(())
    }
}

impl PrinterModule {
    async fn draw(&mut self, screen: &mut State) -> Result<()> {
        match &mut self.state {
            StateMachine::SelectPrinter { selected } => {
                let message = format!("Printer: {}", self.printers[*selected].name);
                screen.rect(Vector2::new(0, 1), Vector2::new(40, 1), b' '.into());
                screen.write_string(Vector2::new(0, 1), message.as_bytes());
                screen.draw().await?;
            }
            StateMachine::Uploading {
                file, last_update, ..
            } => {
                if last_update.elapsed().as_millis() >= 250 {
                    *last_update = Instant::now();

                    screen.write_string(
                        Vector2::new(0, 1),
                        format!("Received {} bytes.", file.len()).as_bytes(),
                    );

                    screen.draw().await?;
                }
            }
            StateMachine::Printing { .. } => {
                screen.write_string(Vector2::new(0, 2), b"Upload complete!");
                screen.draw().await?;
            }
        }
        Ok(())
    }
}

impl Default for PrinterModule {
    fn default() -> Self {
        Self {
            printers: get_printers(),
            state: StateMachine::SelectPrinter { selected: 0 },
        }
    }
}
