use std::convert::TryFrom;
use std::convert::TryInto;
use std::time::SystemTime;

use uuid::Uuid;

/// Struct representing format flags for the log file.
///
/// `FormatFlags` contains options that modify the format of the log file:
/// - `mavlink_only`: If set, only MAVLink messages are logged, allowing for a more compact log file.
/// - `no_timestamp`: If set, timestamps per entry are not included in the log file.
pub struct FormatFlags {
    /// If set, only MAVLink messages are logged allowing for a more compact log file.
    pub mavlink_only: bool,
    /// If set, timestamps per entry are not included in the log file.
    pub no_timestamp: bool,
}

impl FormatFlags {
    /// Unpacks a 16-bit integer into a `FormatFlags` struct.
    ///
    /// # Arguments
    /// - `packed_data`: A 16-bit integer representing the format flags.
    ///
    /// # Returns
    /// A `FormatFlags` struct with the corresponding flags set.
    #[cfg(feature = "parser")]
    pub fn unpack(packed_data: u16) -> Self {
        FormatFlags {
            mavlink_only: packed_data & 0x01 != 0,
            no_timestamp: packed_data & 0x02 != 0,
        }
    }

    /// Packs the `FormatFlags` into a 2-byte array.
    ///
    /// This method converts the `FormatFlags` into a 2-byte array where each flag is represented by a bit.
    ///
    /// # Returns
    /// A `[u8; 2]` array containing the packed representation of the `FormatFlags`.
    #[cfg(feature = "logger")]
    pub fn pack(&self) -> [u8; 2] {
        let flags: u16 = (self.mavlink_only as u16) | ((self.no_timestamp as u16) << 1);
        flags.to_le_bytes()
    }
}

impl Default for FormatFlags {
    /// Provides default values for `FormatFlags`.
    ///
    /// By default, both `mavlink_only` and `no_timestamp` are set to `false`.
    fn default() -> Self {
        FormatFlags {
            mavlink_only: false,
            no_timestamp: false,
        }
    }
}

/// Enum representing the payload type for MAVLink message definitions.
///
/// `MavlinkDefinitionPayloadType` specifies the type of payload used to identify message definitions:
/// - `None`: No payload. Use MAVLink main XML definition as default.
/// - `Utf8SpaceDelimitedUrlsForXMLFiles`: UTF-8 encoded space-delimited URLs for XML files.
/// - `Utf8Xml`: UTF-8 encoded XML.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MavlinkDefinitionPayloadType {
    /// No payload. Use MAVLink main XML definition as default.
    None = 0,
    /// UTF-8 encoded comma delimited URLs for XML files.
    Utf8SpaceDelimitedUrlsForXMLFiles = 1,
    /// UTF-8 encoded XML.
    Utf8Xml = 2,
}

impl TryFrom<u16> for MavlinkDefinitionPayloadType {
    type Error = ();

    /// Converts a 16-bit integer into a `MavlinkDefinitionPayloadType`.
    ///
    /// # Arguments
    /// - `value`: A 16-bit integer representing the payload type.
    ///
    /// # Returns
    /// A `MavlinkDefinitionPayloadType` enum variant, or an error if the value is invalid.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MavlinkDefinitionPayloadType::None),
            1 => Ok(MavlinkDefinitionPayloadType::Utf8SpaceDelimitedUrlsForXMLFiles),
            2 => Ok(MavlinkDefinitionPayloadType::Utf8Xml),
            _ => Err(()),
        }
    }
}

/// Struct representing a MAVLink message definition.
///
/// `MavlinkMessageDefinition` contains information about the MAVLink protocol version, dialect, payload type, and the actual payload.
pub struct MavlinkMessageDefinition {
    /// MAVLink protocol major version number.
    pub version_major: u32,
    /// MAVLink protocol minor version number.
    pub version_minor: u32,
    /// String identifying mavlink dialect.
    pub dialect: String,
    /// Type of payload used to identify message definition.
    pub payload_type: MavlinkDefinitionPayloadType,
    /// Size of message definition payload in bytes.
    pub size: u32,
    /// Variable size message definition payload.
    pub payload: Option<Vec<u8>>,
}

