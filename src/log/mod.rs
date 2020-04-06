use chrono::prelude::*;
use log_event::LogEvent;
use regex::Regex;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod log_event;

#[derive(Debug)]
pub struct LogEntry {
    pub time: DateTime<Local>,
    pub file_name: String,
    pub event: LogEvent,
}

#[derive(Debug)]
pub struct LogFile {
    pub entries: Vec<LogEntry>,
    pub files: HashSet<String>,

    pub synth_calls: Vec<usize>,
    pub average_examples: f32,
    pub default_vs_custom_focus: f32,
    pub example_changes: u32,
}

impl LogFile {
    pub fn from_file(file: File) -> Option<LogFile> {
        let mut entries = Vec::new();
        let mut synth_calls = Vec::new();
        let mut files = HashSet::new();
        let mut total_examples = 0;
        let mut default_focus = 0;
        let mut custom_focus = 0;
        let mut example_changes = 0;

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
                        LogEvent::SynthStart { example_count, .. } => {
                            synth_calls.push(entries.len());
                            total_examples += example_count;
                        }
                        LogEvent::PBFocusCustom(_) => custom_focus += 1,
                        LogEvent::PBFocusDefault(_) => default_focus += 1,
                        LogEvent::ExampleChanged { .. } => example_changes += 1,
                        _ => (),
                    }
                    entries.push(entry);
                }
                None => println!("Line not recognized: {}", line),
            }
        }

        Some(LogFile {
            average_examples: total_examples as f32 / synth_calls.len() as f32,
            default_vs_custom_focus: default_focus as f32 / custom_focus as f32,
            entries,
            synth_calls,
            files,
            example_changes,
        })
    }

    pub fn summary(self) -> String {
        format!(
			"Total log entries: {}\n\
			 Total synth calls: {}\n\
			 Programming tasks: {:?}\n\
			 Total example changes: {}\n\
			 Default vs. custom focus events: {}",
			self.entries.len(),
			self.synth_calls.len(),
			self.files,
			self.example_changes,
			self.default_vs_custom_focus)
    }
}
