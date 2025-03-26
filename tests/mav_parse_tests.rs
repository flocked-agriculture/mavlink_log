#[cfg(all(feature = "mavlog", feature = "parser"))]
#[cfg(test)]
mod mav_parse_tests {
    use std::io::Write;

    use mavlink::common::{
        ATTITUDE_DATA, GPS2_RAW_DATA, GpsFixType, HEARTBEAT_DATA, MavAutopilot, MavMessage,
        MavModeFlag, MavState, MavType,
    };
    use mavlink::{MAVLinkV2MessageRaw, MavHeader};
    use mavlink_log::mav_parser::MavParser;
    use mavlink_log::mavlog::parser::MavLogParser;

    use tempfile;

    #[test]
    #[should_panic(expected = "Failed to read file header.")]
    fn test_mav_log_parser_file_to_small_for_header() {
        // not enough data for header
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write(&[0u8]).expect("Failed to write test file");
        MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(temp_file.path().to_str().unwrap());
    }

    #[test]
    #[should_panic(expected = "Unsupported file format version.")]
    fn test_mav_log_parser_file_invalid_file_version() {
        // not enough data for header
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let packed_data: [u8; 108] = [
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 2, // format_version
            0, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(temp_file.path().to_str().unwrap());
    }

    #[test]
    #[should_panic(expected = "Unsupported MAVLink version.")]
    fn test_mav_log_parser_file_invalid_mav_version() {
        // not enough data for header
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let packed_data: [u8; 108] = [
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            0, 0, // format_flags
            // message_definition
            3, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(temp_file.path().to_str().unwrap());
    }

    #[test]
    #[should_panic(expected = "Custom XML files for message definitions are not supported.")]
    fn test_mav_log_parser_file_unsupported_payload_type_urls() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let packed_data: [u8; 108] = [
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            0, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            1, 0, // payload_type
            0, 0, 0, 0, // size
        ];

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(temp_file.path().to_str().unwrap());
    }

    #[test]
    #[should_panic(expected = "XML for message definitions is not supported.")]
    fn test_mav_log_parser_file_unsupported_payload_type_xml() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let packed_data: [u8; 108] = [
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            0, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            2, 0, // payload_type
            0, 0, 0, 0, // size
        ];

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(temp_file.path().to_str().unwrap());
    }