impl MavlinkMessageDefinition {
    /// Default dialect for MAVLink message definitions.
    pub const DEFAULT_DIALECT: &str = "common";

    /// Unpacks a fixed-size byte array into a `MavlinkMessageDefinition` struct.
    ///
    /// # Arguments
    /// - `packed_data`: A fixed-size byte array containing the packed message definition.
    ///
    /// # Returns
    /// A `MavlinkMessageDefinition` struct with the unpacked data.
    #[cfg(feature = "parser")]
    pub fn unpack(packed_data: &[u8; 46]) -> Self {
        // stop at the first null byte when unpacking a string
        let end_dialect_ind: usize = match packed_data[8..40].iter().position(|&x| x == 0) {
            Some(index) => index + 8,
            None => 40,
        };
        MavlinkMessageDefinition {
            version_major: u32::from_le_bytes(packed_data[0..4].try_into().unwrap()),
            version_minor: u32::from_le_bytes(packed_data[4..8].try_into().unwrap()),
            dialect: String::from_utf8(packed_data[8..end_dialect_ind].to_vec()).unwrap(),
            payload_type: u16::from_le_bytes(packed_data[40..42].try_into().unwrap())
                .try_into()
                .unwrap(),
            size: u32::from_le_bytes(packed_data[42..46].try_into().unwrap()),
            payload: None,
        }
    }

    /// Unpacks the payload for the message definition.
    ///
    /// # Arguments
    /// - `packed_data`: A byte slice containing the packed payload data.
    #[cfg(feature = "parser")]
    pub fn unpack_payload(&mut self, packed_data: &[u8]) {
        match self.payload_type {
            MavlinkDefinitionPayloadType::Utf8SpaceDelimitedUrlsForXMLFiles => {
                self.payload = Some(packed_data.to_vec());
            }
            MavlinkDefinitionPayloadType::Utf8Xml => {
                self.payload = Some(packed_data.to_vec());
            }
            _ => {}
        }
    }

    /// Packs the `MavlinkMessageDefinition` into a vector of bytes.
    ///
    /// This function serializes the `MavlinkMessageDefinition` into a byte vector
    /// by converting its fields into their respective byte representations and
    /// appending them to the vector. The packed data includes the major and minor
    /// version numbers, the dialect, the payload type, and the size. If the payload
    /// type is not `None`, the payload is also included in the packed data.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the packed byte representation of the `MavlinkMessageDefinition`.
    #[cfg(feature = "logger")]
    pub fn pack(&self) -> Vec<u8> {
        assert!(self.dialect.len() <= 32, "dialect must be 32 bytes or less");
        let mut dialect_bytes = [0u8; 32];
        dialect_bytes[..self.dialect.len()].copy_from_slice(self.dialect.as_bytes());

        let mut packed: Vec<u8> = Vec::new();
        packed.extend_from_slice(&self.version_major.to_le_bytes());
        packed.extend_from_slice(&self.version_minor.to_le_bytes());
        packed.extend_from_slice(&dialect_bytes);
        let payload_type: u16 = self.payload_type as u16;
        packed.extend_from_slice(&payload_type.to_le_bytes());
        packed.extend_from_slice(&self.size.to_le_bytes());
        if self.payload_type != MavlinkDefinitionPayloadType::None {
            match &self.payload {
                Some(payload) => packed.extend_from_slice(payload),
                None => {}
            }
        }
        packed
    }
}

impl Default for MavlinkMessageDefinition {
    /// Provides default values for `MavlinkMessageDefinition`.
    ///
    /// By default, the major version is set to 2, minor version to 0, dialect to `DEFAULT_DIALECT`,
    /// payload type to `None`, size to 0, and payload to an empty vector.
    fn default() -> Self {
        MavlinkMessageDefinition {
            version_major: 2,
            version_minor: 0,
            dialect: String::from(MavlinkMessageDefinition::DEFAULT_DIALECT),
            payload_type: MavlinkDefinitionPayloadType::None,
            size: 0,
            payload: None,
        }
    }
}

