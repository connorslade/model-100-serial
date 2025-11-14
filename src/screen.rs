use std::io::Write;

use anyhow::Result;
use nd_vec::Vec2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Char {
    char: u8,
    inverted: bool,
}

pub struct Screen {
    previous: [Char; Self::WIDTH * Self::HEIGHT],
    cursor: Vec2<usize>,
    inverted: bool,

    chars: [Char; Self::WIDTH * Self::HEIGHT],
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

    fn index(pos: Vec2<usize>) -> Option<usize> {
        (pos.x() < Self::WIDTH && pos.y() < Self::HEIGHT).then(|| pos.x() + pos.y() * Self::WIDTH)
    }

    fn from_index(index: usize) -> Option<Vec2<usize>> {
        (index < Self::SIZE).then(|| Vec2::new([index % Self::WIDTH, index / Self::WIDTH]))
    }
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            previous: [Char::default(); Self::SIZE],
            cursor: Vec2::zero(),
            inverted: false,

            chars: [Char::default(); Self::SIZE],
        }
    }

    pub fn put(&mut self, pos: Vec2<usize>, chr: Char) {
        if let Some(index) = Screen::index(pos) {
            self.chars[index] = chr;
        }
    }

    pub fn clear(&mut self) {
        self.chars.fill(Char::default());
    }

    pub fn write_string(&mut self, pos: Vec2<usize>, str: &[u8]) {
        for (i, chr) in str.iter().enumerate() {
            self.put(pos + Vec2::new([i, 0]), (*chr).into());
        }
    }

    pub fn write_string_wrapped(&mut self, pos: Vec2<usize>, str: &[u8], width: usize) {
        let mut offset = Vec2::zero();
        for word in str.split(|x| *x == b' ') {
            if offset.x() + word.len() > width {
                offset = Vec2::new([0, offset.y() + 1]);
            }

            for (i, chr) in word.iter().enumerate() {
                self.put(pos + offset + Vec2::new([i, 0]), (*chr).into());
            }

            offset += Vec2::new([word.len() + 1, 0]);
        }
    }

    pub fn rect(&mut self, pos: Vec2<usize>, size: Vec2<usize>, chr: Char) {
        for y in 0..size.y() {
            for x in 0..size.x() {
                self.put(pos + Vec2::new([x, y]), chr);
            }
        }
    }
}

impl Screen {
    fn move_cursor<T: Write>(&mut self, pos: Vec2<usize>, writer: &mut T) -> Result<()> {
        let dx = pos.x() as isize - self.cursor.x() as isize;
        let dy = pos.y() as isize - self.cursor.y() as isize;

        let horizontal_mover = [b"\x1BD", b"\x1BC"][(dx > 0) as usize];
        let vertical_mover = [b"\x1BA", b"\x1BB"][(dy > 0) as usize];

        for _ in 0..dx.abs() {
            writer.write_all(horizontal_mover)?;
        }

        for _ in 0..dy.abs() {
            writer.write_all(vertical_mover)?;
        }

        self.cursor = pos;
        Ok(())
    }

    pub fn draw<T: Write>(&mut self, writer: &mut T) -> Result<()> {
        for i in 0..Self::SIZE {
            let (prev, chr) = (self.previous[i], self.chars[i]);
            if chr == prev {
                continue;
            }

            let pos = Self::from_index(i).unwrap();
            self.move_cursor(pos, writer)?;

            if chr.inverted != self.inverted {
                self.inverted ^= true;
                writer.write_all([b"\x1Bq", b"\x1Bp"][self.inverted as usize])?;
            }

            writer.write_all(&[chr.char])?;
            let idx = Screen::index(self.cursor).unwrap();
            if idx + 1 < Self::SIZE {
                self.cursor = Screen::from_index(idx + 1).unwrap();
            }
        }

        writer.flush()?;
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
