//! Implementation of a Markovian DayForecaster

use std::{cmp::Ordering, collections::BTreeMap, fs::File, io::{BufReader, Read}, rc::Rc};

use crate::{day_forecaster::{DayForecaster, Forecast}, encodings::ActivityCategory};

#[derive(Debug)]
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

        // interpret header of ablk file: blocks per day
        let mut blocks_per_day = [0; 4];
        reader.read_exact(&mut blocks_per_day).expect("unable to read 'blocks per day' of activity block file");
        let blocks_per_day = u32::from_le_bytes(blocks_per_day) as usize;

        // interpret header of ablk file: number of days
        let mut day_count = [0; 8];
        reader.read_exact(&mut day_count).expect("unable to read 'day count' of activity block file");
        let day_count = u64::from_le_bytes(day_count) as usize;

        // allocate the necessary precursors (1 less than BLOCK_COUNT)
        let mut precursors = Vec::with_capacity(blocks_per_day);
        for _ in 0..blocks_per_day - 1 {
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
        let precursors = BlockStateChangeMatrixPrecursor::from_block_encoding(filename);
        //println!("last precursor:\n{:?}", precursors.last().unwrap());
        precursors.iter()
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

#[derive(Debug)]
pub struct MarkovChain<const BLOCK_DURATION: u32> {
    matrices: Vec<Box<BlockStateChangeMatrix>>,
}

impl<const BLOCK_DURATION: u32>  MarkovChain<BLOCK_DURATION> {
    const SIMULATION_DENSITY: usize = 100;//50000;//1;

    /// creates a markov chain change matrix for a given block of the day
    /// provide the number of blocks in the day 
    pub fn from_block_encoding(
        filename: &str
    ) -> Self {
        let matrices = BlockStateChangeMatrix::from_block_encoding(filename);
        //println!("last matrix:\n{:?}", matrices.last().unwrap());

        if matrices.len() != (24 * 60 / BLOCK_DURATION - 1) as usize {
            panic!(
                "matrices encode the wrong block duration (got {} matrices)",
                matrices.len()
            );
        };

        Self {
            matrices
        }
    }

    /// gets a single forecast based on the following initial conditions
    fn get_single_forecast(&self, initial_conditions: &Vec<ActivityCategory>) -> Vec<ActivityCategory> {
        let mut activity = *initial_conditions.last().unwrap();
        let mut forecast = Vec::new();
        for matrix_index in initial_conditions.len() - 1..self.matrices.len() {
            let matrix = &self.matrices[matrix_index];
            activity = ActivityCategory::from_code(
                matrix.get_random_transition(activity.into_code())
            ).unwrap();
            forecast.push(activity);
        }
        forecast
    }
}

#[derive(Debug)]
struct ForecastDataKey(Vec<ActivityCategory>);

impl PartialEq for ForecastDataKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ForecastDataKey {}

impl PartialOrd for ForecastDataKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for ForecastDataKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for (act, other_act) in self.0.iter().zip(other.0.iter()) {
            let cmp_result = act.into_code().cmp(&other_act.into_code());
            if cmp_result != Ordering::Equal {
                return cmp_result;
            }
        }
        Ordering::Equal
    }
}

impl<const BLOCK_DURATION: u32> DayForecaster<BLOCK_DURATION> for  MarkovChain<BLOCK_DURATION> {
    fn forecast(
        &self,
        initial_conditions: std::rc::Rc<Vec<ActivityCategory>>,
        forecast_count: usize,
        simulation_size: u32,
    ) -> Vec<std::rc::Rc<crate::day_forecaster::Forecast<BLOCK_DURATION>>> {
        let simulation_count = simulation_size;
        let mut simulation_results: BTreeMap<ForecastDataKey, u32> = BTreeMap::new();

        for _ in 0..simulation_count {
            let forecast = ForecastDataKey(self.get_single_forecast(&initial_conditions));
            if simulation_results.contains_key(&forecast) {
                *simulation_results.get_mut(&forecast).unwrap() += 1;
            } else {
                simulation_results.insert(forecast, 1);
            }
        }

        let mut sim_results_array: Vec<(Vec<ActivityCategory>, u32)> = simulation_results.iter()
            .map(|result| (result.0.0.clone(), result.1.clone()))
        	.collect();
        sim_results_array.sort_by_key(|f| f.1);

        let mut forecasts = Vec::new();
        for i in 0..forecast_count {
            let result = sim_results_array.pop().unwrap();
            forecasts.push(Rc::new(Forecast::new(
                initial_conditions.clone(),
                result.0,
                format!("MCM Forecast {} ({} occurrences)", i + 1, result.1)
            )));
        }
        forecasts
    }
}

