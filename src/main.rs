use std::io::{BufWriter, ErrorKind, Write};

use anyhow::{Result, bail};
use nd_vec::Vec2;

mod screen;
use screen::Screen;
use serialport::FlowControl;

fn main() -> Result<()> {
    let mut port = serialport::new("/dev/ttyUSB1", 19_200)
        .flow_control(FlowControl::Software)
        .open()?;
    let mut writer = BufWriter::new(port.try_clone()?);

    writer.write_all(b"\x0C\x1Bq\x1BV\x1BQ")?; // Reset screen, disable scroll, hide cursor
    // writer.write_all(b"\x1BP")?; // Show cursor

    let mut screen = Screen::new();

    for y in 1..7 {
        screen.put(Vec2::new([0, y]), b'\xF5'.into());
        screen.put(Vec2::new([39, y]), b'\xF5'.into());
    }

    for x in 1..39 {
        screen.put(Vec2::new([x, 0]), b'\xF1'.into());
        screen.put(Vec2::new([x, 5]), b'\xF1'.into());
        screen.put(Vec2::new([x, 7]), b'\xF1'.into());
    }

    screen.write_string(Vec2::new([15, 0]), b" CHAT-GPT ");

    screen.put(Vec2::new([0, 0]), b'\xF0'.into());
    screen.put(Vec2::new([39, 0]), b'\xF2'.into());

    screen.put(Vec2::new([0, 5]), b'\xF4'.into());
    screen.put(Vec2::new([39, 5]), b'\xF9'.into());

    screen.put(Vec2::new([0, 7]), b'\xF6'.into());
    screen.put(Vec2::new([39, 7]), b'\xF7'.into());

    screen.put(Vec2::new([0, 6]), b'>'.into());
    screen.put(Vec2::new([1, 6]), b'\xE9'.into());

    screen.draw(&mut writer)?;
    writer.flush()?;

    let mut prompt = vec![b'>'];
    let mut response = Vec::new();
    loop {
        let mut buf = [0u8; 1];
        match port.read(&mut buf) {
            Ok(0) => continue,
            Ok(_) => {}
            Err(e) if matches!(e.kind(), ErrorKind::Interrupted | ErrorKind::TimedOut) => continue,
            Err(e) => bail!(e),
        }

        dbg!(buf[0]);
        if buf[0] == 8 {
            if prompt.len() > 1 {
                prompt.pop();
            }
        } else if buf[0] == 13 {
            response = prompt[1..].to_vec();
            prompt = vec![b'>'];
        } else {
            prompt.push(buf[0]);
        }

        let display = &prompt[prompt.len().saturating_sub(39)..];
        screen.rect(Vec2::new([0, 6]), Vec2::new([39, 1]), b' '.into());
        screen.rect(Vec2::new([1, 1]), Vec2::new([38, 4]), b' '.into());

        screen.write_string(Vec2::new([0, 6]), &display);
        screen.put(Vec2::new([39, 6]), b'\xF5'.into());
        screen.put(Vec2::new([display.len(), 6]), b'\xE9'.into());

        screen.write_string_wrapped(Vec2::new([1, 1]), &response, 38);

        screen.draw(&mut writer)?;
        writer.flush()?;
    }
}
