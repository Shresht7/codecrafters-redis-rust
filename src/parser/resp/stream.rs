use std::collections::HashMap;

pub struct StreamID {
    pub milliseconds: u64,
    pub sequence: u64,
}

impl Default for StreamID {
    fn default() -> Self {
        StreamID {
            milliseconds: get_unix_timestamp(),
            sequence: 0,
        }
    }
}

impl StreamID {
    pub fn from_parts(milliseconds: u64, sequence: u64) -> StreamID {
        StreamID {
            milliseconds,
            sequence,
        }
    }

    pub fn from_id(id: &str) -> StreamID {
        match id.split_once("-") {
            Some((milliseconds, sequence)) => {
                let milliseconds = milliseconds.parse::<u64>().unwrap_or(0);
                let sequence = sequence.parse::<u64>().unwrap_or(0);
                StreamID {
                    milliseconds,
                    sequence,
                }
            }
            None => StreamID {
                milliseconds: 0,
                sequence: 0,
            },
        }
    }

    pub fn parse(id: &str, last_entry: Option<(String, HashMap<String, String>)>) -> StreamID {
        let timestamp = get_unix_timestamp();
        match id {
            "*" => {
                if let Some((last_id, _)) = last_entry {
                    let last = StreamID::from_id(&last_id);
                    if timestamp == last.milliseconds {
                        StreamID {
                            milliseconds: timestamp,
                            sequence: last.sequence + 1,
                        }
                    } else {
                        StreamID::default()
                    }
                } else {
                    StreamID::default()
                }
            }
            _ => match id.split_once("-") {
                Some((milliseconds, sequence)) => {
                    let milliseconds = parse_milliseconds(milliseconds);
                    let sequence = parse_sequence(sequence, milliseconds, last_entry);
                    StreamID {
                        milliseconds,
                        sequence,
                    }
                }
                None => StreamID::default(),
            },
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}-{}", self.milliseconds, self.sequence)
    }

    pub fn next(&self) -> StreamID {
        StreamID {
            milliseconds: self.milliseconds,
            sequence: self.sequence + 1,
        }
    }
}

fn parse_milliseconds(milliseconds: &str) -> u64 {
    match milliseconds {
        "*" => get_unix_timestamp(),
        _ => milliseconds.parse::<u64>().unwrap_or(0),
    }
}

fn parse_sequence(
    sequence: &str,
    milliseconds: u64,
    last_entry: Option<(String, HashMap<String, String>)>,
) -> u64 {
    match sequence {
        "*" => {
            if let Some((last_id, _)) = last_entry {
                let last = StreamID::from_id(&last_id);
                if milliseconds == last.milliseconds {
                    last.sequence + 1
                } else {
                    0
                }
            } else {
                1
            }
        }
        _ => sequence.parse::<u64>().unwrap_or(0),
    }
}

// TODO: There is a duplicate of this somewhere else in the codebase, I think. Refactor to use a common function.
/// Returns the current Unix timestamp in milliseconds.
fn get_unix_timestamp() -> u64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
