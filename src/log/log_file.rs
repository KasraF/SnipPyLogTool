use std::io::{BufRead, BufReader};
use chrono::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs::File;

#[derive(Debug)]
struct LogFile {
    pub entries: Vec<LogEntry>,
	pub files: HashSet<String>,
	pub synth_calls: Vec<usize>
}

impl LogFile {
    pub fn from_file(file: File) -> Option<LogFile> {
        let mut entries = Vec::new();
		let mut synth_calls = Vec::new();
		let mut files = HashSet::new();
		
        let reader = BufReader::new(file);
        let regex =
            Regex::new(r"(?P<time>\d+),(?P<file>[^,]+),(?P<event>[^,]+),*(?P<content>.*)").unwrap();

        for line in reader.lines() {
            let line = line.unwrap();

            match regex.captures(&line) {
                Some(capture) => {
                    let time = capture
                        .name("time")
                        .unwrap()
                        .as_str()
                        .parse::<i64>()
                        .unwrap();
                    let file = capture.name("file").unwrap().as_str().to_owned();
                    let event = LogEvent::from_str(
                        capture.name("event").unwrap().as_str(),
                        capture.name("content").unwrap().as_str(),
                    );

					let entry = LogEntry {
                        time: Utc.timestamp_millis(time).with_timezone::<Local>(&Local),
						file_name: file.clone(),
                        event: event,
                    };
					
					files.insert(file);
					match entry.event {
						LogEvent::SynthStart{ .. } => synth_calls.push(entries.len()),
						_ => ()
					}
					entries.push(entry);
                }
                None => println!("Line not recognized: {}", line),
            }
        }

        Some(LogFile { entries, synth_calls, files })
    }
}

