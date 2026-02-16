# Data Formats

## Data Recategorization

Data from the original source has been recategorized/remaped as follows:

| New Category | New Code | Original Codes |
| ----: | :----: | :---- |
| Sleeping | (1) | 010100 - 010199 |
| Personal Care | (2) | 010200 - 019999 and 100000 |
| Household Chores | (3) | 020000 - 029999 |
| Childcare | (4) | 030000 - 030399 and 040000 - 040399 |
| AdultCare | (5) | 030400 - 039999 and 040400 - 049999 |
| Work | (6) | 050000 - 059999 |
| Classes/Lecture | (7) | 060000 - 060199 |
| non-Sports extracir | (8) | 060200 - 060299 |
| Homework | (9) | 060300 - 060399 |
| Other educ | (10) | 060400 - 069999 |
| Shopping | (11) | 070000 - 079999 |
| Services | (12) | 080000 - 100199 and 100304 and 100400 - 109999 |
| Civic Duties | (13) | 100200 - 100299 and 100303 and 100399 |
| Eating and Drinking | (14) | 110000 - 119999 |
| Leisure | (15) | 120000 - 129999 and 130200 - 130399 |
| Exercise | (16) | 130000 - 130199 and 130400 - 139999 |
| Religious Activities | (17) | 140000 - 149999 |
| Volunteering | (18) | 150000 - 159999 |
| Calls | (19) | 160000 - 169999 |
| Travel | (20) | 180000 - 189999 |
| MissingData | (21) | 500000 - 509999 |


## Activity Block Format (.ablk)

A format for storing activity data from multiple independent days.

Each day is represented by a string of activity codes (one byte each). The time this acitivty occurs is determined by the index of the activity, and the number of activity blocks in the day.

For example, in a 24 block day, the first code in that day corresponds to the activity performed from 12:00AM to 12:59AM in the first hour of the day, while the second code corresponds to the activity performed from 01:00AM to 01:59AM. However, for a 48 block day, the first code would be for 12:00AM - 12:29AM and the second for 12:30AM - 12:59AM.

The first 12 bytes are the little endian header of the file, and the rest are activity codes stored contiguously.O


| Bytes | Description |
| ----- | ----- |
| 0-3   | the number of blocks in each day |
| 4-11 | the number of days in the file|
| 12+ | byte data describing the actions in each block |

