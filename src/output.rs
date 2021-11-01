use std::io::{Result as IoResult, Write};

pub struct Output(Vec<u8>);

impl Output {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn flush_to(&mut self, stdout: &mut impl Write) {
        stdout.write_all(&self.0).unwrap();
        stdout.flush().unwrap();

        self.0.clear();
    }
}

impl Write for Output {
    fn write(&mut self, data: &[u8]) -> IoResult<usize> {
        self.0.extend_from_slice(data);

        Ok(data.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        let mut stdout = std::io::stdout();

        stdout.write_all(&self.0)?;
        stdout.flush()
    }
}
