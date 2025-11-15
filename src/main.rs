use anyhow::Result;
use tokio::{
    io::{self, AsyncReadExt, BufWriter},
    select,
};
use tokio_serial::{FlowControl, SerialPortBuilderExt};

mod modules;
mod screen;
mod state;

use crate::{
    modules::{Module, menu::Menu},
    state::State,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let port = tokio_serial::new("/dev/ttyUSB1", 19_200)
        .flow_control(FlowControl::Software)
        .open_native_async()?;
    let (mut rx, tx) = io::split(port);

    let mut screen = State::new(BufWriter::new(tx)).await?;
    let mut menu = Menu::default();
    menu.init(&mut screen).await?;

    loop {
        select! {
            key = rx.read_u8() => menu.on_key(&mut screen, key?).await?,
            _ = async {
                if let Some(t) = screen.timeouts.peek() {
                    tokio::time::sleep_until(t.time).await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                let kind = screen.timeouts.pop().unwrap().kind;
                menu.callback(&mut screen, kind).await?;
            }
        }
    }
}
