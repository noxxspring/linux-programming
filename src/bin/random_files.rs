use std::{env, fs::{self, File}, io::{self, Write}, path::PathBuf, time::Instant};

use rand::Rng;


fn generate_random_filenames(n: usize) -> Vec<String> {
    let mut filenames = Vec::new();
    let mut rng = rand::thread_rng();

    while filenames.len() < n {
        let num = rng.gen_range(0..1_000_1000);
        let name = format!("x{:06}", num);
        if !filenames.contains(&name) {
        filenames.push(name);
    }
}


    filenames
}


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <num_files> <target_directory>", args[0]);
        std::process::exit(1);
    }
    
    let num_files: usize = args[1].parse().expect("invalid number");
    let target_dir = PathBuf::from(&args[2]);

    //Ensure directory exists 
    fs::create_dir(&target_dir)?;

    //Generate unique random filenames
    let filenames = generate_random_filenames(num_files);

    //start timer creation
    let start_timer = Instant::now();

    for filename in &filenames {
        let path = target_dir.join(filename);
        let mut file = File::create(&path)?;
        file.write_all(&[0u8])?;
    }

    let duration_create = start_timer.elapsed();
    println!("created {} files in {:?}", num_files, duration_create);

    //sort filenames for deletion
    let mut sorted_filenames = filenames.clone();
    sorted_filenames.sort();

    //start timer deletio
    let start_delete = Instant::now();
    
    for filename in &sorted_filenames {
        let path = target_dir.join(filename);
        fs::remove_file(&path)?;
    }

    let duration_delete = start_delete.elapsed();
    println!("Deleted {} files {:?}", num_files, duration_delete);

    Ok(())
}