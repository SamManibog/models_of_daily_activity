//! Defines encodings and transformations of data used by the project. Useful for decreasing
//! loading or processing times.

use std::{collections::BTreeMap, io::Write};

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct OriginalRecord {
    #[serde(rename = "YEAR")]
    year: u32,

    #[serde(rename = "CASEID")]
    caseid: u64,

    #[serde(rename = "SERIAL")]
    serial: u64,

    #[serde(rename = "FAMINCOME")]
    family_income: u32,

    #[serde(rename = "HHTENURE")]
    tenure: u32,

    #[serde(rename = "HOUSETYPE")]
    housetype: u32,

    #[serde(rename = "PERNUM")]
    person_number: u32,

    #[serde(rename = "LINENO")]
    line_number: u32,

   #[serde(rename = "WT06")]
    weight: f64,

    #[serde(rename = "SCHLCOLL")]
    schooling: u32,

    #[serde(rename = "ACTIVITY")]
    activity: u64,

    #[serde(rename = "START")]
    start: String,

    #[serde(rename = "STOP")]
    stop: String
}

/// converts time in the form "hours:minutes:seconds" to seconds after midnight
fn time_to_secs_after_midnight(time_str: &str) -> i32 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        panic!("Invalid time format: {}", time_str);
    }

    let hours = parts[0].parse::<u32>().unwrap() as i32;
    assert!(hours >= 0 && hours < 24, "Hours must be between 0 and 23");
    let minutes = parts[1].parse::<u32>().unwrap() as i32;
    assert!(minutes >= 0 && minutes < 60, "Minutes must be between 0 and 59");
    let seconds = parts[2].parse::<u32>().unwrap() as i32;
    assert!(seconds >= 0 && seconds < 60, "Seconds must be between 0 and 59");

    (hours * 60 + minutes) * 60 + seconds
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RemappedRecord {
    /// the year the data was collected
    year: u32,

    /// the id of the person surveyed
    serial: u64,

    /// the activity completed, remapped to a smaller set of categories
    activity: u8,

    /// the time the activity started in seconds after midnight
    start: i32,

    /// the time the activity started in seconds after midnight
    stop: i32
}

impl From<OriginalRecord> for Option<RemappedRecord> {
    fn from(record: OriginalRecord) -> Self {
        let start_parsed = time_to_secs_after_midnight(&record.start);
        let stop_parsed = time_to_secs_after_midnight(&record.stop);

        Some(RemappedRecord {
            year: record.year,
            serial: record.serial,
            activity: ActivityCategory::from_original_code(record.activity as u32)?.into_code(),
            start: start_parsed,
            stop: stop_parsed
        })
    }
}

/// a record of a single activity
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ActivityRecord {
    day_id: u32,
    start: i32,
    stop: i32,
    activity: u8,
}

#[derive(Debug, Hash)]
struct DayId {
    year: u32,
    case_id: u64,
}

impl PartialEq for DayId {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year && self.case_id == other.case_id
    }
}

impl Eq for DayId {}

impl PartialOrd for DayId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DayId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.case_id.cmp(&other.case_id) {
            std::cmp::Ordering::Equal => self.year.cmp(&other.year),
            other => other
        }
    }
}

/// the category of an activity performed
pub enum ActivityCategory {
    Sleeping,
    PersonalCare,
    HouseholdChores,
    Childcare,
    AdultCare,
    Work,
    Classes,
    NonAthleticExtracurricular,
    Homework,
    OtherEducation,
    Shopping,
    Services,
    CivicDuties,
    EatingDrinking,
    Leisure,
    Exercise,
    ReligiousActivities,
    Volunteering,
    Calls,
    Travel,
    MissingData
}

impl ActivityCategory {
    pub const MAX_CODE: usize = 20;

