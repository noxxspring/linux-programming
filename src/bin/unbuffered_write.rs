use std::{fs::OpenOptions, io::Write};

fn main() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open("no_buffer.txt")?;

    for _ in 0..100_000_000 {
        file.write_all(b"A")?;
    }
    
    Ok(())
}