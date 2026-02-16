# Data Formats

## Activity Block Format (.ablk)

A less-processed form of data where each day is represented as a list of unsigned bytes where each byte is mapped to an activity performed in the day. The time this acitivty occurs is determined by the position of the byte relative to the start of the day and the number of blocks in the day.

For example, in a 24 block day, the first byte in that day corresponds to the activity performed in the first hour of the day. If this were a 48 block day, it would correspond to the one occuring in the first half-hour.

Note: Data is little endian.


| Bytes | Description |
| ----- | ----- |
| 0-3   | the number of blocks in each day |
| 4-11 | the number of days in the file|
| 12+ | byte data describing the actions in each block |