    /// gets an instance of self based on the original encoding for activities
    pub fn from_original_code(code: u32) -> Option<Self> {
        match code {
            010100..=010199 => Some(Self::Sleeping),

            010200..=019999 | 100000 => Some(Self::PersonalCare),

            020000..=029999 => Some(Self::HouseholdChores),

            030000..=030399 | 040000..=040399 => Some(Self::Childcare),

            030400..=039999 | 040400..=049999 => Some(Self::AdultCare),

            050000..=059999 => Some(Self::Work),

            060000..=060199 => Some(Self::Classes),

            060200..=060299 => Some(Self::NonAthleticExtracurricular),

            060300..=060399 => Some(Self::Homework),

            060400..=069999 => Some(Self::OtherEducation),

            070000..=079999 => Some(Self::Shopping),

            080000..=100199 | 100304 | 100400..=109999 => Some(Self::Services),

            100200..=100299 | 100303 | 100399 => Some(Self::CivicDuties),

            110000..=119999 => Some(Self::EatingDrinking),

            120000..=129999 | 130200..=130399 => Some(Self::Leisure),

            130000..=130199 | 130400..=139999 => Some(Self::Exercise),

            140000..=149999 => Some(Self::ReligiousActivities),

            150000..=159999 => Some(Self::Volunteering),

            160000..=169999 => Some(Self::Calls),

            180000..=189999 => Some(Self::Travel),

            500000..=509999 => Some(Self::MissingData),

            _ => None,
        }
    }

    /// gets a &str naming the cateogry
    pub fn into_str(&self) -> &'static str {
        match self {
            Self::Sleeping => "Sleeping",
            Self::PersonalCare => "Personal Care",
            Self::HouseholdChores => "Household Chores",
            Self::Childcare => "Childcare",
            Self::AdultCare => "Adult Care",
            Self::Work => "Work",
            Self::Classes => "Classes",
            Self::NonAthleticExtracurricular => "Extracurricular",
            Self::Homework => "Homework",
            Self::OtherEducation => "Other Education",
            Self::Shopping => "Shopping",
            Self::Services => "Services",
            Self::CivicDuties => "Civic Duties",
            Self::EatingDrinking => "Eating and Drinking",
            Self::Leisure => "Leisure",
            Self::Exercise => "Exercise",
            Self::ReligiousActivities => "Religious Activities",
            Self::Volunteering => "Volunteering",
            Self::Calls => "Calls",
            Self::Travel => "Travel",
            Self::MissingData => "Missing Data",
        }
    }

    /// gets an instance from the code used to internally represent the activity
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Sleeping),
            1 => Some(Self::PersonalCare),
            2 => Some(Self::HouseholdChores),
            3 => Some(Self::Childcare),
            4 => Some(Self::AdultCare),
            5 => Some(Self::Work),
            6 => Some(Self::Classes),
            7 => Some(Self::NonAthleticExtracurricular),
            8 => Some(Self::Homework),
            9 => Some(Self::OtherEducation),
            10 => Some(Self::Shopping),
            11 => Some(Self::Services),
            12 => Some(Self::CivicDuties),
            13 => Some(Self::EatingDrinking),
            14 => Some(Self::Leisure),
            15 => Some(Self::Exercise),
            16 => Some(Self::ReligiousActivities),
            17 => Some(Self::Volunteering),
            18 => Some(Self::Calls),
            19 => Some(Self::Travel),
            20 => Some(Self::MissingData),

            _ => None,
        }
    }

    /// converts into the code used to internally represent the data
    /// we guarantee:
    /// 	1) codes are positive
    /// 	2) codes are consecutive starting from 0
    /// 	3) the greatest value code corresponds to Self::MissingData
    pub fn into_code(&self) -> u8 {
        match self {
            Self::Sleeping => 0,
            Self::PersonalCare => 1,
            Self::HouseholdChores => 2,
            Self::Childcare => 3,
            Self::AdultCare => 4,
            Self::Work => 5,
            Self::Classes => 6,
            Self::NonAthleticExtracurricular => 7,
            Self::Homework => 8,
            Self::OtherEducation => 9,
            Self::Shopping => 10,
            Self::Services => 11,
            Self::CivicDuties => 12,
            Self::EatingDrinking => 13,
            Self::Leisure => 14,
            Self::Exercise => 15,
            Self::ReligiousActivities => 16,
            Self::Volunteering => 17,
            Self::Calls => 18,
            Self::Travel => 19,
            Self::MissingData => 20
        }
    }

    /// returns an iterator over all categories excluding MissingData
    pub fn valid_iter() -> impl Iterator<Item = Self> {
        (0..Self::MAX_CODE).into_iter()
        .map(|c| Self::from_code(c as u8).unwrap())
    }
}

