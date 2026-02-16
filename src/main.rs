use modeling_daily_activity::{app::ActivityForecastApp, day_forecaster::RandomForecaster, encodings, state_space::BlockStateChangeMatrix};

fn main() {
    run_app();
}

#[allow(dead_code)]
fn process_data() {
    encodings::remap_original("./data/timedata.csv", "./data/timedata_remap.csv");

    encodings::day_id_remap("./data/timedata_remap.csv", "./data/timedata_remap_dayid.csv");

    encodings::block_remap(15, "./data/timedata_remap_dayid.csv", "./data/15blocks");

    let _ = BlockStateChangeMatrix::from_block_encoding("./data/15blocks.ablk");
}

#[allow(dead_code)]
fn run_app() {
    let rng = rand::rng();
    let forecaster = RandomForecaster::<_, 15>::new(rng);

    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Daily Activity Model",
        native_options,
        Box::new(|cc| Ok(Box::new(
            ActivityForecastApp::new(cc, Box::new(forecaster))
        )))
    );
}
