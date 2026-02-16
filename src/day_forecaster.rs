use std::{cell::RefCell, rc::Rc};

use rand::{Rng, RngExt};

use crate::encodings::ActivityCategory;

/// a forecast with blocks of activity
pub struct Forecast<const BLOCK_DURATION: u32> {
    /// the initial conditions that lead to the given forecast
    initial_conditions: Rc<Vec<ActivityCategory>>,

    /// the forecast itself
    prediction: Vec<ActivityCategory>,

    /// the certainty that the forecast will come true
    certainty: f64
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
        certainty: f64
    ) -> Self {
        assert!(24 * 60 % BLOCK_DURATION == 0, "block_duration must divide evenly into a day");

        let block_count = Self::block_count();
        assert!(
            initial_conditions.len() + forecast_data.len() == block_count as usize,
            "for a forecast with block_duration {}, initial_conditions and forecast_data must contain {} blocks",
            BLOCK_DURATION, block_count
        );

        assert!(
            0.0 <= certainty && certainty <= 1.0,
            "certainty must be a number between 0.0 and 1.0"
        );

        Self {
            initial_conditions,
            prediction: forecast_data,
            certainty
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

    /// the certainty of the forecast
    pub fn certainty(&self) -> f64 {
        self.certainty
    }
}

/// forecasts the activities performed later in the day based on activities formed during the day
pub trait DayForecaster<const BLOCK_DURATION: u32>{
    /// generate a forecast for the day based on the activities already performed in the day
    /// forecasts should have the same block_duration and their certainties should sum to 1
    fn forecast(
        &self,
        initial_conditions: Rc<Vec<ActivityCategory>>,
        forecast_count: usize,
    ) -> Vec<Box<Forecast<BLOCK_DURATION>>>;
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
    ) -> Vec<Box<Forecast<BLOCK_DURATION>>> {
        // the additional number of blocks to generate
        let additional_block_count = Forecast::<BLOCK_DURATION>::block_count() - initial_conditions.len();

        // the rng extracted from the refcell
        let mut rng = self.rng_cell.borrow_mut();

        // the list of forecasts produced
        let mut forecasts = Vec::with_capacity(forecast_count);

        for _ in 0..forecast_count {
            let mut forecast_data = Vec::with_capacity(additional_block_count);

            for _ in 0..additional_block_count {
                forecast_data.push(ActivityCategory::from_code(
                    rng.random_range(0..ActivityCategory::MAX_CODE as u8)
                ).unwrap());
            }

            forecasts.push(Box::new(Forecast::new(
                initial_conditions.clone(),
                forecast_data,
                1.0 / forecast_count as f64,
            )))
        }

        forecasts
    }
}


