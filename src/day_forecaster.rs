//!	Defines the DayForecaster trait and associated types in order to allow generic implementations
//!	for easier data visualization.
//!
//! Also defines a RandomForecaster for testing data visualizers.

use std::{cell::RefCell, rc::Rc};

use rand::{Rng, RngExt};

use crate::encodings::ActivityCategory;

/// a forecast with blocks of activity
#[derive(Debug, Clone)]
pub struct Forecast<const BLOCK_DURATION: u32> {
    /// the initial conditions that lead to the given forecast
    initial_conditions: Rc<Vec<ActivityCategory>>,

    /// the forecast itself
    prediction: Vec<ActivityCategory>,

    /// the name of the forecast
    name: String
}

impl<const BLOCK_DURATION: u32> Forecast<BLOCK_DURATION> {
    /// gets the total number of blocks that should be in the forecast
    pub fn block_count() -> usize {
        (24 * 60 / BLOCK_DURATION) as usize
    }

    /// creates a new forecast
    pub fn new(
        initial_conditions: Rc<Vec<ActivityCategory>>,
        forecast_data: Vec<ActivityCategory>,
        name: String
    ) -> Self {
        assert!(24 * 60 % BLOCK_DURATION == 0, "block_duration must divide evenly into a day");

        let block_count = Self::block_count();
        assert!(
            initial_conditions.len() + forecast_data.len() == block_count as usize,
            "for a forecast with block_duration {}, initial_conditions and forecast_data must contain {} blocks",
            BLOCK_DURATION, block_count
        );

        Self {
            initial_conditions,
            prediction: forecast_data,
            name
        }
    }

    /// the initial blocks that led to the creation of a forecast
    pub fn initial_conditions(&self) -> Rc<Vec<ActivityCategory>> {
        self.initial_conditions.clone()
    }

    /// the predicted rest of the day
    pub fn forecast_data(&self) -> &[ActivityCategory] {
        &self.prediction
    }

    /// the name of the forecast
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

/// forecasts the activities performed later in the day based on activities formed during the day
pub trait DayForecaster<const BLOCK_DURATION: u32>{
    /// generate a given number of forcasts based on the given conditions and simulation size
    fn forecast(
        &self,
        initial_conditions: Rc<Vec<ActivityCategory>>,
        forecast_count: usize,
        simulation_size: u32,
    ) -> Vec<Rc<Forecast<BLOCK_DURATION>>>;
}

/// forecasts days randomly, used for testing purposes
pub struct RandomForecaster<R: Rng, const BLOCK_DURATION: u32> {
    /// the rng used to forecast activities
    rng_cell: RefCell<R>,
}

impl<R: Rng, const BLOCK_DURATION: u32> RandomForecaster<R, BLOCK_DURATION> {
    /// creates a new RandomForecaster by consuming a rng
    pub fn new(rng: R) -> Self {
        Self {
            rng_cell: RefCell::new(rng)
        }
    }

    /// gets the rng from the forecaster, destroying it
    pub fn rng(self) -> R {
        self.rng_cell.into_inner()
    }
}

impl<R: Rng, const BLOCK_DURATION: u32> DayForecaster<BLOCK_DURATION> for RandomForecaster<R, BLOCK_DURATION> {
    fn forecast(
        &self,
        initial_conditions: Rc<Vec<ActivityCategory>>,
        forecast_count: usize,
        simulationSize: u32,
    ) -> Vec<Rc<Forecast<BLOCK_DURATION>>> {
        let _ = simulationSize;

        // the additional number of blocks to generate
        let additional_block_count = Forecast::<BLOCK_DURATION>::block_count() - initial_conditions.len();

        // the rng extracted from the refcell
        let mut rng = self.rng_cell.borrow_mut();

        // the list of forecasts produced
        let mut forecasts = Vec::with_capacity(forecast_count);

        for i in 0..forecast_count {
            let mut forecast_data = Vec::with_capacity(additional_block_count);

            for _ in 0..additional_block_count {
                forecast_data.push(ActivityCategory::from_code(
                    rng.random_range(0..ActivityCategory::MAX_CODE as u8)
                ).unwrap());
            }

            forecasts.push(Rc::new(Forecast::new(
                initial_conditions.clone(),
                forecast_data,
                format!("Random Forecast {}", i + 1)
            )))
        }


        forecasts
    }
}


