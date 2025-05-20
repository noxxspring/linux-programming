use std::{ffi::CString, ptr};

use libc::{close, open, posix_memalign, O_CREAT, O_DIRECT, O_RDWR, S_IRUSR, S_IWUSR};
fn main() {
    // 4096-byte alignment
    const ALIGNMENT: usize = 4096;
    const SIZE: usize = 4096;

    //Allocate the aligned buffer
    let mut buf: *mut u8 = ptr::null_mut();
    unsafe {
        if posix_memalign(&mut buf as *mut *mut u8 as *mut _, ALIGNMENT, SIZE) != 0 {
            panic!("posix_memalign failed");
        }

        // fill the buffer with A
        ptr::write_bytes(buf, b'A', SIZE);
    }

    // open the fle with O_DIRECT
    let path = CString::new("direct_io.txt").unwrap();
    let fd = unsafe {
        open(
            path.as_ptr(),
        O_CREAT | O_RDWR | O_DIRECT,
               S_IRUSR | S_IWUSR,)
    };

    if fd < 0 {
        panic!("failed to open with O_DIRECT");
    }

    // write to file directly (using the aligned buffer)
    let written = unsafe {
        libc::write(fd, buf as *const _, SIZE)
    };

    if written != SIZE as isize {
        eprintln!("Failed to write all data. wrote: {}", written);
    }

    unsafe {
        close(fd);
        libc::free(buf as *mut _);
    }
    println!("Wrote {} bytes using Direct I/O", written)
    
}