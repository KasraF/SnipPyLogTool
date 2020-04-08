#[derive(Debug)]
pub enum LogEvent {
    SynthStart {
        index: usize,
        line_no: u32,
        example_count: u32,
    },
    SynthOut(String),
    SynthErr(String),
    SynthEnd {
        index: usize,
        exit_code: i32,
        result: String,
    },
    PBFocusDefault(String),
    PBFocusCustom(String),
    PBExit,
    ExampleBlur {
        index: usize,
        content: String,
    },
    ExampleFocus {
        index: usize,
        content: String,
    },
    ExampleChanged {
        index: usize,
        before: String,
        after: String,
    },
    ExampleInclude {
        index: usize,
        content: String,
    },
    ExampleExclude {
        index: usize,
        content: String,
    },
    ExampleReset,
    Unknown(String),
}

impl LogEvent {
	pub fn index(&self) -> Option<usize> {
		match self {
			LogEvent::SynthStart { index, ..} => Some(*index),
			LogEvent::SynthEnd { index, ..} => Some(*index),
			LogEvent::ExampleBlur { index, ..} => Some(*index),
			LogEvent::ExampleFocus { index, ..} => Some(*index),
			LogEvent::ExampleChanged { index, ..} => Some(*index),
			LogEvent::ExampleInclude { index, ..} => Some(*index),
			LogEvent::ExampleExclude { index, ..} => Some(*index),
			_ => None
		}
	}

	pub fn lineno(&self) -> Option<u32> {
		match self {
			LogEvent::SynthStart { line_no, ..} => Some(*line_no),
			_ => None
		}
	}
	
    // TODO This is really ugly.
    pub fn from_str(event_str: &str, content: &str) -> LogEvent {
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
                        let index = split.next().unwrap().parse::<usize>().unwrap();
                        let line_no = split.next().unwrap().parse::<u32>().unwrap();
                        let example_count = split.next().unwrap().parse::<u32>().unwrap();
                        LogEvent::SynthStart {
                            index,
                            line_no,
                            example_count,
                        }
                    }
                    Some("end") => {
                        let index = split.next().unwrap().parse::<usize>().unwrap();
                        let exit_code = split.next().unwrap().parse::<i32>().unwrap();
                        LogEvent::SynthEnd {
                            index,
                            exit_code,
                            result: content.to_owned(),
                        }
                    }
                    Some("stdout") => LogEvent::SynthOut(content.to_owned()),
                    Some("sterr") => LogEvent::SynthErr(content.to_owned()),
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
                    {
                        let index = split.next().unwrap().parse::<usize>().unwrap();
                        let subtype = split.next();
                        match subtype {
                            Some("blur") => LogEvent::ExampleBlur { index, content: content.to_owned() },
                            Some("focus") => LogEvent::ExampleFocus { index, content: content.to_owned() },
                            _ => LogEvent::Unknown(event_str.to_owned())
                        }
                    }
                    _ => LogEvent::Unknown(event_str.to_owned()),
                }
            }
            /*
            example.${idx}.change
            example.${idx}.include
            example.${idx}.exclude
            example.all.reset
            */
            Some("example") =>
            {
                let index_or_all = split.next();
                match index_or_all {
                    Some("all") => match split.next() {
                        Some("reset") => LogEvent::ExampleReset,
                        _ => LogEvent::Unknown(event_str.to_owned())
                    }
                    Some(index_str) => match index_str.parse::<usize>() {
                        Ok(index) => match split.next() {
                            Some("change") => {
                                let mut before_after = content.split(',');
                                let before = before_after.next().unwrap().to_owned();
                                let after = before_after.next().unwrap().to_owned();
                                LogEvent::ExampleChanged { index, before, after }
                            }
                            Some("include") => LogEvent::ExampleInclude { index, content: content.to_owned() },
							Some("exclude") => LogEvent::ExampleExclude { index, content: content.to_owned() },
                            _ => LogEvent::Unknown(event_str.to_owned())
                        },
						Err(_) => LogEvent::Unknown(event_str.to_owned())
                    }
                    None => LogEvent::Unknown(event_str.to_owned())
                }
            }
            _ => LogEvent::Unknown(event_str.to_owned()),
        }
    }
}

