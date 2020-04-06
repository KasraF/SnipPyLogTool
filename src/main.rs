use crate::log::LogFile;
use std::fs::File;
use std::path::Path;

mod log;

fn main() -> Result<(), std::io::Error> {
    let file = get_log()?;
    let log = LogFile::from_file(file).unwrap();

	println!("{}", log.summary());
	
    Ok(())
}

fn get_log() -> Result<File, std::io::Error> {
    let path_str = std::env::args().nth(1);
    let path_str = match path_str {
        Some(p) => p,
        None => ".".to_owned(),
    };

    let path = Path::new(&path_str);
    let log_path = if path.is_file() {
        path.to_path_buf()
    } else {
        path.join(Path::new("snippy.log"))
    };

    let log_path = log_path.as_path();
    File::open(log_path)
}
