/// This module provides functionality for parsing telemetry log (TLOG) files
/// using the MAVLink protocol. It defines the `TlogParser` struct, which
/// implements the `MavParser` trait to read and process MAVLink messages
/// from a TLOG file.
/// See /docs/tlog_file_format.md for more information on the TLOG file format.
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};

use mavlink::error::MessageReadError;
use mavlink::peek_reader::PeekReader;
use mavlink::{MavlinkVersion, Message};
use mavlink_tlog::{TlogReader, Version};

use crate::mav_parser::LogEntry;
use crate::mav_parser::MavParser;

/// A parser for telemetry log (TLOG) files that uses the MAVLink protocol.
///
/// The `TlogParser` struct wraps a MAVLink connection to read messages
/// from a TLOG file. It implements the `MavParser` trait, allowing it to
/// process MAVLink messages and return them as `LogEntry` objects.
///
/// # Type Parameters
/// - `M`: The type of MAVLink message being parsed.
pub struct TlogParser<M: Message> {
    /// Reads timestamped records from the TLOG file.
    reader: TlogReader<BufReader<File>>,
    _msg: std::marker::PhantomData<M>,
}

impl<M: Message> TlogParser<M> {
    /// Creates a new `TlogParser` instance for the specified TLOG file path.
    ///
    /// # Arguments
    /// - `file_path`: The path to the TLOG file to be parsed.
    ///
    /// # Panics
    /// This function will panic if the provided file path is invalid or if
    /// the connection to the TLOG file cannot be established.
    ///
    pub fn new(file_path: &str) -> Self {
        let file = File::open(file_path).expect("An invalid file path was provided");
        Self {
            reader: TlogReader::new(BufReader::new(file)),
            _msg: std::marker::PhantomData,
        }
    }
}

impl<M: Message> MavParser for TlogParser<M> {
    type M = M;

    /// Reads the next MAVLink message from the TLOG file and returns it as a `LogEntry`.
    ///
    /// # Returns
    /// - `Ok(LogEntry)`: If a message is successfully read from the TLOG file.
    /// - `Err(MessageReadError)`: If an error occurs while reading the message.
    ///
    /// The `LogEntry` contains the MAVLink message, its header, and optional
    /// metadata such as a timestamp or raw data.
    ///
    fn parse_next_entry(&mut self) -> Result<LogEntry<Self::M>, MessageReadError> {
        loop {
            let record = match self.reader.next_record()? {
                Some(record) => record,
                None => return Err(MessageReadError::Io(Error::from(ErrorKind::UnexpectedEof))),
            };
            let version = match record.frame.version() {
                Version::V1 => MavlinkVersion::V1,
                Version::V2 => MavlinkVersion::V2,
            };
            let mut frame = PeekReader::new(record.frame.as_bytes());
            if let Ok((header, msg)) = mavlink::read_versioned_msg::<M, _>(&mut frame, version) {
                return Ok(LogEntry {
                    timestamp: Some(record.timestamp_us),
                    mav_header: Some(header),
                    mav_message: Some(msg),
                    text: None,
                    raw: None,
                });
            }
        }
    }
}
