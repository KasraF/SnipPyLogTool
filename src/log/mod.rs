use chrono::prelude::*;
use log_event::LogEvent;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use zip_option::GetZipOption;

mod log_event;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Task {
    Abbreviate,
    CountDuplicates,
    MaxAndMin,
    Palindrome
}

impl Task {
    pub fn from(s: &str) -> Option<Self> {
	if s.contains("abbreviate") {
	    Some(Self::Abbreviate)
	} else if s.contains("count") {
	    Some(Self::CountDuplicates)
	} else if s.contains("max") || s.contains("min") {
	    Some(Self::MaxAndMin)
	} else if s.contains("palindrome") {
	    Some(Self::Palindrome)
	} else {
	    None
	}
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
	    Self::Abbreviate => "Abbreviate",
	    Self::CountDuplicates => "Count Duplicates",
	    Self::MaxAndMin => "Max And Min",
	    Self::Palindrome => "Palindrome",
	};

	write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub time: DateTime<Local>,
    pub file_name: String,
    pub event: LogEvent,
}

#[derive(Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
    pub synth_calls: Vec<usize>,
}

impl Log {
    pub fn new() -> Self {
	Log {
	    entries: Vec::new(),
	    synth_calls: Vec::new(),
	}
    }

    pub fn with(e: LogEntry) -> Self {
	let mut log = Self::new();
	log.push(e);
	log
    }

    pub fn push(&mut self, e: LogEntry) {
	// match e.event {
        //     LogEvent::SynthStart { example_count, .. } => {
	// 	self.average_examples = (self.average_examples * self.synth_calls.len() as f32 + example_count as f32) / (self.synth_calls.len() as f32 + 1.0);
        //         self.synth_calls.push(self.entries.len());                
        //     }
        //     LogEvent::PBFocusCustom(_) => self.custom_focus += 1,
        //     LogEvent::PBFocusDefault(_) => self.default_focus += 1,
        //     LogEventLogEvent::ExampleChanged { .. } => self.example_changes += 1,
        //     _ => (),
        // }

        self.entries.push(e);
    }

    pub fn synth_call_entries(&self) -> Vec<&LogEntry> {
	self.synth_calls.iter().map(|idx| &self.entries[*idx]).collect()
    }
}

#[derive(Debug)]
pub struct LogFile {
    pub entry_map: HashMap<Task, Log>
}

impl LogFile {
    pub fn from_file(file: File) -> Option<LogFile> {	
        let mut entry_map: HashMap<Task, Log> = HashMap::new();

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
                        event,
                    };

		    match Task::from(&file) {
			Some(task) =>  {
			    match entry_map.get_mut(&task) {
				Some(log) => log.push(entry),
				None => {
				    entry_map.insert(task, Log::with(entry));
				}
			    }
			},
			None => ()
		    }
                }
                None => println!("Line not recognized: {}", line),
            }
        }

        Some(LogFile { entry_map })
    }

    pub fn summary(&self) -> String {
	let mut rs = String::from("File,Synth Calls,Failed,Max Time,Min Time,AverageTime,Average Examples\n");
	let mut examples: HashMap<&Task, Vec<u32>> = HashMap::new();
	
	for (task_name, log) in &self.entry_map {
	    examples.insert(task_name.clone(), Vec::new());
	    let mut synth_calls = 0;
	    let mut failed = 0;
	    let mut max_time = 0.0;
	    let mut min_time = f32::MAX;
	    let mut total_time = 0.0;
	    let mut total_examples = 0;

	    let mut current_synth: Option<&DateTime<Local>> = None;

	    for entry in &log.entries {
		match entry.event {
		    LogEvent::SynthStart { example_count, .. } => {
			synth_calls += 1;
			current_synth = Some(&entry.time);
			total_examples += example_count;
			examples.get_mut(task_name).unwrap().push(example_count);
		    },
		    LogEvent::SynthEnd { exit_code, index, .. } => {

			if exit_code != 0 {
			    failed += 1;
			}
			
			let start_time = current_synth.expect(&format!("Failed to find matching start synth call for index {}", index));
			let duration = entry.time.signed_duration_since(*start_time).num_milliseconds() as f32 / 1000.0;

			total_time += duration;

			if duration < min_time {
			    min_time = duration;
			}

			if duration > max_time {
			    max_time = duration;
			}
			
		    }
		    _ => ()
		}
	    }

	    rs.push_str(&format!("{},{},{},{},{},{},{}\n", task_name, synth_calls, failed, max_time, min_time, total_time / synth_calls as f32, total_examples as f32/ synth_calls as f32));
	}

	rs.push_str("\n\n");

	// Print the examples for each synth call
	let mut i = examples.iter();
	match i.next() {
	    Some(first) => match i.next() {
		Some(second) => {
		    rs.push_str(&format!("{},{}\n", first.0, second.0));
		    for examples in first.1.iter().zip_option(second.1.iter()) {
			match examples {
			    (Some(fst), Some(snd)) => rs.push_str(&format!("{},{}\n", fst, snd)),
			    (Some(fst), None) => rs.push_str(&format!("{},\n", fst)),
			    (None, Some(snd)) => rs.push_str(&format!(",{}\n", snd)),
			    _ => ()
			}
		    }
		},
		None => {
		    rs.push_str(&first.0.to_string());
		    for ex in first.1.iter() {
			rs.push_str(&ex.to_string());
		    }
		},
	    }
	    None => ()
	}
	
	rs
    }
}