pub fn block_remap(block_duration: usize, input: &str, output: &str) {
    debug_assert!(60*24 % block_duration == 0, "block duration must divide evenly into a day");

    let mut reader = csv::Reader::from_path(input).unwrap();
    let mut output_file = std::fs::File::create(format!("{output}.ablk")).expect("failed to create output file");

    let mut map = BTreeMap::<u32, Vec<ActivityRecord>>::new();
    for result in reader.deserialize() {
        let record: ActivityRecord = result.expect("failed to deserialize record");
        map.entry(record.day_id).or_insert_with(Vec::new).push(record);
    }

    let blocks_per_day: [u8; 4] = ((60 * 24 / block_duration) as u32).to_le_bytes();
    let day_count: [u8; 8] = (map.len() as u64).to_le_bytes();

    output_file.write(&blocks_per_day).expect("failed to write blocks per day to file");
    output_file.write(&day_count).expect("failed to write day count to file");

    for records in map.values() {
        let blocks = get_day_blocks(block_duration, records);

        /*
        let text = blocks.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ");
        output_file.write_all(text.as_bytes()).expect("failed to write text to file");
        output_file.write_all(b"\n").expect("failed to write newline to file");
        */

        output_file.write_all(&blocks).expect("failed to write block to file");
    }

    output_file.flush().expect("failed to flush output file");
}

/// gets the entire activity record for a given day, given as a list of activity codes for the day
fn get_day_blocks(block_duration: usize, records: &[ActivityRecord]) -> Vec<u8> {
    let num_blocks = 60 * 24 / block_duration;
    let mut blocks = Vec::with_capacity(num_blocks);

    for block_index in 0..num_blocks {
        blocks.push(get_block(block_duration, block_index, records));
    }

    blocks
}

/// gets the activity of a given block, given a list of records for the day
fn get_block(block_duration: usize, block_index: usize, records: &[ActivityRecord]) -> u8 {
    let block_start = block_index * block_duration * 60;
    let block_end = (block_index + 1) * block_duration * 60;

    let mut seconds_per_code = [0; ActivityCategory::MAX_CODE];
    for record in records {
        let time: i32 = if record.start < record.stop {
            record.stop.clamp(block_start as i32, block_end as i32)
            - record.start.clamp(block_start as i32, block_end as i32)
        } else {
            // sometimes the activity goes past midnight, in which case the above method won't work
            // for determining time spent in an activity during this block
            block_end as i32 - record.start.clamp(block_start as i32, block_end as i32)
        };

        // ignore missing data
        if record.activity != ActivityCategory::MAX_CODE as u8 {
            seconds_per_code[record.activity as usize] += time;
        }
    }

    // determine the most performed activity during the block
    let mut max_seconds = 0;
    let mut max_code = ActivityCategory::MAX_CODE as u8;
    for (i, &seconds) in seconds_per_code.iter().enumerate() {
        // total_seconds += seconds;
        if seconds > max_seconds {
            max_seconds = seconds;
            max_code = i as u8;
        }
    }

    // if no activity is found, return missing data
    if max_seconds == 0 {
        ActivityCategory::MAX_CODE as u8
    } else {
        max_code
    }
}

pub fn day_id_remap(input: &str, output: &str) {
    let mut reader = csv::Reader::from_path(input).unwrap();
    let mut writer = csv::Writer::from_path(output).unwrap();

    let mut map = BTreeMap::<DayId, u32>::new();
    let mut id_counter = 0;
    for result in reader.deserialize() {
        let record: RemappedRecord = result.unwrap();
        let day_id = DayId {
            year: record.year,
            case_id: record.serial
        };
        let this_id = match map.entry(day_id) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(id_counter);
                id_counter += 1;
                id_counter - 1
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                *entry.get()
            }
        };
        let activity_record = ActivityRecord {
            day_id: this_id,
            start: record.start,
            stop: record.stop,
            activity: record.activity
        };
        let _ = writer.serialize(activity_record).expect("Failed to write record");
    }
}

pub fn remap_original(input: &str, output: &str) {
    let mut reader = csv::Reader::from_path(input).unwrap();
    let mut writer = csv::Writer::from_path(output).unwrap();
    for result in reader.deserialize() {
        let record: OriginalRecord = result.unwrap();
        let remapped_record = Option::<RemappedRecord>::from(record)
            .expect("unable to remap activity code");
        let _ = writer.serialize(remapped_record).expect("Failed to write record");
    }
    writer.flush().expect("Failed to flush writer");
}