/// Struct representing the file header for the log file.
///
/// `FileHeader` contains metadata about the log file, including a unique identifier, timestamp, source application ID,
/// format version, format flags, and message definitions.
pub struct FileHeader {
    /// Unique id for log file. It is expected the uuid library will be used to generate this.
    pub uuid: Uuid,
    /// The system unix timestamp in microseconds when the logger was initialized.
    pub timestamp_us: u64,
    /// String identifying the application used to generate the log file.
    pub src_application_id: String,
    /// A format version number. This is to allow compatability detection for future changes to the log file format.
    pub format_version: u32,
    /// A struct inidicating optional log file format changes.
    pub format_flags: FormatFlags,
    /// The message definitions for the log file.
    pub message_definition: MavlinkMessageDefinition,
}

impl FileHeader {
    /// Minimum size of the file header in bytes. Can be more if message definitions are included.
    pub const MIN_SIZE: usize = 108;
    /// Currently supported file format version.
    pub const FILE_FORMAT_VERSION: u32 = 1;
    /// Default source application ID.
    pub const SRC_APPLICATION_ID: &str = "mavlink_logger";

    /// Creates a new `FileHeader` with the provided format flags and message definition.
    ///
    /// This method initializes a new `FileHeader` with a unique UUID, the current timestamp in microseconds,
    /// the source application ID, format version, format flags, and message definition.
    ///
    /// # Arguments
    ///
    /// * `format_flags` - A `FormatFlags` struct indicating optional log file format changes.
    /// * `message_definition` - A `MavlinkMessageDefinition` struct containing the message definitions for the log file.
    ///
    /// # Returns
    ///
    /// A new `FileHeader` instance.
    #[cfg(feature = "logger")]
    pub fn new(
        format_flags: FormatFlags,
        message_definition: MavlinkMessageDefinition,
    ) -> FileHeader {
        let timestamp_us: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as u64;

        FileHeader {
            uuid: Uuid::new_v4(),
            timestamp_us,
            src_application_id: String::from(FileHeader::SRC_APPLICATION_ID),
            format_version: FileHeader::FILE_FORMAT_VERSION,
            format_flags,
            message_definition,
        }
    }

    /// Unpacks a fixed-size byte array into a `FileHeader` struct.
    ///
    /// # Arguments
    /// - `packed_data`: A fixed-size byte array containing the packed file header.
    ///
    /// # Returns
    /// A `FileHeader` struct with the unpacked data.
    #[cfg(feature = "parser")]
    pub fn unpack(packed_data: &[u8; 108]) -> Self {
        let id_end: usize = match packed_data[24..56].iter().position(|&x| x == 0) {
            Some(index) => index + 24,
            None => 56,
        };
        let src_application_id: String = match String::from_utf8(packed_data[24..id_end].to_vec()) {
            Ok(v) => v,
            Err(_e) => "".to_string(),
        };

        FileHeader {
            uuid: Uuid::from_bytes(packed_data[0..16].try_into().unwrap()),
            timestamp_us: u64::from_le_bytes(packed_data[16..24].try_into().unwrap()),
            src_application_id,
            format_version: u32::from_le_bytes(packed_data[56..60].try_into().unwrap()),
            format_flags: FormatFlags::unpack(u16::from_le_bytes(
                packed_data[60..62].try_into().unwrap(),
            )),
            message_definition: MavlinkMessageDefinition::unpack(
                packed_data[62..].try_into().unwrap(),
            ),
        }
    }

