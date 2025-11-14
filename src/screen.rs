use anyhow::Result;
use nalgebra::Vector2;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Char {
    char: u8,
    inverted: bool,
}

pub struct Screen {
    previous: [Char; Self::WIDTH * Self::HEIGHT],
    chars: [Char; Self::WIDTH * Self::HEIGHT],
    cursor: Vector2<usize>,
    inverted: bool,
}

impl Char {
    pub fn invert(mut self) -> Self {
        self.inverted ^= true;
        self
    }
}

impl Screen {
    const WIDTH: usize = 40;
    const HEIGHT: usize = 8;

    const SIZE: usize = Self::WIDTH * Self::HEIGHT;

    fn index(pos: Vector2<usize>) -> Option<usize> {
        (pos.x < Self::WIDTH && pos.y < Self::HEIGHT).then(|| pos.x + pos.y * Self::WIDTH)
    }

    fn from_index(index: usize) -> Option<Vector2<usize>> {
        (index < Self::SIZE).then(|| Vector2::new(index % Self::WIDTH, index / Self::WIDTH))
    }
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            previous: [Char::default(); Self::SIZE],
            cursor: Vector2::zeros(),
            inverted: false,

            chars: [Char::default(); Self::SIZE],
        }
    }

    pub fn put(&mut self, pos: Vector2<usize>, chr: Char) {
        if let Some(index) = Screen::index(pos) {
            self.chars[index] = chr;
        }
    }

    pub fn clear(&mut self) {
        self.chars.fill(Char::default());
    }

    pub fn write_string(&mut self, pos: Vector2<usize>, str: &[u8]) {
        for (i, chr) in str.iter().enumerate() {
            self.put(pos + Vector2::x() * i, (*chr).into());
        }
    }

    pub fn write_string_inverted(&mut self, pos: Vector2<usize>, str: &[u8], invert: bool) {
        for (i, chr) in str.iter().enumerate() {
            let mut chr = Char::from(*chr);
            invert.then(|| chr = chr.invert());

            self.put(pos + Vector2::x() * i, chr);
        }
    }

    pub fn write_string_wrapped(&mut self, pos: Vector2<usize>, str: &[u8], width: usize) {
        let mut offset = Vector2::zeros();
        for word in str.split(|x| *x == b' ') {
            if offset.x + word.len() > width {
                offset.x = 0;
                offset.y += 1;
            }

            for (i, chr) in word.iter().enumerate() {
                self.put(pos + offset + Vector2::x() * i, (*chr).into());
            }

            offset.x += word.len() + 1;
        }
    }

    pub fn rect(&mut self, pos: Vector2<usize>, size: Vector2<usize>, chr: Char) {
        for y in 0..size.y {
            for x in 0..size.x {
                self.put(pos + Vector2::new(x, y), chr);
            }
        }
    }
}

impl Screen {
    async fn move_cursor<T: AsyncWrite + Unpin>(
        &mut self,
        pos: Vector2<usize>,
        writer: &mut T,
    ) -> Result<()> {
        let dx = pos.x as isize - self.cursor.x as isize;
        let dy = pos.y as isize - self.cursor.y as isize;

        let horizontal_mover = [b"\x1BD", b"\x1BC"][(dx > 0) as usize];
        let vertical_mover = [b"\x1BA", b"\x1BB"][(dy > 0) as usize];

        for _ in 0..dx.abs() {
            writer.write_all(horizontal_mover).await?;
        }

        for _ in 0..dy.abs() {
            writer.write_all(vertical_mover).await?;
        }

        self.cursor = pos;
        Ok(())
    }

    pub async fn draw<T: AsyncWrite + Unpin>(&mut self, writer: &mut T) -> Result<()> {
        for i in 0..Self::SIZE {
            let (prev, chr) = (self.previous[i], self.chars[i]);
            if chr == prev {
                continue;
            }

            let pos = Self::from_index(i).unwrap();
            self.move_cursor(pos, writer).await?;

            if chr.inverted != self.inverted {
                self.inverted ^= true;
                writer
                    .write_all([b"\x1Bq", b"\x1Bp"][self.inverted as usize])
                    .await?;
            }

            writer.write_u8(chr.char).await?;
            let idx = Screen::index(self.cursor).unwrap();
            if idx + 1 < Self::SIZE {
                self.cursor = Screen::from_index(idx + 1).unwrap();
            }
        }

        writer.flush().await?;
        self.previous = self.chars.clone();

        Ok(())
    }
}

impl From<&char> for Char {
    fn from(value: &char) -> Self {
        Self {
            char: if value.is_ascii() { *value as u8 } else { b'?' },
            inverted: false,
        }
    }
}

impl From<u8> for Char {
    fn from(value: u8) -> Self {
        Self {
            char: value,
            inverted: false,
        }
    }
}

impl Default for Char {
    fn default() -> Self {
        Self {
            char: b' ',
            inverted: false,
        }
    }
}
