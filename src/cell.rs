use draw::Draw;
use std::io::{self, Write};
use termion::color;

#[derive(Clone, Copy)]
pub struct Cell(pub bool);

impl Cell {
    pub fn oppo(&mut self) {
        self.0 = !self.0;
    }
}

impl Draw for Cell {
    type Ret = ();
    type Args = usize;

    fn draw(&self, writer: &mut impl Write) -> io::Result<()> {
        if self.0 {
            writer.write(&color::Bg(color::White).to_string().into_bytes())?;
        } else {
            writer.write(&color::Bg(color::Black).to_string().into_bytes())?;
        }

        writer.write(b" ")?;

        Ok(())
    }

    fn update(&mut self, neighbors: usize) {
        if self.0 && neighbors == 2 {
            return;
        }

        if neighbors == 3 {
            self.0 = true;
            return;
        }

        self.0 = false;
    }
}
