//! This module defines the `RotatingTLog` struct which implements the `MavLogger` trait.
//! It provides functionality to log MAVLink messages to a rotating tlog file.
//! The tlog format is the informal logging format used by MAVLink ground stations.
//! You can learn more at docs/tlog_file_format.md.

use std::time::SystemTime;

use mavlink::{MavFrame, Message};
use rotating_file_handler::RotatingFileHandler;

use crate::mav_logger::MavLogger;
use mavlink::{MAVLinkV1MessageRaw, MAVLinkV2MessageRaw};

/// `RotatingTLog` is a logger that writes MAVLink messages to a file with rotation support.
/// The log file rotates when it reaches a specified size limit.
pub struct RotatingTlog {
    file_handler: RotatingFileHandler,
}

impl RotatingTlog {
    /// Creates a new `RotatingTLog` instance.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path for the log files. A file extension of .tlog is recommended.
    ///     If the path includes more than the file name, such as parent directories, it is
    ///     expected the folder path already exists.
    /// * `max_bytes` - The maximum size in bytes before the log file rotates.
    /// * `backup_count` - The number of backup files to keep.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the `RotatingTLog` was created successfully, or an `Err` if there was an error.
    pub fn new(base_path: &str, max_bytes: u64, backup_count: usize) -> std::io::Result<Self> {
        let file_handler = RotatingFileHandler::new(base_path, max_bytes, backup_count, None)?;
        Ok(Self { file_handler })
    }
}

impl MavLogger for RotatingTlog {
    /// Writes a MAVLink message to the log file.
    ///
    /// # Arguments
    ///
    /// * `frame` - The MAVLink message to log.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the message was logged successfully, or an `Err` if there was an error.
    fn write_mavlink<M: Message>(&mut self, frame: MavFrame<M>) -> std::io::Result<()> {
        let timestamp_us: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as u64;
        let mut record_bytes: Vec<u8> = timestamp_us.to_le_bytes().to_vec();

        match frame.protocol_version {
            mavlink::MavlinkVersion::V1 => {
                let mut msg: MAVLinkV1MessageRaw = MAVLinkV1MessageRaw::new();
                msg.serialize_message(frame.header, &frame.msg);
                record_bytes.extend_from_slice(msg.raw_bytes());
            }
            mavlink::MavlinkVersion::V2 => {
                let mut msg: MAVLinkV2MessageRaw = MAVLinkV2MessageRaw::new();
                msg.serialize_message(frame.header, &frame.msg);
                record_bytes.extend_from_slice(msg.raw_bytes());
            }
        }
        self.file_handler.emit(&record_bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mavlink::common::{ATTITUDE_DATA, MavMessage};
    use mavlink::error::MessageReadError;
    use mavlink::{MavHeader, MavlinkVersion};
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_write_mavlink_message() {
        // Define the name of the test log file
        const CASE_FILE_NAME: &str = "test_log.bin";
        // Remove the test log file if it exists from previous runs
        std::fs::remove_file(CASE_FILE_NAME).unwrap_or_else(|_| {});

        // Create a new RotatingTLog instance with a maximum size of 1024 bytes and no backup files
        let mut logger = RotatingTlog::new(CASE_FILE_NAME, 1024, 0).unwrap();

        // Define two MAVLink frames with different messages
        let mav_frame1 = MavFrame {
            header: MavHeader::default(),
            msg: MavMessage::HEARTBEAT(Default::default()),
            protocol_version: MavlinkVersion::V2,
        };
        let mav_frame2 = MavFrame {
            header: MavHeader::default(),
            msg: MavMessage::ATTITUDE(ATTITUDE_DATA {
                time_boot_ms: 33,
                roll: 43.5,
                pitch: 23.3,
                yaw: 99.9,
                rollspeed: 1.3,
                pitchspeed: 13.0,
                yawspeed: 0.0,
            }),
            protocol_version: MavlinkVersion::V2,
        };

        // Write the MAVLink frames to the log file multiple times
        for _ in 0..10 {
            // Write the first MAVLink frame and assert that it was successful
            let result = logger.write_mavlink(mav_frame1.clone());
            assert!(result.is_ok());
            // Write the second MAVLink frame and assert that it was successful
            let result = logger.write_mavlink(mav_frame2.clone());
            assert!(result.is_ok());
        }

        // Read the log file and verify its content
        let mut file: File = File::open(CASE_FILE_NAME).unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        // Assert that the log file has the expected length
        assert_eq!(content.len(), 10 * (27 + 44));

        // Verify the content of the log file
        for i in 0..10 {
            let offset = i * (27 + 44);
            // Assert that the timestamp is not zero
            assert_ne!(content[offset..offset + 8], [0u8; 8]);
            // Assert that the first MAVLink frame has the expected content
            assert_eq!(
                &content[offset + 8..offset + 27],
                [
                    253, 7, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 59, 166
                ]
            );

            // Assert that the timestamp is not zero
            assert_ne!(content[offset + 27..offset + 35], [0u8; 8]);
            // Assert that the second MAVLink frame has the expected content
            assert_eq!(
                &content[offset + 35..offset + 71],
                [
                    253, 24, 0, 0, 0, 255, 0, 30, 0, 0, 33, 0, 0, 0, 0, 0, 46, 66, 102, 102, 186,
                    65, 205, 204, 199, 66, 102, 102, 166, 63, 0, 0, 80, 65, 93, 187
                ]
            );
        }

        // check interpretation of tlog against existing library implementation
        let connection_string = format!("file:{CASE_FILE_NAME}");
        let vehicle = mavlink::connect::<MavMessage>(&connection_string);
        assert!(vehicle.is_ok(), "Incomplete address should error");

        let vehicle = vehicle.unwrap();
        let mut counter = 0;
        loop {
            match vehicle.recv() {
                Ok((_header, _msg)) => {
                    if counter % 2 == 0 {
                        assert_eq!(_msg, mav_frame1.msg);
                    } else {
                        assert_eq!(_msg, mav_frame2.msg);
                    }
                    counter += 1;
                }
                Err(MessageReadError::Io(e)) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        continue;
                    } else {
                        println!("recv error: {e:?}");
                        break;
                    }
                }
                _ => {}
            }
        }

        assert!(
            counter == 20,
            "Unable to hit the necessary amount of matches"
        );

        // Remove the test log file after the test
        std::fs::remove_file(CASE_FILE_NAME).unwrap_or_else(|_| {});
    }
}
