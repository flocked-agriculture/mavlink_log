use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::File;

use mavlink::error::MessageReadError;
use mavlink::peek_reader::PeekReader;
use mavlink::{MavlinkVersion, Message, read_versioned_msg};

use super::header::{FileHeader, MavlinkDefinitionPayloadType};
use crate::mav_parser::{LogEntry, MavParser};

/// Enum representing the type of log entry.
///
/// `EntryType` specifies the type of data stored in a log entry:
/// - `Raw`: Raw binary data.
/// - `Mavlink`: MAVLink message.
/// - `Utf8Text`: UTF-8 encoded text.
enum EntryType {
    Raw = 0,
    Mavlink = 1,
    Utf8Text = 2,
}

impl TryFrom<u8> for EntryType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EntryType::Raw),
            1 => Ok(EntryType::Mavlink),
            2 => Ok(EntryType::Utf8Text),
            _ => Err(()),
        }
    }
}

/// Parser for MAVLink-only log files without timestamps.
///
/// This parser assumes the log file contains only MAVLink messages and no timestamps.
/// It reads MAVLink messages sequentially from the file.
struct MavlinkOnlyNoTimestampParser<M: Message> {
    reader: PeekReader<File>,
    mav_version: MavlinkVersion,
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Message> MavParser for MavlinkOnlyNoTimestampParser<M> {
    type M = M;

    /// Reads the next MAVLink message from the log file.
    ///
    /// If the data is corrupted, it will block and search for the next valid MAVLink packet.
    ///
    /// # Returns
    ///
    /// A `LogEntry` containing the MAVLink message and its header.
    ///
    /// # Errors
    ///
    /// Returns a `MessageReadError` if there is an issue reading or parsing the MAVLink message.
    /// This includes errors such as:
    /// - I/O errors while reading from the file.
    /// - Corrupted MAVLink packets that cannot be parsed.
    ///
    /// # Panics
    ///
    /// Panics if the `read_versioned_msg` function encounters an unrecoverable error.
    ///
    fn parse_next_entry(&mut self) -> Result<LogEntry<M>, MessageReadError> {
        let mut entry: LogEntry<M> = LogEntry::default();
        // NOTE: the read_versioned_msg function will do a blocking search for the next valid mavlink packet if
        // it tries to unpack the current data and gets something unexpected. Since this is a mavlink only file with
        // no timestamps, we can safely allow this to happen. The Mavlink infrastructure has a lot of hours and false
        // positives in the magic number search do not seem like a problem with Mavlink only data streams.
        let (header, message) = read_versioned_msg::<M, File>(&mut self.reader, self.mav_version)?;
        entry.mav_header = Some(header);
        entry.mav_message = Some(message);
        Ok(entry)
    }
}

/// Parser for MAVLink-only log files with timestamps.
///
/// This parser assumes the log file contains only MAVLink type data, each preceded by a timestamp.
/// It reads MAVLink messages and their associated timestamps sequentially from the file.
struct TimestampedMavlinkOnlyParser<M: Message> {
    reader: PeekReader<File>,
    mav_version: MavlinkVersion,
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Message> MavParser for TimestampedMavlinkOnlyParser<M> {
    type M = M;

    /// Reads the next MAVLink message and its timestamp from the log file.
    ///
    /// If the data is corrupted, it will silently fail and attempt to read the next MAVLink message.
    ///
    /// # Returns
    ///
    /// A `LogEntry` containing the MAVLink message, its header, and the timestamp.
    ///
    /// # Errors
    ///
    /// Returns a `MessageReadError` if there is an issue reading or parsing the MAVLink message or timestamp.
    /// This includes:
    /// - I/O errors while reading from the file.
    /// - Corrupted MAVLink packets or invalid timestamps.
    ///
    /// # Panics
    ///
    /// Panics if the `peek_exact` or `read_exact` methods encounter an unrecoverable error.
    ///
    fn parse_next_entry(&mut self) -> Result<LogEntry<M>, MessageReadError> {
        let mut entry: LogEntry<M> = LogEntry::default();
        let magic_number: u8 = match self.mav_version {
            MavlinkVersion::V1 => mavlink::MAV_STX,
            MavlinkVersion::V2 => mavlink::MAV_STX_V2,
        };
        if self.reader.peek_exact(9)?[8] == magic_number {
            let timestamp_raw: &[u8] = self.reader.read_exact(8)?;
            entry.timestamp = match timestamp_raw.try_into() {
                Ok(bytes) => Some(u64::from_le_bytes(bytes)),
                Err(_) => None,
            };
        }
        // WARNING: this will silently fail and try to get next mavlink message on data corruption
        // this is a concern that some messages could be associated with the wrong timestamp
        // we need a version of this to fail immediately on any parsing issue
        let (header, message) = read_versioned_msg::<M, File>(&mut self.reader, self.mav_version)?;
        entry.mav_header = Some(header);
        entry.mav_message = Some(message);
        Ok(entry)
    }
}

/// Parser for mixed log files containing various entry types.
///
/// This parser can handle log files with raw data, MAVLink messages, and UTF-8 text entries.
/// It also supports optional timestamps for each entry.
pub struct MixedParser<M: Message> {
    timestamped: bool,
    reader: PeekReader<File>,
    mav_version: MavlinkVersion,
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Message> MavParser for MixedParser<M> {
    type M = M;

    /// Reads the next log entry from the file.
    ///
    /// Determines the entry type and processes it accordingly:
    /// - `Raw`: Reads raw binary data.
    /// - `Mavlink`: Reads a MAVLink message.
    /// - `Utf8Text`: Reads UTF-8 encoded text.
    /// If timestamps are enabled, reads the timestamp for the entry.
    ///
    /// # Returns
    ///
    /// A `LogEntry` containing the parsed data, which may include a timestamp, MAVLink message, or text.
    ///
    /// # Errors
    ///
    /// Returns a `MessageReadError` if there is an issue parsing the log entry data. This includes:
    /// - I/O errors while reading from the file.
    /// - Corrupted MAVLink packets or invalid UTF-8 text.
    ///
    /// # Panics
    ///
    /// Panics if the entry payload size cannot be read because this is unrecoverable.
    ///
    fn parse_next_entry(&mut self) -> Result<LogEntry<M>, MessageReadError> {
        let mut entry: LogEntry<M> = LogEntry::default();
        let entry_type: EntryType = self
            .reader
            .read_u8()
            // If entry type is unknown default to raw
            .map(|value| value.try_into().unwrap_or(EntryType::Raw))?;
        if self.timestamped {
            let timestamp_raw: &[u8] = self.reader.read_exact(8)?;
            entry.timestamp = match timestamp_raw.try_into() {
                Ok(bytes) => Some(u64::from_le_bytes(bytes)),
                Err(_) => None,
            };
        }
        let payload_size: u16 = u16::from_le_bytes(
            self.reader
                .read_exact(2)?
                .try_into()
                .expect("Failed to read log entry payload size."),
        );
        match entry_type {
            EntryType::Raw => {
                let payload = self.reader.read_exact(payload_size as usize)?;
                entry.raw = Some(payload.to_vec())
            }
            EntryType::Mavlink => {
                // WARNING: this will silently fail and try to get next mavlink message on data corruption
                // this is a concern that some messages could be associated with the wrong timestamp
                // or non mavlink entries could get skipped
                // we need a version of this to fail immediately on any parsing issue
                let (header, message) =
                    read_versioned_msg::<M, File>(&mut self.reader, self.mav_version)?;
                entry.mav_header = Some(header);
                entry.mav_message = Some(message);
                return Ok(entry);
            }
            EntryType::Utf8Text => {
                let payload = self.reader.read_exact(payload_size as usize)?;
                entry.text = match String::from_utf8(payload.to_vec()) {
                    Ok(text) => Some(text),
                    Err(_) => {
                        return Err(MessageReadError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Failed to decode UTF-8 text from payload",
                        )));
                    }
                };
            }
        }
        Ok(entry)
    }
}

/// High-level parser for MAVLink log files.
///
/// `MavLogParser` automatically determines the log file format and selects the appropriate parser.
/// It supports MAVLink-only files (with or without timestamps) and mixed log files.
pub struct MavLogParser<M: Message + 'static> {
    parser: Box<dyn MavParser<M = M>>,
}

