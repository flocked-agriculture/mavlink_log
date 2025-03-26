# Mavlink Log

[![Crates.io Version](https://img.shields.io/crates/v/mavlink_log.svg)](https://crates.io/crates/mavlink_log)
![Build + Test](https://github.com/flocked-agriculture/mavlink_log/actions/workflows/main_ci.yml/badge.svg?branch=main)

## Installation

`cargo add mavlink_log`

## File Formats

[.mav log](docs/mav_log_file_format.md) - requires the MavLog feature
[.tlog log](docs/tlog_file_format.md) - requires the Tlog feature

## Examples

TBD - generate examples with the different features

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
- allow optional buffering during writing
- use rust features to select for certain optimizations such as no timestamps or mavlink only
- support no copy logging if possible and necessary
- add support for logging in embedded systems
