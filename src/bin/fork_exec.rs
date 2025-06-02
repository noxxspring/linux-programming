use std::{collections::HashSet, ffi::CString};

use caps::{CapSet, Capability};
use nix::{sys::wait::waitpid, unistd::{execve, fork, ForkResult}};

fn main() {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!(" Parent: Forked child with PID {}", child);

            let _ = waitpid(child, None).unwrap();
            println!("Parent: Child has finished.");
        }

        Ok(ForkResult::Child) => {
            println!(" Child: Setting capability CAP_NET_RAW");

            //give the child the CAP_NET_RAWin the permitted * effective set
            let mut cap_set = HashSet::new();
            cap_set.insert(Capability::CAP_NET_RAW);

            caps::set(None, CapSet::Effective, &cap_set).unwrap();

            // check that i has actually set 
            let has_cap = caps::has_cap(None, CapSet::Effective, Capability::CAP_NET_RAW ).unwrap();
            println!(" Child: CAP_NET_RAW is set: {}", has_cap);



            let path = CString::new("/bin/ls").unwrap();
            let args = [
                CString::new("ls").unwrap(),
                CString::new("-l").unwrap(),
            ];

            let env = [CString::new("PATH=/bin").unwrap()];

            // if execve is successful, this process becomes ls, and nothing below wil run
            match execve(&path, &args, &env) {
                Ok(_)=> {}
                Err(err) => {
                    eprintln!("execve failed: {}", err);
                }
                
            }
        }

        Err(err) => {
            eprintln!("Forked failed: {}", err);
        }
        
    }
    
}