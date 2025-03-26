# Mavlink Log

[![Crates.io Version](https://img.shields.io/crates/v/mavlink_log.svg)](https://crates.io/crates/mavlink_log)
[![Documentation](https://docs.rs/mavlink_log/badge.svg)](https://docs.rs/mavlink_log)
![Build + Test](https://github.com/flocked-agriculture/mavlink_log/actions/workflows/main_ci.yml/badge.svg?branch=main)
![Build + Test + Deploy](https://github.com/flocked-agriculture/mavlink_log/actions/workflows/release.yml/badge.svg)
![GitHub Issues](https://img.shields.io/github/issues/flocked-agriculture/mavlink_log.svg?style=flat-square)

## Installation

`cargo add mavlink_log`

## File Formats

[.mav log](docs/mav_log_file_format.md) - requires the MavLog feature
[.tlog log](docs/tlog_file_format.md) - requires the Tlog feature

## Known Issues

### read_versioned_msg

> **NOTE**
> This is only relevant when parsing a .mav file that does not have the mavlink_only format flag set to true or no_timestamp is false.

The read_versioned_msg function from rust_mavlink is not built for mixed data streams which can cause problems on the mixed stream log file. Specifically it will search through the data for the mavlink packet start key. This only a problem for our parser if the file data got corrupted or an unexpected message defintion was used. Meaning it tried to read the current mavlink packet and failed. This method will immediately search for the next valid MAVLink message. It should in theory recover on the next mavlink message but until then, it is looping over potentially valid non mavlink file records or there could be false positives. This can result in misaligned timestamps as well.

We would rather it exit on failure so we can take the data as raw and move on to the next entry.

## Examples

> **WARNING**
> The following examples are were pieced together from the automated testing. They are meant to illustrate the usage concept and thus could err if attempting to run on your system.

### Mav File Logging

features: mavlog, logger

```rust
use mavlink_log::mav_logger::MavLogger;
use mavlink_log::mavlog::header::FormatFlags;
use mavlink_log::mavlog::logger::RotatingMavLogger;
use mavlink::common::MavMessage;
use mavlink::{MavHeader, MavlinkVersion, MavFrame};

fn main() {
    // initialize a mav logger with no optimizations
    // this means all entries will be timestamped and raw, text, mavlink are all supported
    let mut logger: RotatingMavLogger =
            RotatingMavLogger::new("/tmp/ground_station.mav", 1024, 3, None, None)
                .expect("Failed to create logger");

    // example mav frame. this would most likely be received from a mavlink connection
    let mav_frame = MavFrame {
        header: MavHeader::default(),
        msg: MavMessage::HEARTBEAT(Default::default()),
        protocol_version: MavlinkVersion::V2,
    };

    // write mavlink
    let result = logger.write_mavlink(mav_frame);
    assert!(result.is_ok());

    // write text
    let result = logger.write_text("Test log entry");
    assert!(result.is_ok());

    // write raw
    let result = logger.write_raw(&[1, 2, 3, 4, 5]);
    assert!(result.is_ok());

    // initialize a mav logger with optimizations
    let flags = FormatFlags {
        mavlink_only: true,
        no_timestamp: true,
    };
    let mut logger: RotatingMavLogger =
            RotatingMavLogger::new("/tmp/ground_station.mav", 1024, 3, Some(flags), None)
                .expect("Failed to create logger");

    // example mav frame. this would most likely be received from a mavlink connection
    let mav_frame = MavFrame {
        header: MavHeader::default(),
        msg: MavMessage::HEARTBEAT(Default::default()),
        protocol_version: MavlinkVersion::V2,
    };

    // write mavlink
    let result = logger.write_mavlink(mav_frame);
    assert!(result.is_ok());

    // write text
    let result = logger.write_text("Test log entry");
    assert!(!result.is_ok());

    // write raw
    let result = logger.write_raw(&[1, 2, 3, 4, 5]);
    assert!(!result.is_ok());
}
```

### Mav File Parsing

features: mavlog, parser

```rust
use std::io::ErrorKind::UnexpectedEof;
use mavlink::common::MavMessage;
use mavlink::error::MessageReadError;
use mavlink_log::mavlog::parser::MavLogParser;
use mavlink_log::mav_parser::{LogEntry, MavParser};

fn main() {
    let mut parser = MavLogParser::<MavMessage>::new("/tmp/ground_station.mav");
    let mut count: u64 = 0;
    loop {
        // retrieve next entry in a loop
        let entry: Result<LogEntry<MavMessage>, MessageReadError> = parser.parse_next_entry();
        match entry {
            Ok(entry_data) => {
                count += 1;
                // optionally get timestamp
                let timestamp = entry_data.timestamp;
                // optionally get the mavlink header
                let mav_header = entry_data.mav_header;
                // optionally get the mavlink message
                let mav_message = entry_data.mav_message;
                // optionally get text
                let text = entry_data.text;
                // optionally get raw
                let raw = entry_data.raw;
            }
            // exit loop on end of file
            Err(MessageReadError::Io(e)) => {
                if e.kind() == UnexpectedEof {
                    break
                }
            },
            Err(_) => {},
        }
    }
}

```

### Tlog File Logging

features: tlog, logger

```rust
use mavlink_log::mav_logger::MavLogger;
use mavlink_log::tlog::logger::RotatingTlog;
use mavlink::common::MavMessage;
use mavlink::{MavHeader, MavlinkVersion, MavFrame};

fn main(){
    // example mav frame. this would most likely be received from a mavlink connection
    let mav_frame = MavFrame {
        header: MavHeader::default(),
        msg: MavMessage::HEARTBEAT(Default::default()),
        protocol_version: MavlinkVersion::V2,
    };

    // initialize rotating tlog logger
    let mut logger = RotatingTlog::new("/tmp/ground_station.tlog", 1024, 3).unwrap();
    // write a mavlink frame
    let result = logger.write_mavlink(mav_frame);
    assert!(result.is_ok());
}
```

### Tlog File Parsing

features: tlog, parser

```rust
use std::io::ErrorKind::UnexpectedEof;
use mavlink::common::MavMessage;
use mavlink::error::MessageReadError;
use mavlink_log::mav_parser::{LogEntry, MavParser};
use mavlink_log::tlog::parser::TlogParser;

fn main() {
    // initialize the tlog parser
    let mut tlog = TlogParser::<MavMessage>::new("/tmp/ground_station.tlog");
    let mut count: u64 = 0;
    loop {
        // retrieve next entry in a loop
        let entry: Result<LogEntry<MavMessage>, MessageReadError> = tlog.parse_next_entry();
        match entry {
            Ok(entry_data) => {
                count += 1;
                // optionally get the mavlink header
                let mav_header = entry_data.mav_header;
                // optionally get the mavlink message
                let mav_message = entry_data.mav_message;
            }
            // exit loop on end of file
            Err(MessageReadError::Io(e)) => {
                if e.kind() == UnexpectedEof {
                    break
                }
            },
            Err(_) => {},
        }
    }
}
```

## License

Licensed under either of the following:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contribution

Public contribution is welcome and encouraged. Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

### Documentation

All documentation that is associated with a specific block of code, should be documented inline per the rust documentation expectations. This documentation should be available in the auto geenrated rust docs. Documentation of more complex technical concepts surrounding this crate as a whole should be put in markdown files in the docs folder. Everything else such as crate requirements, installation instructions, etc should be documented in this README.md.

### Code Standards

- all code should be formatted per cargo's default fmt command
- code should target 80% automated code coverage

### Release Process

Releases are managed through both git tags and branches. Branches are used for convenience and tags actually trigger the relevant release actions. Whenever there is a new major or minor release a branch must be created at the relevant hash in the format v\<major\>.\<minor\> (ie v1.33). Branches with such a format are protected by a ruleset and can only be modified by admins. All release tags must point to hashes on said branch. There is also a ruleset protecting all git tags matching the semantic versioning format v*.*.\* so that only admins can add such tags.

#### Major or Minor Release

In summary, you must be an admin and complete the following steps:

- pick a hash
- confirm all automated tests have passed
- create a branch at the relevant hash in the format v\<major\>.\<minor\> (ie v1.33).
- if necessary perform any last minuted changes
- create a git tag pointing to the tip of that branch in the format v\<major\>.\<minor\>.0 (ie v1.33.0).

The git tag will kick off an automated process that deploys the crate to crates.io after validating crate version matches the tag version and all automated tests pass.

## Future Work

- URGENT - create a new read_versioned_msg for the more complex log file types
- return error rather than panic even on critical errors so a parent can potentially handle and take action
- extract timestamps from tlog while parsing
- support async
- allow optional buffering during writing. maybe use features to support this.
- use rust features to select for certain optimizations such as no timestamps or mavlink only
- support no copy logging if possible and necessary
- add support for logging in embedded systems
