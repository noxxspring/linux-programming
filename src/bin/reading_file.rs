use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::read;

fn main() {
    // Open the file - returns OwnedFd
    let file = open("/etc/hostname", OFlag::O_RDONLY, Mode::empty())
        .expect("Failed to open file");
    
    let mut buffer = [0u8; 128];
    // Use as_raw_fd() to get the raw descriptor when needed
    let bytes_read = read(file, &mut buffer)
        .expect("Failed to read file");
    
    println!("Read {} bytes: {}", 
        bytes_read, 
        String::from_utf8_lossy(&buffer[..bytes_read]));
    
    // No need to explicitly close - OwnedFd implements Drop
}