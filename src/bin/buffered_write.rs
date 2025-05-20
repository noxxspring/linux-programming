use std::{fs::File, io::{BufWriter, Write}};

fn main() -> std::io::Result<()> {
    let file = File::create("buffered.txt")?;
    let mut writer = BufWriter::new(file);

    let chunk = vec![b'A'; 8192];

    for _ in 0..1000 {
        writer.write_all(&chunk)?;
    }

    writer.flush()?;
    Ok(())
    
}