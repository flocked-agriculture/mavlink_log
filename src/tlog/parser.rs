/// This module provides functionality for parsing telemetry log (TLOG) files
/// using the MAVLink protocol. It defines the `TlogParser` struct, which
/// implements the `MavParser` trait to read and process MAVLink messages
/// from a TLOG file.
/// See /docs/tlog_file_format.md for more information on the TLOG file format.
use mavlink::error::MessageReadError;
use mavlink::{MavConnection, Message};

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
    /// A boxed MAVLink connection object for reading messages from the TLOG file.
    file_conn: Box<dyn MavConnection<M> + Sync + Send>,
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
        let connection_string = format!("file:{}", file_path);
        let vehicle =
            mavlink::connect::<M>(&connection_string).expect("An invalid file path was provided");
        Self { file_conn: vehicle }
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
        match self.file_conn.recv() {
            Ok((header, msg)) => Ok(LogEntry {
                timestamp: None,
                mav_header: Some(header),
                mav_message: Some(msg),
                text: None,
                raw: None,
            }),
            Err(err) => return Err(err),
        }
    }
}