    /// Packs the `FileHeader` into a vector of bytes.
    ///
    /// This method serializes the `FileHeader` fields into a byte vector in the following order:
    /// - UUID (16 bytes)
    /// - Timestamp in microseconds (8 bytes)
    /// - Source application ID (32 bytes, UTF-8 encoded)
    /// - Format version (8 bytes)
    /// - Format flags (2 bytes, packed)
    /// - Message definition (variable length, packed)
    /// All bytes are packed in little-endian format.
    ///
    /// # Returns
    /// A `Vec<u8>` containing the packed representation of the `FileHeader`.
    #[cfg(feature = "logger")]
    pub fn pack(&self) -> Vec<u8> {
        assert!(
            self.src_application_id.len() <= 32,
            "src_application_id must be 32 bytes or less"
        );
        let mut app_id_bytes = [0u8; 32];
        app_id_bytes[..self.src_application_id.len()]
            .copy_from_slice(self.src_application_id.as_bytes());

        let mut packed: Vec<u8> = Vec::new();
        packed.extend_from_slice(self.uuid.as_bytes());
        packed.extend_from_slice(&self.timestamp_us.to_le_bytes());
        packed.extend_from_slice(&app_id_bytes);
        packed.extend_from_slice(&self.format_version.to_le_bytes());
        packed.extend_from_slice(&self.format_flags.pack());
        packed.extend_from_slice(&self.message_definition.pack());
        packed
    }
}

impl Default for FileHeader {
    /// Provides default values for `FileHeader`.
    ///
    /// By default, the UUID is generated using the `uuid` library, the timestamp is set to the current time in microseconds,
    /// the source application ID is set to `SRC_APPLICATION_ID`, the format version is set to `FILE_FORMAT_VERSION`,
    /// the format flags are set to `FormatFlags::default()`, and the message definition is set to `MavlinkMessageDefinition::default()`.
    fn default() -> Self {
        let timestamp_us: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros() as u64;

        FileHeader {
            uuid: Uuid::new_v4(),
            timestamp_us,
            src_application_id: String::from(FileHeader::SRC_APPLICATION_ID),
            format_version: FileHeader::FILE_FORMAT_VERSION,
            format_flags: FormatFlags::default(),
            message_definition: MavlinkMessageDefinition::default(),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "parser")]
/// Unit tests for the `header` module.
///
/// This module contains tests for the functionality provided by the `header` module,
/// including unpacking format flags, MAVLink message definitions, and file headers.
mod parser_tests {
    use super::*;

    #[test]
    /// Tests the `unpack` method of `FormatFlags` to ensure it correctly extracts flags from a 16-bit integer.
    fn test_format_flags_unpack() {
        let packed_data: u16 = 0b11;
        let flags = FormatFlags::unpack(packed_data);
        assert!(flags.mavlink_only);
        assert!(flags.no_timestamp);

        let packed_data: u16 = 0b01;
        let flags = FormatFlags::unpack(packed_data);
        assert!(flags.mavlink_only);
        assert!(!flags.no_timestamp);

        let packed_data: u16 = 0b10;
        let flags = FormatFlags::unpack(packed_data);
        assert!(!flags.mavlink_only);
        assert!(flags.no_timestamp);

        let packed_data: u16 = 0b00;
        let flags = FormatFlags::unpack(packed_data);
        assert!(!flags.mavlink_only);
        assert!(!flags.no_timestamp);
    }

    #[test]
    /// Tests the `try_from` implementation for `MavlinkDefinitionPayloadType` to ensure it correctly converts
    /// valid 16-bit integers into the corresponding enum variants and returns an error for invalid values.
    fn test_mavlink_definition_payload_type_try_from() {
        assert_eq!(
            MavlinkDefinitionPayloadType::try_from(0).unwrap(),
            MavlinkDefinitionPayloadType::None
        );
        assert_eq!(
            MavlinkDefinitionPayloadType::try_from(1).unwrap(),
            MavlinkDefinitionPayloadType::Utf8SpaceDelimitedUrlsForXMLFiles
        );
        assert_eq!(
            MavlinkDefinitionPayloadType::try_from(2).unwrap(),
            MavlinkDefinitionPayloadType::Utf8Xml
        );
        assert!(MavlinkDefinitionPayloadType::try_from(3).is_err());
    }