impl<M: Message + 'static> MavLogParser<M> {
    /// Creates a new `MavLogParser` for the specified log file.
    ///
    /// Automatically detects the log file format and initializes the appropriate parser.
    ///
    /// # Arguments
    ///
    /// - `file_path`: Path to the log file.
    ///
    /// # Returns
    ///
    /// An instance of `MavLogParser` initialized with the appropriate parser.
    ///
    /// # Panics
    ///
    /// Panics if the file header cannot be read or if the format is unsupported.
    ///
    pub fn new(file_path: &str) -> Self {
        let file: File = File::open(file_path).expect("Failed to open file");
        let mut reader: PeekReader<File> = PeekReader::new(file);

        let header = Self::read_file_header(&mut reader);

        let mav_version = Self::determine_mavlink_version(&header);

        let parser: Box<dyn MavParser<M = M>> = if header.format_flags.mavlink_only {
            if header.format_flags.no_timestamp {
                Box::new(MavlinkOnlyNoTimestampParser {
                    reader,
                    mav_version,
                    _phantom: std::marker::PhantomData,
                })
            } else {
                Box::new(TimestampedMavlinkOnlyParser {
                    reader,
                    mav_version,
                    _phantom: std::marker::PhantomData,
                })
            }
        } else {
            Box::new(MixedParser {
                timestamped: !header.format_flags.no_timestamp,
                reader,
                mav_version,
                _phantom: std::marker::PhantomData,
            })
        };

        MavLogParser { parser }
    }

    /// Reads the file header to extract metadata and format information.
    ///
    /// # Arguments
    /// - `reader`: A `PeekReader` for the log file.
    ///
    /// # Returns
    /// A `FileHeader` containing metadata about the log file.
    ///
    /// # Panics
    ///
    /// Panics if the file header is corrupted or if the format is unsupported since that makes
    /// it impossible to guarantee correct parsing.
    ///
    fn read_file_header(reader: &mut PeekReader<File>) -> FileHeader {
        let header_bytes: [u8; 108] = reader
            .read_exact(FileHeader::MIN_SIZE)
            .expect("Failed to read file header.")
            .try_into()
            .expect("Failed to read file header.");
        let mut header = FileHeader::unpack(&header_bytes);
        if header.message_definition.payload_type != MavlinkDefinitionPayloadType::None {
            let definitions_raw: &[u8] = reader
                .read_exact(header.message_definition.size as usize)
                .expect("Failed to read message definitions.");
            header.message_definition.unpack_payload(definitions_raw);
        } else {
            header.message_definition.size = 0;
        }

        match header.message_definition.payload_type {
            MavlinkDefinitionPayloadType::None => {}
            MavlinkDefinitionPayloadType::Utf8SpaceDelimitedUrlsForXMLFiles => {
                panic!("Custom XML files for message definitions are not supported.");
            }
            MavlinkDefinitionPayloadType::Utf8Xml => {
                panic!("XML for message definitions is not supported.");
            }
        }

        match header.format_version {
            1 => {}
            _ => panic!("Unsupported file format version."),
        }

        header
    }

    /// Determines the MAVLink version based on the file header.
    ///
    /// # Arguments
    /// - `header`: A `FileHeader` containing metadata about the log file.
    ///
    /// # Returns
    /// The MAVLink version (`V1` or `V2`).
    ///
    /// # Panics
    ///
    /// Panics if the MAVLink version is unsupported.
    ///
    fn determine_mavlink_version(header: &FileHeader) -> MavlinkVersion {
        match header.message_definition.version_major {
            2 => MavlinkVersion::V2,
            1 => MavlinkVersion::V1,
            _ => panic!("Unsupported MAVLink version."),
        }
    }
}

impl<M: Message + 'static> MavParser for MavLogParser<M> {
    type M = M;

    /// Reads the next log entry using the underlying parser.
    ///
    /// Delegates the call to the specific parser selected during initialization.
    ///
    /// # Returns
    ///
    /// a `LogEntry` containing the parsed data, which may include a timestamp, MAVLink message, or text.
    ///
    fn parse_next_entry(&mut self) -> Result<LogEntry<M>, MessageReadError> {
        self.parser.parse_next_entry()
    }
}
