use modeling_daily_activity::{app::ActivityForecastApp, encodings, markov_chain::{BlockStateChangeMatrix, MarkovChain}};

fn main() {
    //process_data();
    //run_app("./data/15blocks.ablk");
    run_app("./15blocks.ablk");
}

#[allow(dead_code)]
fn process_data() {
    encodings::remap_original("./data/timedata.csv", "./data/timedata_remap.csv");

    encodings::day_id_remap("./data/timedata_remap.csv", "./data/timedata_remap_dayid.csv");

    encodings::block_remap(15, "./data/timedata_remap_dayid.csv", "./data/15blocks", true);
    encodings::block_remap(15, "./data/timedata_remap_dayid.csv", "./data/15blocks", false);

    let _ = BlockStateChangeMatrix::from_block_encoding("./data/15blocks.ablk");
}

#[allow(dead_code)]
fn run_app(path: &str) {
    //let forecaster = RandomForecaster::<_, 15>::new(rand::rng());
    
    let forecaster = MarkovChain::<15>::from_block_encoding(path);

    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Daily Activity Model",
        native_options,
        Box::new(|cc| Ok(Box::new(
            ActivityForecastApp::new(cc, Box::new(forecaster))
        )))
    );
}