    #[test]
    /// Tests the `unpack` method of `MavlinkMessageDefinition` to ensure it correctly extracts message definition
    /// data from a fixed-size byte array and handles payload unpacking properly.
    fn test_mavlink_message_definition_unpack() {
        let packed_data: [u8; 46] = [
            1, 0, 0, 0, // version_major
            2, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];
        let definition = MavlinkMessageDefinition::unpack(&packed_data);
        assert_eq!(definition.version_major, 1);
        assert_eq!(definition.version_minor, 2);
        assert_eq!(definition.dialect, "test");
        assert_eq!(definition.payload_type, MavlinkDefinitionPayloadType::None);
        assert_eq!(definition.size, 0);
        assert!(definition.payload.is_none());

        let mut packed_data: [u8; 46] = [
            1, 0, 0, 2, // version_major
            2, 0, 0, 1, // version_minor
            b't', b'e', b's', b't', b' ', b'1', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, // dialect
            1, 0, // payload_type
            0, 0, 0, 0, // size
        ];
        let urls_str: String = String::from("http://example.com http://example.2.com");
        let encoded_urls: &[u8] = urls_str.as_bytes();
        packed_data[42..46].copy_from_slice(&(encoded_urls.len() as u32).to_le_bytes());
        let mut definition = MavlinkMessageDefinition::unpack(&packed_data);
        assert_eq!(definition.version_major, 0x02000001);
        assert_eq!(definition.version_minor, 0x01000002);
        assert_eq!(definition.dialect, "test 1");
        assert_eq!(
            definition.payload_type,
            MavlinkDefinitionPayloadType::Utf8SpaceDelimitedUrlsForXMLFiles
        );
        assert_eq!(definition.size, encoded_urls.len() as u32);
        assert!(definition.payload.is_none());
        definition.unpack_payload(encoded_urls);
        assert_eq!(definition.payload, Some(encoded_urls.to_vec()));
    }

    #[test]
    /// Tests the `unpack` method of `FileHeader` to ensure it correctly extracts file header data from a fixed-size
    /// byte array, including UUID, timestamp, application ID, format flags, and message definitions.
    fn test_file_header_unpack() {
        let packed_data: [u8; 108] = [
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 2, // format_version
            3, 4, // format_flags
            // message_definition
            4, 0, 0, 5, // version_major
            6, 0, 0, 7, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            2, 0, // payload_type
            10, 0, 0, 0, // size
        ];
        let header = FileHeader::unpack(&packed_data);
        assert_eq!(
            header.uuid,
            Uuid::from_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
        );
        assert_eq!(header.timestamp_us, 0x1100000000000010);
        assert_eq!(header.src_application_id, "app");
        assert_eq!(header.format_version, 0x02000001);
        assert!(header.format_flags.mavlink_only);
        assert!(header.format_flags.no_timestamp);
        assert_eq!(header.message_definition.version_major, 0x05000004);
        assert_eq!(header.message_definition.version_minor, 0x07000006);
        assert_eq!(header.message_definition.dialect, "test");
        assert_eq!(
            header.message_definition.payload_type,
            MavlinkDefinitionPayloadType::Utf8Xml
        );
        assert_eq!(header.message_definition.size, 10);
        assert!(header.message_definition.payload.is_none());
    }
}

#[cfg(test)]
#[cfg(feature = "logger")]
mod logger_tests {
    use super::*;

    #[test]
    /// Tests the `pack` method of `FormatFlags`.
    ///
    /// This test verifies that the `pack` method correctly converts the `FormatFlags`
    /// struct into a 2-byte array representation. It checks various combinations of
    /// the `mavlink_only` and `no_timestamp` flags to ensure the correct bit
    /// representation in the packed array.
    fn test_format_flags_pack() {
        let flags = FormatFlags {
            mavlink_only: false,
            no_timestamp: false,
        };
        assert_eq!(flags.pack(), [0, 0]);

        let flags = FormatFlags {
            mavlink_only: true,
            no_timestamp: false,
        };
        assert_eq!(flags.pack(), [1, 0]);

        let flags = FormatFlags {
            mavlink_only: false,
            no_timestamp: true,
        };
        assert_eq!(flags.pack(), [2, 0]);

        let flags = FormatFlags {
            mavlink_only: true,
            no_timestamp: true,
        };
        assert_eq!(flags.pack(), [3, 0]);
    }

