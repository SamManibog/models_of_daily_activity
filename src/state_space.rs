use std::{fs::File, io::{BufReader, Read}};

use crate::encodings::ActivityCategory;

struct BlockStateChangeMatrixPrecursor {
    /// the number of times a change from activity i to activity j occurs in the data
    counts: [[u32; ActivityCategory::MAX_CODE]; ActivityCategory::MAX_CODE],
}

impl BlockStateChangeMatrixPrecursor {
    pub fn new() -> Self {
        Self {
            counts: [[0; ActivityCategory::MAX_CODE]; ActivityCategory::MAX_CODE]
        }
    }

    pub fn from_block_encoding(
        filename: &str
    ) -> Vec<Box<Self>> {
        // get reader
        let f = File::open(filename).expect("unable to open file");
        let mut reader = BufReader::new(f);

        // interpret header of block file
        let mut blocks_per_day = [0; 4];
        reader.read_exact(&mut blocks_per_day).expect("unable to read 'blocks per day' of activity block file");
        let blocks_per_day = u32::from_le_bytes(blocks_per_day) as usize;

        let mut day_count = [0; 8];
        reader.read_exact(&mut day_count).expect("unable to read 'day count' of activity block file");
        let day_count = u64::from_le_bytes(day_count) as usize;

        // allocate the necessary precursors (1 less than BLOCK_COUNT)
        let mut precursors = Vec::with_capacity(blocks_per_day);
        for _ in 0..blocks_per_day {
            precursors.push(Box::new(Self::new()));
        }

        let mut activities: Vec<u8> = vec![0; blocks_per_day];
        for i in 0..day_count {
            reader.read_exact(&mut activities).unwrap_or_else(|_| panic!("unable to read day index {}", i));
            let mut previous = activities[0];
            for (block_idx, activity) in activities.iter().skip(1).enumerate() {
                precursors[block_idx].add_change(previous, *activity);
                previous = *activity;
            }
        }

        precursors
    }

    pub fn add_change(&mut self, from: u8, to: u8) {
        if from < ActivityCategory::MAX_CODE as u8 && to < ActivityCategory::MAX_CODE as u8 {
            self.counts[from as usize][to as usize] += 1;
        }
    }

    pub fn get_change_count(&self, from: usize, to: usize) -> u32 {
        self.counts[from][to]
    }
}

#[derive(Debug)]
pub struct BlockStateChangeMatrix {
    /// probabilities[i][j] - probabilities[i][j - 1] (or 0 if j == 0) is the probability that
    /// a change from activity i to activity j occurs, given that we are initially in activity i
    probabilities: [[f64; ActivityCategory::MAX_CODE]; ActivityCategory::MAX_CODE]
}

impl BlockStateChangeMatrix {
    /// creates a state change matrix for a given block of the day
    /// provide the number of blocks in the day 
    pub fn from_block_encoding(
        filename: &str
    ) -> Vec<Box<Self>> {
        BlockStateChangeMatrixPrecursor::from_block_encoding(filename)
            .iter()
        	.map(|p| Box::new(Self::from_precursor(p)))
            .collect()
    }

    fn from_precursor(precursor: &BlockStateChangeMatrixPrecursor) -> Self {
        let mut probabilities = [[0.0; ActivityCategory::MAX_CODE]; ActivityCategory::MAX_CODE];
        for i in 0..ActivityCategory::MAX_CODE {
            let mut total_changes_from_i: u32 = 0;
            for j in 0..ActivityCategory::MAX_CODE {
                total_changes_from_i += precursor.get_change_count(i, j);
            }

            let mut cumulative_probability = 0.0;
            if total_changes_from_i == 0 {
                for j in 0..ActivityCategory::MAX_CODE {
                    cumulative_probability += 1.0 / (ActivityCategory::MAX_CODE) as f64;
                    probabilities[i][j] = cumulative_probability;
                }
            } else {
                for j in 0..ActivityCategory::MAX_CODE {
                    cumulative_probability += precursor.get_change_count(i, j) as f64 / total_changes_from_i as f64;
                    probabilities[i][j] = cumulative_probability;
                }
            }
        }
        Self { probabilities }
    }

    /// gets a random activity to transition to, given the current activity
    pub fn get_random_transition(&self, from: u8) -> u8 {
        let rand: f64 = rand::random();
        for (to, &cumulative_probability) in self.probabilities[from as usize].iter().enumerate() {
            if rand <= cumulative_probability {
                return to as u8;
            }
        }
        return (ActivityCategory::MAX_CODE - 1) as u8;
    }
}

