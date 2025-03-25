# Mav Log File Format

This document describes a log file format for log data centered around mavlink but where some flexibility might be needed. It includes a header to support that flexibility.

## Overview

On log file creation, the file header as specified below should be written. After the header is the mavlink message definitions. The mavlink message definitions can be captured via one of the formats captured in [Mavlink Definition Payload Type](#mavlink-definition-payload-type-enum). After the header, any number of entries can be added. This file can contain any data. Each entry into the file will have a header and payload.

It is expected that endianness match the mavlink spec for [pack format](https://mavlink.io/en/guide/serialization.html#packet_format) (little-endian).

## File Extension

".mav" is strongly encouraged. ".bin" should be supported.

## Sections

1. Header
1. Mavlink Definitions
1. Entries

## File Header (62 bytes)

| Field              | C Type   | Description                                                                                                                                |
| :----------------- | :------- | :----------------------------------------------------------------------------------------------------------------------------------------- |
| uuid               | char[16] | A unique identifier for this log file.                                                                                                     |
| timestamp_us       | uint64_t | Unix timestamp that notes when logging started in microseconds.                                                                            |
| src_application_id | char[32] | A string intended to uniquely represent the application creating the log file. (ie mavlink_logger)                                         |
| format_version     | uint32_t | Version number for this file format as determined by this documentation. Currently 1. 0 means a custom format is being used.               |
| format_flags       | uint16_t | (Bitmask) Set of flags to allow for various format changes. 0 means none of the flags apply. See [Format Flags](#format-flags-enum) below. |

### Format Flags Enum

This is a bitmask meaning each entry should double the previous.

| Value | Name         | Description                                                     |
| :---- | :----------- | :-------------------------------------------------------------- |
| 1     | MAVLINK_ONLY | Flag indicating this file only contains packed mavlink content. |
| 2     | NO_TIMESTAMP | Flag indicating each entity has a timestamp                     |

## Mavlink Message Definitions (46 bytes without payload)

| Field             | C Type   | Description                                                                          |
| :---------------- | :------- | :----------------------------------------------------------------------------------- |
| mav_version_major | uint32_t | MAVLink protocol major version.                                                      |
| mav_version_minor | uint32_t | MAVLink protocol minor version.                                                      |
| mav_dialect       | char[32] | [mavlink message dialect](https://mavlink.io/en/messages/) being used.               |
| payload_type      | uint16_t | [Mavlink Definition Payload Type](#mavlink-definition-payload-type-enum)             |
| size              | uint32_t | Size of the following payload in bytes. 0 if definition xml file is not retrievable. |
| payload           | N/A      | This content type is determined by the payload_type field above.                     |

### Mavlink Definition Payload Type Enum

This determines what information will be provided in the payload field for the MAVLink messages definitions.

| Value | Name                 | Description                                                                                      |
| :---- | :------------------- | :----------------------------------------------------------------------------------------------- |
| 0     | NONE                 | No message definitions are provided.                                                             |
| 1     | SPACE_DELIMITED_URLS | A UTF-8 encoded string as a set of whitespace separated urls pointing to the relevant XML files. |
| 2     | XML                  | UTF-8 encoded XML definitions.                                                                   |

## Entries (0-11 bytes without payload)

As many entries as there are room to write can be appended to the file content post mavlink definitions. Each entry could have up to the following structure. Each field in the following structure is optional as determined by the flags listed above.

| Field        | C Type   | Description                                                                                                                                    |
| :----------- | :------- | :--------------------------------------------------------------------------------------------------------------------------------------------- |
| type         | uint8_t  | This indicates the payload type. See [Entry Type](#entry-type-enum) below. This field is NOT present if the MAVLINK_ONLY flag is set.          |
| timestamp_us | uint64_t | Unix timestamp in microseconds for which this corresponding payload was acted upon. This field is NOT present if the NO_TIMESTAMP flag is set. |
| size         | uint16_t | Size of the entry in bytes without the header. This field is NOT present if the MAVLINK_ONLY flag is set.                                      |
| payload      | N/A      | Any bytes content.                                                                                                                             |

### Entry Type Enum

| Value | Name    | Description                  |
| :---- | :------ | :--------------------------- |
| 0     | RAW     | Catch all for raw bytes data |
| 1     | MAVLINK | Entry is a mavlink message   |
| 2     | TEXT    | Entry is UTF-8 encoded text  |
