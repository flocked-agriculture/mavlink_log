#![crate_name = "mavlink_log"]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../docs/mav_log_file_format.md")]
#![doc = include_str!("../docs/tlog_file_format.md")]

#[cfg(feature = "tlog")]
pub mod tlog;

#[cfg(feature = "mavlog")]
pub mod mavlog;

#[cfg(feature = "logger")]
pub mod mav_logger {
    use mavlink::{MavFrame, Message};

    pub trait MavLogger {
        fn write_mavlink<M: Message>(&mut self, frame: MavFrame<M>) -> std::io::Result<()>;
    }
}

#[cfg(feature = "parser")]
pub mod mav_parser {
    use std::option::Option;

    use mavlink::error::MessageReadError;
    use mavlink::{MavHeader, Message};

    /// Represents a single log entry in a MAVLink log or telemetry log.
    ///
    /// # Type Parameters
    /// - `M`: A type that implements the `Message` trait, representing a MAVLink message.
    ///
    /// # Fields
    /// - `timestamp`: The timestamp of the log entry, if available.
    /// - `mav_header`: The MAVLink header associated with the message, if available.
    /// - `mav_message`: The MAVLink message, if available.
    /// - `text`: Any textual information associated with the log entry, if available.
    /// - `raw`: The raw binary data of the log entry, if available.
    pub struct LogEntry<M: Message> {
        pub timestamp: Option<u64>,
        pub mav_header: Option<MavHeader>,
        pub mav_message: Option<M>,
        pub text: Option<String>,
        pub raw: Option<Vec<u8>>,
    }

    impl<M: Message> Default for LogEntry<M> {
        /// Provides a default implementation for `LogEntry`.
        ///
        /// All fields are initialized to `None`.
        fn default() -> Self {
            Self {
                timestamp: None,
                mav_header: None,
                mav_message: None,
                text: None,
                raw: None,
            }
        }
    }

    /// A trait for parsing MAVLink logs or telemetry logs.
    ///
    /// # Associated Types
    /// - `M`: A type that implements the `Message` trait, representing a MAVLink message.
    ///
    /// # Required Methods
    /// - `next`: Reads the next log entry from the log source.
    ///
    /// # Errors
    /// Returns a `MessageReadError` if there is an issue reading the next log entry. This
    /// applies to EOF as well.
    pub trait MavParser {
        type M: Message;

        /// Reads the next log entry from the log source.
        ///
        /// # Returns
        /// - `Ok(LogEntry<Self::M>)`: The next log entry if successfully read.
        /// - `Err(MessageReadError)`: An error if the log entry could not be read.
        fn parse_next_entry(&mut self) -> Result<LogEntry<Self::M>, MessageReadError>;
    }
}
