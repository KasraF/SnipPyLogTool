use chrono::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
enum LogEvent {
    SynthStart {
        index: u32,
        line_no: u32,
        example_count: u32,
    },
    SynthOut(String),
    SynthErr(String),
    SynthEnd {
        index: u32,
        exit_code: i32,
        result: String,
    },
    PBFocusDefault(String),
    PBFocusCustom(String),
    PBExit,
    ExampleBlur {
        index: u32,
        content: String,
    },
    ExampleFocus {
        index: u32,
        content: String,
    },
    ExampleChanged {
        index: u32,
        before: String,
        after: String,
    },
    ExampleInclude {
        index: u32,
        content: String,
    },
    ExampleExclude {
        index: u32,
        content: String,
    },
    ExampleReset,
    Unknown(String),
}

impl LogEvent {
    pub fn from_str(event_str: &str, content: &str) -> LogEvent {
        /*
        List of possible events:
        example.${idx}.change
        example.${idx}.include
        example.${idx}.exclude
        example.all.reset
         */
        let mut split = event_str.split('.');
        let etype = split.next();

        match etype {
            Some("synth") => {
                /*
                synth.start.${this.synthRequestCounter}.${lineno}.${examples}
                synth.stdout
                synth.sterr
                synth.end.${this.synthRequestCounter - 1}.${exitCode}
                 */
                let subtype = split.next();
                match subtype {
                    Some("start") => {
                        let index = split.next().unwrap().parse::<u32>().unwrap();
                        let line_no = split.next().unwrap().parse::<u32>().unwrap();
                        let example_count = split.next().unwrap().parse::<u32>().unwrap();
                        LogEvent::SynthStart {
                            index,
                            line_no,
                            example_count,
                        }
                    }
                    Some("end") => {
                        let index = split.next().unwrap().parse::<u32>().unwrap();
                        let exit_code = split.next().unwrap().parse::<i32>().unwrap();
                        LogEvent::SynthEnd {
                            index,
                            exit_code,
                            result: content.to_owned(),
                        }
                    }
                    Some("stdout") => LogEvent::SynthOut(content.to_owned()),
                    Some("stderr") => LogEvent::SynthErr(content.to_owned()),
                    _ => LogEvent::Unknown(event_str.to_owned()),
                }
            }
            Some("focus") => {
                /*
                focus.projectionBox.focus.custom
                focus.projectionBox.focus.default
                focus.projectionBox.exit
                focus.example.${idx}.blur
                focus.example.${idx}.focus
                 */
                let subtype = split.next();
                match subtype {
                    Some("projectionBox") => {
                        let subsubtype = split.next();
                        match subsubtype {
                            Some("focus") => match split.next() {
                                Some("custom") => LogEvent::PBFocusCustom(content.to_owned()),
                                Some("default") => LogEvent::PBFocusDefault(content.to_owned()),
                                _ => LogEvent::Unknown(event_str.to_owned()),
                            },
                            Some("exit") => LogEvent::PBExit,
                            _ => LogEvent::Unknown(event_str.to_owned()),
                        }
                    }
                    Some("example") =>
                    // TODO
                    {
                        LogEvent::Unknown(event_str.to_owned())
                    }
                    _ => LogEvent::Unknown(event_str.to_owned()),
                }
            }
            Some("example") =>
            // TODO
            {
                LogEvent::Unknown(event_str.to_owned())
            }
            _ => LogEvent::Unknown(event_str.to_owned()),
        }
    }
}

#[derive(Debug)]
struct LogEntry {
    time: DateTime<Local>,
    file_name: String,
    event: LogEvent,
}

#[derive(Debug)]
struct LogFile {
    pub entries: Vec<LogEntry>,
}

impl LogFile {
    pub fn from_file(file: File) -> Option<LogFile> {
        let mut entries = Vec::new();
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
                    let file = capture.name("file").unwrap().as_str();
                    let event = LogEvent::from_str(
                        capture.name("event").unwrap().as_str(),
                        capture.name("content").unwrap().as_str(),
                    );

                    entries.push(LogEntry {
                        time: Utc.timestamp_millis(time).with_timezone::<Local>(&Local),
                        file_name: file.to_owned(),
                        event: event,
                    })
                }
                None => println!("Line not recognized: {}", line),
            }
        }

        Some(LogFile { entries })
    }
}

fn main() -> Result<(), std::io::Error> {
    let file = get_log()?;
    let log = LogFile::from_file(file).unwrap();

    for entry in log.entries {
        println!("{:?}", entry);
    }

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
