# Tlog File Format

This document describes the file format of mavlink telemetry logs as has been informally adopted by the open source ground station community. This file typically has a .tlog file extension.

## Existing Documentation

- [QGC](https://docs.qgroundcontrol.com/master/en/qgc-dev-guide/file_formats/mavlink.html)
- [Mission Planner](https://ardupilot.org/planner/docs/mission-planner-telemetry-logs.html)

## File Extension

.tlog

## Sections

1. entries

## Entries

As many entries as there are room to write can be appended to the file. Each entry will have the following structure.

| Field        | C Type   | Description                                                                           |
| :----------- | :------- | :------------------------------------------------------------------------------------ |
| timestamp_us | uint64_t | Timestamp in microseconds for which entry was logged                                  |
| payload      | N/A      | [MAVLink Serialization Documentation](https://mavlink.io/en/guide/serialization.html) |