    #[test]
    #[should_panic(expected = "Failed to read message definitions.")]
    fn test_mav_log_parser_failed_to_unpack_message_definitions() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let packed_data: [u8; 108] = [
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            0, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            2, 0, // payload_type
            10, 0, 0, 0, // size
        ];

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(temp_file.path().to_str().unwrap());
    }

    #[test]
    fn test_mav_log_parser_sub_parser_mavlink_only_no_timestamp() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let mut packed_data: Vec<u8> = vec![
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            3, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];
        populate_data(true, false, &mut packed_data);

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        let mut parser = MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(
            temp_file.path().to_str().unwrap(),
        );
        for i in 0..60 {
            let entry = parser.parse_next_entry();
            assert!(entry.is_ok(), "{i} {:?}", entry.err());
            let entry = entry.unwrap();
            assert!(entry.timestamp.is_none());
            assert!(entry.mav_header.is_some());
            assert!(entry.mav_message.is_some());
        }
    }

    #[test]
    fn test_mav_log_parser_sub_parser_mavlink_only_and_timestamp() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let mut packed_data: Vec<u8> = vec![
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            1, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];
        populate_data(true, true, &mut packed_data);

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        let mut parser = MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(
            temp_file.path().to_str().unwrap(),
        );
        for i in 0..60 {
            let entry = parser.parse_next_entry();
            assert!(entry.is_ok(), "Iteration: {i} {:?}", entry.err());
            let entry = entry.unwrap();
            assert!(entry.timestamp.is_some(), "Iteration: {i}");
            assert_eq!(entry.timestamp.unwrap(), i as u64);
            assert!(entry.mav_header.is_some());
            assert!(entry.mav_message.is_some());
        }
        temp_file.close().unwrap();
    }

    #[test]
    fn test_mav_log_parser_sub_parser_mixed_and_timestamp() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let mut packed_data: Vec<u8> = vec![
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            0, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];
        populate_data(false, true, &mut packed_data);

        temp_file
            .write(&packed_data)
            .expect("Failed to write test file");

        println!("Len of packed_data: {}", packed_data.len());

        let mut parser = MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(
            temp_file.path().to_str().unwrap(),
        );
        for i in 0..20 {
            // handle raw entry
            let entry = parser.parse_next_entry();
            assert!(entry.is_ok(), "Iteration: {i} {:?}", entry.err());
            let entry = entry.unwrap();
            assert!(entry.timestamp.is_some(), "Iteration: {i}");
            assert_eq!(entry.timestamp.unwrap(), i * 5 as u64);
            assert!(entry.mav_header.is_none());
            assert!(entry.mav_message.is_none());
            assert!(entry.raw.is_some());
            let raw = entry.raw.unwrap();
            assert_eq!(raw.len(), 10);
            assert_eq!(
                raw,
                vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD]
            );

            // handle text entry
            let entry = parser.parse_next_entry();
            assert!(entry.is_ok(), "Iteration: {i} {:?}", entry.err());
            let entry = entry.unwrap();
            assert!(entry.timestamp.is_some(), "Iteration: {i}");
            assert_eq!(entry.timestamp.unwrap(), i * 5 + 1 as u64);
            assert!(entry.mav_header.is_none());
            assert!(entry.mav_message.is_none());
            assert!(entry.raw.is_none());
            assert!(entry.text.is_some());
            let text = entry.text.unwrap();
            assert_eq!(text, "abcde");

            // handle mavlink entries
            for j in 0..3 {
                let entry = parser.parse_next_entry();
                assert!(entry.is_ok(), "Iteration: {i} {:?}", entry.err());
                let entry = entry.unwrap();
                assert!(entry.timestamp.is_some(), "Iteration: {i}");
                assert_eq!(entry.timestamp.unwrap(), i * 5 + 2 + j as u64);
                assert!(entry.mav_header.is_some());
                assert!(entry.mav_message.is_some());
            }
        }
        temp_file.close().unwrap();
    }

    #[test]
    fn test_mav_log_parser_sub_parser_mavlink_only_and_timestamp_handle_missing_timestamp() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let mut packed_data: Vec<u8> = vec![
            // file header
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // uuid
            16, 0, 0, 0, 0, 0, 0, 17, // timestamp_us
            b'a', b'p', b'p', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, // src_application_id
            1, 0, 0, 0, // format_version
            1, 0, // format_flags
            // message_definition
            2, 0, 0, 0, // version_major
            1, 0, 0, 0, // version_minor
            b't', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, // dialect
            0, 0, // payload_type
            0, 0, 0, 0, // size
        ];
        populate_data(true, true, &mut packed_data);

        // Remove the timestamp data from packed_data
        packed_data.drain(108..116);

        temp_file
            .write(&packed_data)
            .expect("Failed to write modified test file");

        let mut parser = MavLogParser::<mavlink::ardupilotmega::MavMessage>::new(
            temp_file.path().to_str().unwrap(),
        );

        // Check the first iteration
        let first_entry = parser.parse_next_entry();
        assert!(
            first_entry.is_ok(),
            "First iteration failed: {:?}",
            first_entry.err()
        );
        let first_entry = first_entry.unwrap();
        assert!(
            first_entry.timestamp.is_none(),
            "Expected no timestamp in the first entry"
        );
        assert!(
            first_entry.mav_message.is_some(),
            "Expected a mav_message in the first entry"
        );

        // Check subsequent iterations
        for i in 1..60 {
            let entry = parser.parse_next_entry();
            assert!(entry.is_ok(), "Iteration: {i} {:?}", entry.err());
            let entry = entry.unwrap();
            assert!(entry.timestamp.is_some(), "Iteration: {i}");
            assert_eq!(entry.timestamp.unwrap(), i as u64);
            assert!(entry.mav_header.is_some());
            assert!(entry.mav_message.is_some());
        }
        temp_file.close().unwrap();
    }

    fn populate_data(mavlink_only: bool, timestamp: bool, data: &mut Vec<u8>) {
        let mut msg = MAVLinkV2MessageRaw::new();
        let mut header = MavHeader {
            sequence: 0,
            system_id: 1,
            component_id: 2,
        };
        let mut heartbeat = HEARTBEAT_DATA {
            custom_mode: 0,
            mavtype: MavType::MAV_TYPE_QUADROTOR,
            autopilot: MavAutopilot::MAV_AUTOPILOT_PX4,
            base_mode: MavModeFlag::empty(),
            system_status: MavState::MAV_STATE_STANDBY,
            mavlink_version: 0x3,
        };
        let mut attitude = ATTITUDE_DATA {
            time_boot_ms: 0,
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            rollspeed: 0.0,
            pitchspeed: 0.0,
            yawspeed: 0.0,
        };
        let mut gps = GPS2_RAW_DATA {
            time_usec: 0,
            fix_type: GpsFixType::GPS_FIX_TYPE_RTK_FIXED,
            lat: 0,
            lon: 0,
            alt: 0,
            eph: 0,
            epv: 0,
            vel: 0,
            cog: 0,
            satellites_visible: 0,
            dgps_numch: 0,
            dgps_age: 0,
        };
        let mut time: u64 = 0u64;
        for _ in 0..20 {
            if !mavlink_only {
                // add a raw entry
                data.extend_from_slice(&[0x00]); // type
                if timestamp {
                    data.extend_from_slice(&time.to_le_bytes()); // timestamp
                }
                data.extend_from_slice(&10u16.to_le_bytes()); // size 10
                data.extend_from_slice(&[
                    0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD,
                ]); // payload
                time += 1;

                // add a text entry
                data.extend_from_slice(&[0x02]); // type
                if timestamp {
                    data.extend_from_slice(&time.to_le_bytes()); // timestamp
                }
                data.extend_from_slice(&5u16.to_le_bytes()); // size 5
                data.extend_from_slice(&[b'a', b'b', b'c', b'd', b'e']); // payload
                time += 1;
            }

            // add a mavlink entries
            header.sequence += 1;
            heartbeat.custom_mode += 1;
            msg.serialize_message(header, &MavMessage::HEARTBEAT(heartbeat.clone()));
            add_mavlink(mavlink_only, timestamp, data, &mut msg, time);
            time += 1;

            attitude.time_boot_ms += 1;
            attitude.roll += 1.0;
            attitude.pitch += 1.0;
            attitude.yaw += 1.0;
            attitude.rollspeed += 1.0;
            attitude.pitchspeed += 1.0;
            attitude.yawspeed += 1.0;
            msg.serialize_message(header, &MavMessage::ATTITUDE(attitude.clone()));
            add_mavlink(mavlink_only, timestamp, data, &mut msg, time);
            time += 1;

            gps.time_usec += 1;
            gps.lat += 1;
            gps.lon += 1;
            gps.alt += 1;
            gps.eph += 1;
            gps.epv += 1;
            gps.vel += 1;
            gps.cog += 1;
            gps.satellites_visible += 1;
            gps.dgps_numch += 1;
            gps.dgps_age += 1;
            msg.serialize_message(header, &MavMessage::GPS2_RAW(gps.clone()));
            add_mavlink(mavlink_only, timestamp, data, &mut msg, time);
            time += 1;
        }
    }

    fn add_mavlink(
        mavlink_only: bool,
        timestamp: bool,
        data: &mut Vec<u8>,
        msg: &mut MAVLinkV2MessageRaw,
        time: u64,
    ) {
        if !mavlink_only {
            data.extend_from_slice(&[0x01]); // type
        }
        if timestamp {
            data.extend_from_slice(&time.to_le_bytes()); // timestamp
        }
        if !mavlink_only {
            data.extend_from_slice(&(msg.raw_bytes().len() as u16).to_le_bytes());
            // size
        }
        data.extend_from_slice(&msg.raw_bytes()); // payload
    }
}
