use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{
    close, fork, getpgrp, getpid, getppid, pipe, read, setpgid, setsid, tcsetpgrp, write, ForkResult, Pid
};
use nix::sys::signal::{kill, killpg, sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::fs::File;
use std::{thread};
use std::time::Duration;
use std::os::unix::io::AsRawFd;


extern "C" fn handle_sigint(_: i32){
    println!("Process {} received SIGINT, exiting....", getpid());
    std::process::exit(0);
}

extern "C" fn handle_sigtstp(_:i32){
    println!("Process {} received SIGTSTP (stop), stopping....", getpid());
    //suspend process by sending SIGSTOP to itself
    kill(getpid(), Signal::SIGSTOP).unwrap();
}

extern "C" fn handle_sigcont(_:i32) {
    println!("Process {} received SIGCONT (continue), resuming......", getpid());
}

fn install_sigint_handler() {
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigint),
        SaFlags::empty(),
        SigSet::empty(),
    );

    let sigtstp_action = SigAction::new(
        SigHandler::Handler(handle_sigtstp),
        SaFlags::empty(),
        SigSet::empty(),
    );

    let sigcont_action = SigAction::new(
        SigHandler::Handler(handle_sigcont),
        SaFlags::empty(),
        SigSet::empty(),
    );

    unsafe {
        sigaction(Signal::SIGINT, &sig_action).expect("Failed to set SIGINT handler");
        sigaction(Signal::SIGTSTP, &sigtstp_action).expect("Failed to set SIGTSTP handler");
        sigaction(Signal::SIGCONT, &sigcont_action).expect("Failed to set SIGCONT handler");
    }
}


fn wait_until_foreground(fd: &std::fs::File, target_pgid: nix::unistd::Pid) {
    use nix::unistd::tcgetpgrp;
    use std::{thread, time::Duration};

    loop {
        match tcgetpgrp(fd) {
            Ok(current) if current == target_pgid => break,
            _ => thread::sleep(Duration::from_millis(100)),
        }
    }
}





