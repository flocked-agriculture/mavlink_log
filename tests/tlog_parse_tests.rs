/// This module contains tests for parsing TLOG files using the `TlogParser`
/// from the `mavlink_log_parser` crate. It verifies the ability to read and
/// process MAVLink messages from a TLOG file.
#[cfg(all(feature = "tlog", feature = "parser"))]
mod tlog_parse_tests {
    use mavlink::ardupilotmega::MavMessage;
    use mavlink::error::MessageReadError;
    use mavlink_log::mav_parser::{LogEntry, MavParser};
    use mavlink_log::tlog::parser::TlogParser;

    /// This test verifies that the `TlogParser` can correctly parse a TLOG file
    /// by counting the number of MAVLink messages it contains. It uses a sample
    /// TLOG file located at `tests/data/tlog_data_0.tlog` and asserts that the
    /// total number of messages parsed matches the expected value (1426).
    ///
    /// # Panics
    /// The test will panic if the number of parsed messages does not match
    /// the expected count.
    #[test]
    fn test_tlog_parse() {
        let mut tlog = TlogParser::<MavMessage>::new("tests/data/tlog_data_0.tlog");
        let mut count: u64 = 0;
        loop {
            let entry: Result<LogEntry<MavMessage>, MessageReadError> = tlog.parse_next_entry();
            match entry {
                Ok(_) => {
                    count += 1;
                }
                Err(_) => break,
            }
        }
        assert_eq!(count, 1426);
    }
}
