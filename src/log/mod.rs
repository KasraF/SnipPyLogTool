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
		let mut starts = self.synth_calls.iter().map(|e| &self.entries[*e]);
		let mut idx_file_duration_lineno_result = Vec::with_capacity(starts.len());
		let mut successes = self.synth_calls.len();

		for entry in &self.entries {
			match &entry.event {
				LogEvent::SynthEnd {index, exit_code, result} => {
					let start = starts.find(|x| x.event.index().unwrap() == *index).unwrap();
					let duration = entry.time.signed_duration_since(start.time).num_milliseconds() as f32 / 1000.;
					
					if *exit_code == 0 {
						idx_file_duration_lineno_result.push((index, entry.file_name.clone(), duration, start.event.lineno().unwrap(), result.clone()));
					} else {
						idx_file_duration_lineno_result.push((index, entry.file_name.clone(), duration, start.event.lineno().unwrap(), format!("Exit code {}", *exit_code)));
						successes -= 1;
					};
				},
				_ => ()
			}
		}		
		
		let mut total_synth_duration = 0.;
		let mut max_synth_duration = 0.;
		let mut min_synth_duration = 10000.;

		for line in &idx_file_duration_lineno_result {
			let dur = &line.2;
			total_synth_duration += *dur;

			if dur > &max_synth_duration {
				max_synth_duration = *dur;
			}
			if dur < &min_synth_duration {
				min_synth_duration = *dur;
			}
		}

		let average_synth_duration = total_synth_duration as f32 / idx_file_duration_lineno_result.len() as f32;
		
        let mut rs = format!(
			"SnipPy log results:\n\
			 ---------------------\n\
			 Total log entries: {}\n\
			 Programming tasks: {:?}\n\
			 Total example changes: {}\n\
			 Default vs. custom focus events: {}\n\
			 ---------------------\n\n\
			 Synthesizer results:\n\
			 ---------------------\n\
			 Total synth calls: {}\n\
			 Successful synth calls: {}\n\
			 Average synth duration: {:.3}\n\
			 Min     synth duration: {}\n\
			 Max     synth duration: {}\n\
			 ---------------------\n\n\
			 Synthesizer calls:\n\
			 ----- ---------------- -------- -------- -------------------------------------------------\n\
			 Index File             Duration Line No. Result\n\
			 ----- ---------------- -------- -------- -------------------------------------------------\n",
			self.entries.len(),
			self.files,
			self.example_changes,
			self.default_vs_custom_focus,
			self.synth_calls.len(),
			successes,
			average_synth_duration,
			min_synth_duration,
			max_synth_duration
		);

		for line in idx_file_duration_lineno_result {
			rs.push_str(&format!("{:5} {:16} {:<8} {:<8} {}\n", line.0, line.1, line.2, line.3, line.4));
		}
		
		rs
    }
}