    #[test]
    /// Tests the `pack` method of `MavlinkMessageDefinition`.
    ///
    /// This test verifies that the `pack` method correctly serializes the
    /// `MavlinkMessageDefinition` struct into a byte vector. It checks the
    /// packed representation for both default and custom values of the
    /// `MavlinkMessageDefinition` fields, ensuring that each field is
    /// correctly converted to its byte representation and appended to the
    /// vector in the correct order.
    fn test_mavlink_message_definition_pack() {
        let definition = MavlinkMessageDefinition {
            version_major: 2,
            version_minor: 0,
            dialect: String::from(MavlinkMessageDefinition::DEFAULT_DIALECT),
            payload_type: MavlinkDefinitionPayloadType::None,
            size: 0,
            payload: None,
        };
        let packed = definition.pack();
        assert_eq!(packed.len(), 46);
        assert_eq!(&packed[0..4], &[2, 0, 0, 0]);
        assert_eq!(&packed[4..8], &[0, 0, 0, 0]);
        assert_eq!(
            String::from_utf8(packed[8..40].to_vec()).unwrap(),
            "common\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
        );
        assert_eq!(&packed[40..42], &[0, 0]);
        assert_eq!(&packed[42..46], &[0, 0, 0, 0]);

        let definition = MavlinkMessageDefinition {
            version_major: 0x01020304,
            version_minor: 0x04050607,
            dialect: String::from(MavlinkMessageDefinition::DEFAULT_DIALECT),
            payload_type: MavlinkDefinitionPayloadType::Utf8Xml,
            size: 5,
            payload: Some(b"hello".to_vec()),
        };
        let packed = definition.pack();
        assert_eq!(packed.len(), 51);
        assert_eq!(&packed[0..4], &[4, 3, 2, 1]);
        assert_eq!(&packed[4..8], &[7, 6, 5, 4]);
        assert_eq!(
            String::from_utf8(packed[8..40].to_vec()).unwrap(),
            "common\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
        );
        assert_eq!(&packed[40..42], &[2, 0]);
        assert_eq!(&packed[42..46], &[5, 0, 0, 0]);
        assert_eq!(&packed[46..51], b"hello");
    }

    #[test]
    /// Tests the `pack` method of `FileHeader`.
    ///
    /// This test verifies that the `pack` method correctly serializes the
    /// `FileHeader` struct into a byte vector. It checks the packed representation
    /// for a `FileHeader` instance with custom values for the `format_flags` and
    /// `message_definition` fields. The test ensures that each field is correctly
    /// converted to its byte representation and appended to the vector in the
    /// correct order, including the UUID, timestamp, source application ID,
    /// format version, format flags, and message definition.
    fn test_file_header_pack() {
        let format_flags = FormatFlags {
            mavlink_only: true,
            no_timestamp: false,
        };
        let message_definition = MavlinkMessageDefinition {
            version_major: 2,
            version_minor: 0,
            dialect: String::from(MavlinkMessageDefinition::DEFAULT_DIALECT),
            payload_type: MavlinkDefinitionPayloadType::Utf8Xml,
            size: 5,
            payload: Some(b"hello".to_vec()),
        };
        let header = FileHeader::new(format_flags, message_definition);
        let packed = header.pack();
        assert_eq!(packed.len(), 113);
        assert_eq!(&packed[16..24], &header.timestamp_us.to_le_bytes()); // timestamp
        assert_eq!(
            String::from_utf8(packed[24..56].to_vec()).unwrap(),
            "mavlink_logger\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
        ); // src application id
        assert_eq!(&packed[56..60], &[1, 0, 0, 0]); // file version
        assert_eq!(&packed[60..62], &[1, 0]); // format flags
        assert_eq!(&packed[62..113], &header.message_definition.pack()[..]);
    }
}
