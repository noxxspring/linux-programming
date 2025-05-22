use std::{io::{BufRead, BufReader}, process::{Command, Stdio}};

use regex::Regex;
use syslog::{ Facility, Formatter3164};

fn main() -> std::io::Result<()> {
    //regex to match faled login attempts
    let fail_regex = Regex::new(r"(?i)failed password|authentication failure|invalid user").unwrap();


    // Setup syslog logger
    let formatter = Formatter3164 {
        facility: Facility::LOG_AUTH,  // Auth-related logs
        hostname: None,
        process: "auth-logger".into(),
        pid: std::process::id(),
    };

    let mut logger = syslog::unix(formatter).expect("could not connect to syslog");

    //Spawn journalctl to follow all logs
    let mut child = Command::new("journalctl")
    .args(&["-f", "-n", "0"]) // -f: follow; -n 0: dont show old files
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to spawn journalctl");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);


    println!("[*] Monitoring system logs for failed authentication attempts....\n");

    // Loop and process lines
    for line in reader.lines(){
        let line = line?;
        if fail_regex.is_match(&line){

            //Avoid logging our own syslog messages
            if line.contains("auth-logger"){
                continue;
            }
            println!("[!] Failed auth detected: {}", &line);
            // log to syslog asinfo level
            logger.info(&line)
            .unwrap_or_else(|e| eprintln!("Failed to send to syslog: {}", e));
        }
    }
    Ok(())
    
}