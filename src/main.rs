use crate::log::LogFile;
use std::fs::DirEntry;
use std::fs::File;
use std::path::Path;
use std::env::args;

mod log;

fn main() -> Result<(), std::io::Error> {
    let mut args = args();
    args.next();
    let files = match args.next() {
	Some(arg) => get_logs(Path::new(&arg))?,
	None => get_logs(Path::new("."))?,
    };
    
    let logs: Vec<LogFile> = files
        .into_iter()
        .filter_map(|f| LogFile::from_file(f))
	.collect();

    // Print Summaries
    logs.iter()
        .map(|l| l.summary())
        .for_each(|s| println!("{}", s));

    // Sum examples
    Ok(())
}

fn get_logs(path: &Path) -> Result<Vec<File>, std::io::Error> {
    let mut rs;

    if path.is_file() {
	rs = vec!(File::open(path)?);
    } else {
	rs = Vec::new();
	let curr_dir = path
            .read_dir()?
            .filter_map(|e| e.ok())
            .collect::<Vec<DirEntry>>();

	for entry in &curr_dir {
            let p_buf = entry.path();
            let p = p_buf.as_path();

            if p.is_dir() {
		rs.append(&mut (get_logs(p)?));
            } else if p
		.file_name()
		.expect(&format!("Failed to read file name for path: {:?}", p))
		.eq("snippy.log")
            {
		let f = File::open(p)?;
		rs.push(f);
            }
	}
    }

    Ok(rs)
}