fn main() {
    // Create a pipe to communicate PGID from child to grandchild
    let (reader, writer) = pipe().expect("Failed to create pipe");

    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            // === CHILD PROCESS ===
            install_sigint_handler();

            // Create new session; child becomes session leader, detached from parent terminal
            let sid = setsid().expect("Failed to create a new session");
            println!("[Child] Created new session with SID: {}", sid);

            let child_pid = getpid();

            // Set child's PGID to its own PID (session leader)
            //  setpgid(child_pid, child_pid).expect("Failed to set PGID for child");


           let child_pgid = getpgrp();

            println!(
                "[Child] PID: {}, PGID: {}, SID: {}, PPID: {}",
                child_pid, child_pgid, sid, getppid()
            );

            // Send PGID to grandchild through pipe
            let pgid_raw = child_pgid.as_raw();
            let pgid_bytes = pgid_raw.to_ne_bytes();
            write(&writer, &pgid_bytes).expect("Child failed to write PGID");
            close(writer.as_raw_fd()).ok();

            match unsafe { fork() } {
                Ok(ForkResult::Child) => {
                    // === GRANDCHILD PROCESS ===
                    install_sigint_handler();

                    let mut buf = [0u8; 4];
                    read(&reader, &mut buf).expect("Grandchild failed to read PGID");
                    close(reader.as_raw_fd()).ok();

                    let pgid_int = i32::from_ne_bytes(buf);
                    let pgid = Pid::from_raw(pgid_int);

                    // join the child's process group
                    setpgid(getpid(), pgid).expect("Grandchild failed to join PGID");

                    let grand_pid = getpid();
                   

                    println!(
                        "[GrandChild] PID: {}, PGID: {}, SID: {}, PPID {}",
                        grand_pid, getpgrp(), sid, getppid()
                    );

                    for i in 1..=10 {
                        println!("Grandchild running... {}", i);
                        thread::sleep(Duration::from_secs(1));
                    }

                    println!("[GrandChild] done.");
                }
                Ok(ForkResult::Parent { child: _grandchild_pid }) => {
                    // === CHILD continues ===
                    for i in 1..=5 {
                        println!("[Child] working... {}", i);
                        thread::sleep(Duration::from_secs(1));
                    }
                    println!("[Child] done.");
                }
                Err(e) => {
                    eprintln!("Failed to fork grandchild: {}", e);
                }
            }
        }

       Ok(ForkResult::Parent { child }) => {
    // === PARENT PROCESS ===
    let parent_pid = getpid();
    println!("[Parent] PID: {}, child PID: {}", parent_pid, child);

    // Open controlling terminal to get a valid FD
    let tty = File::open("/dev/tty").expect("Failed to open /dev/tty");

    // Give terminal control to the child process group
    tcsetpgrp(&tty, child).expect("Failed to set terminal foreground process group");

    // Wait for child and grandchild to set up process groups
    thread::sleep(Duration::from_secs(3));

    println!("[Parent] Sending SIGINT to process group -{}", child);
    kill(Pid::from_raw(-child.as_raw()), Signal::SIGINT)
        .expect("Failed to send SIGINT");

    println!("[Parent] Sending SIGTSTP to process group -{}", child);
    kill(Pid::from_raw(-child.as_raw()), Signal::SIGTSTP)
        .expect("Failed to send SIGTSTP");

    // Reclaim terminal foreground control
    let tty = File::open("/dev/tty").expect("Failed to reopen /dev/tty");
    tcsetpgrp(&tty, parent_pid).expect("Parent failed to regain terminal control");

    thread::sleep(Duration::from_secs(5));

    // Wait to see if child has stopped
    match waitpid(child, Some(WaitPidFlag::WUNTRACED)) {
        Ok(WaitStatus::Stopped(pid, sig)) => {
            println!("[Parent] Child {} stopped by signal {}", pid, sig);

            // Take terminal control (if not already done)
            let tty = File::open("/dev/tty").expect("Failed to reopen /dev/tty");
            tcsetpgrp(&tty, parent_pid).expect("Parent failed to take terminal control back");
            println!("[Parent] Terminal has been restored to parent");

            // Wait for user to continue (like pressing 'fg')
            println!("[Parent] Press ENTER to resume child process group...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            // Wait until this parent process regains terminal control before giving it to child
             wait_until_foreground(&tty, getpgrp());  

            // Resume child process group
            killpg(Pid::from_raw(child.as_raw()), Signal::SIGCONT)
                .expect("Failed to send SIGCONT");

            // Give terminal back to child process group
            tcsetpgrp(&tty, child).expect("Failed to give terminal to child PGID");

            println!("[Parent] Resumed child process group in foreground");

            // Final wait to let child finish execution
            match waitpid(child, None) {
                Ok(WaitStatus::Exited(pid, code)) => {
                    println!("[Parent] Child {} exited with status {}", pid, code);
                }
                Ok(WaitStatus::Signaled(pid, signal, _)) => {
                    println!("[Parent] Child {} terminated by signal {:?}", pid, signal);
                }
                Ok(status) => {
                    println!("[Parent] Received unexpected final status: {:?}", status);
                }
                Err(e) => {
                    eprintln!("[Parent] Final waitpid failed: {}", e);
                }
            }
        }
        Ok(WaitStatus::Exited(pid, code)) => {
            println!("[Parent] Child {} exited with status {}", pid, code);
        }
        Ok(WaitStatus::Signaled(pid, signal, _)) => {
            println!("[Parent] Child {} terminated by signal {:?}", pid, signal);
        }
        Ok(status) => {
            println!("[Parent] Received unexpected status: {:?}", status);
        }
        Err(e) => {
            eprintln!("waitpid failed: {}", e);
        }
    }

    println!("[Parent] done.");
}


        Err(e) => {
            eprintln!("Initial fork failed: {}", e);
        }
    }
}
