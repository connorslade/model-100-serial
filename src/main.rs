use anyhow::Result;
use tokio::io::{self, AsyncReadExt, BufWriter};
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
        let key = rx.read_u8().await?;
        menu.on_key(key, &mut screen).await?;
    }
}
