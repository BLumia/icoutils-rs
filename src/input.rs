use std::{
    fs,
    io::{self, Read},
};

pub fn read_input(name: &str) -> io::Result<Vec<u8>> {
    if name == "-" {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        fs::read(name)
    }
}
