//! Generic application for data visualization. Works with any implementers of the DayForecaster
//! trait.

use std::rc::Rc;

use egui::{Color32, Layout, Sense, Ui, UiBuilder, Vec2, Widget, scroll_area::ScrollSource};

use crate::{day_forecaster::{DayForecaster, Forecast}, encodings::ActivityCategory};

type ActivityColorMap = [Color32; ActivityCategory::MAX_CODE + 1];

pub struct ActivityForecastApp<F: DayForecaster<BLOCK_DURATION>, const BLOCK_DURATION: u32> {
    /// the forecaster used to generate forecasts
    forecaster: Box<F>,

    /// the activity being "painted"
    paint_activity: Option<ActivityCategory>,

    /// the activities currently selected
    selected_activities: Vec<ActivityCategory>,

    /// the current forecast being examined
    /// if none, we are selecting activities
    forecast: Option<Rc<Forecast<BLOCK_DURATION>>>,

    /// a map of colors to use when drawing activities
    activity_color_map: ActivityColorMap,

    /// a list of forecasts that were generated
    generated_forecasts: Vec<Rc<Forecast<BLOCK_DURATION>>>,

    /// whether to display the warning that we cannot forecast yet
    forecast_error: Option<&'static str>,

    /// the selected size of the simulation
    simulation_size: u32,

    /// the number of forecasts to generate
    forecast_count: usize,

    // the inputted text for simulation size
    simulation_size_text: String,

    // the inputted text for forecasts count
    forecast_count_text: String,
}

impl<F, const BLOCK_DURATION: u32> ActivityForecastApp<F, BLOCK_DURATION>
where F: DayForecaster<BLOCK_DURATION> {

    pub fn new(cc: &eframe::CreationContext<'_>, forecaster: Box<F>) -> Self {
        assert!(60 % BLOCK_DURATION == 0, "block duration must divide an hour evenly");

        cc.egui_ctx.style_mut(|style| {
            style.interaction.selectable_labels = false;
        });

        let mut activity_color_map = [Color32::BLACK; ActivityCategory::MAX_CODE + 1];
        for i in 0..activity_color_map.len() {
            activity_color_map[i] = Self::color_from_hsv(
                360.0 / activity_color_map.len() as f32 * i as f32,
                0.5,
                0.4
            );
        }
        *activity_color_map.last_mut().unwrap() = cc.egui_ctx.style().visuals.window_fill;

        Self {
            forecaster: forecaster,
            paint_activity: None,
            selected_activities: vec![ActivityCategory::MissingData; 24 * 60 / BLOCK_DURATION as usize],
            forecast: None,
            activity_color_map,
            generated_forecasts: Vec::new(),
            forecast_error: None,
            forecast_count: 10,
            simulation_size: 1000,
            forecast_count_text: "10".to_string(),
            simulation_size_text: "1000".to_string(),
        }
    }

    fn color_from_hsv(h: f32, s: f32, v: f32) -> Color32 {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

        let (r1, g1, b1) = match h_prime as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x), // covers case 5
        };

        let m = v - c;

        let r = ((r1 + m) * 255.0).round() as u8;
        let g = ((g1 + m) * 255.0).round() as u8;
        let b = ((b1 + m) * 255.0).round() as u8;

        Color32::from_rgb(r, g, b)
    }

    fn paint_activity_button(&mut self, ui: &mut Ui, activity: ActivityCategory, text: &str) {
        if self.paint_activity == Some(activity) {
            let button = egui::Button::new(text)
                .fill(Color32::from_gray(140));
            if ui.add(button).clicked() {
                self.paint_activity = None;
            }
        } else {
            if ui.button(text).clicked() {
                self.paint_activity = Some(activity)
            }
        }
    }

    /// checks if we are ready for a forecast,
    /// returning the error message
    fn is_ready_for_forecast(&mut self) -> Option<&'static str> {
        let activity_contiguity = "Activities must be contiguous since the begining of the day.";
        let activity_emptiness = "There must be at least one empty activity.";
        let forecast_count_range = "There must between 1 and 100 forecasts.";
        let simulation_size_range = "Simulation size must be at least as large as the forecast count and smaller than 4 million.";
		let forecast_parse = "Invalid forecast count entered.";
		let simulation_parse = "Invalid simulation size entered.";

        // try parse forecast_count
        if let Ok(new_count) = self.forecast_count_text.parse::<usize>() {
            self.forecast_count = new_count;
        } else {
            return Some(forecast_parse);
        }

        // try parse simulation_size
        if let Ok(new_size) = self.simulation_size_text.parse::<u32>() {
            self.simulation_size = new_size;
        } else {
            return Some(simulation_parse)
        }

        // check that forecast count is valid
        if self.forecast_count < 1 || self.forecast_count > 100 {
            return Some(forecast_count_range)
        }

        // check that simulation size is valid
        if self.simulation_size > 4000000 || (self.simulation_size as usize) < self.forecast_count {
            return Some(simulation_size_range)
        }

        // check if an activity fills 00:00
        if self.selected_activities[0] == ActivityCategory::MissingData {
            return Some(activity_contiguity);
        }

        // check that activities have no gaps
        let mut found_end = false;
        for activity in &self.selected_activities {
            if *activity == ActivityCategory::MissingData {
                found_end = true;
            } else if found_end {
                return Some(activity_contiguity);
            }
        }

        // check that there is a blank activity
        if *self.selected_activities.last().unwrap() != ActivityCategory::MissingData {
            return Some(activity_emptiness);
        }

        None
    }

    fn block_count() -> usize {
        24 * 60 / BLOCK_DURATION as usize
    }

    fn forecast_central(&mut self, ui: &mut Ui) {
        egui::ScrollArea::new([false, true])
            .scroll_source(ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true })
            .show(ui, |ui| {
                ui.heading("Viewing Forecast");
                ui.separator();
                ui.with_layout(
                    egui::Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: true,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Min,
                        cross_justify: false,
                    }, |ui| {
                        let blocks_per_hour: usize = Self::block_count() / 24;
                        let forecast = self.forecast.clone().unwrap();
                        let initial_conditions = forecast.initial_conditions();
                        let mut activity_iter = initial_conditions.iter()
                            .chain(forecast.forecast_data().iter());
                        for h in 0..24 as usize {
                            let mut activities = Vec::new();
                            for _ in 0..blocks_per_hour {
                                activities.push(*activity_iter.next().unwrap());
                            }
                            ui.add(HourBlock::new(
                                h as u32,
                                blocks_per_hour,
                                activities.iter(),
                                &self.activity_color_map,
                            ));
                        }
                    }
                );
            });
    }

    fn activity_selector_central(&mut self, ui: &mut Ui) {
        egui::ScrollArea::new([false, true])
            .scroll_source(ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true })
            .show(ui, |ui| {
                ui.heading("Activity Selection");
                ui.separator();
                ui.with_layout(
                    egui::Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: true,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Min,
                        cross_justify: false,
                    }, |ui| {
                        let blocks_per_hour: usize = Self::block_count() / 24;
                        for h in 0..24 as usize {
                            let a = h * blocks_per_hour;
                            let b = (h + 1) * blocks_per_hour;
                            ui.add(MutHourBlock::new(
                                h as u32,
                                blocks_per_hour,
                                self.selected_activities[a..b].iter_mut(),
                                &self.activity_color_map,
                                self.paint_activity
                            ));
                        }
                    }
                );
            });
    }

    fn forecast_panel(&mut self, ui: &mut Ui) {
        egui::ScrollArea::new([false, true])
            .scroll_source(ScrollSource { scroll_bar: true, drag: false, mouse_wheel: true })
            .show(ui, |ui| {
                ui.heading("Forecasting");
                ui.separator();
                if self.generated_forecasts.is_empty() {
                    ui.label("no forecasts");
                } else {
                    for forecast in &self.generated_forecasts {
                        if self.forecast.is_some() && Rc::ptr_eq(forecast, self.forecast.as_ref().unwrap()) {
                            let button = egui::Button::new(forecast.name())
                                .fill(Color32::from_gray(140));
                            if ui.add(button).clicked() {
                                self.forecast = None;
                            }
                        } else {
                            if ui.button(forecast.name()).clicked() {
                                self.forecast = Some(forecast.clone());
                            }
                        }
                    }
                }
                ui.separator();

                ui.label("Forecast Count:");
                ui.text_edit_singleline(&mut self.forecast_count_text);
                ui.label("Simulation Size:");
                ui.text_edit_singleline(&mut self.simulation_size_text);

                if ui.button("make forecast").clicked() {
                    self.forecast_error = self.is_ready_for_forecast();
                    if self.forecast_error.is_none() {
                        let mut initial_conditions = Vec::new();
                        for activity in &self.selected_activities {
                            if *activity == ActivityCategory::MissingData {
                                break;
                            }
                            initial_conditions.push(*activity);
                        }
                        self.generated_forecasts = self.forecaster.forecast(
                            Rc::new(initial_conditions),
                            self.forecast_count,
                            self.simulation_size
                        );
                        self.forecast = Some(self.generated_forecasts[0].clone());
                    }
                }
                if let Some(error_msg) = self.forecast_error {
                    ui.label(error_msg);
                }
            });
    }

}

impl<F, const BLOCK_DURATION: u32> eframe::App for ActivityForecastApp<F, BLOCK_DURATION>
where F: DayForecaster<BLOCK_DURATION> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("forecast selector")
            .resizable(true)
            .show(ctx, |ui| {
                self.forecast_panel(ui);
            });

        if self.forecast.is_none() {
            egui::TopBottomPanel::bottom("activity_selector").show(ctx, |ui| {
                ui.label("add an activity:");
                ui.with_layout(
                    egui::Layout {
                        main_dir: egui::Direction::LeftToRight,
                        main_wrap: true,
                        main_align: egui::Align::Min,
                        main_justify: false,
                        cross_align: egui::Align::Min,
                        cross_justify: false,
                    }, |ui| {
                        for activity in ActivityCategory::valid_iter() {
                            self.paint_activity_button(ui, activity, activity.into_str());
                        }
                    }
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.paint_activity_button(ui, ActivityCategory::MissingData, "delete");
                })
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.forecast.is_none() {
                self.activity_selector_central(ui);
            } else {
                self.paint_activity = None;
                self.forecast_central(ui);
            }
        });
    }
}

#[derive(Debug)]
struct HourBlock<'a, T: Iterator<Item = &'a ActivityCategory>> {
    /// the hour in 24-hour time
    hour: u32,

    /// the maximum number of activities meant to be held
    max_activities: usize,

    /// the activities held
    activities: T,

    /// a map describing the color of each activity
    map: &'a ActivityColorMap,
}

impl<'a, T: Iterator<Item = &'a ActivityCategory>> HourBlock<'a, T> {
    const ADDITIONAL_SIZE: Vec2 = Vec2::new(8.0, 30.0);
    const CHILD_GAP: f32 = 4.0;
    const PADDING: f32 = 4.0;

    pub fn new(
        hour: u32,
        max_activities: usize,
        activities: T,
        map: &'a ActivityColorMap,
    ) -> Self {
        Self {
            hour,
            max_activities,
            activities,
            map,
        }
    }
}

impl<'a, T: Iterator<Item = &'a ActivityCategory>> Widget for HourBlock<'a, T> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = Self::ADDITIONAL_SIZE + Vec2::new(
            ActivityBlob::SIZE.x,
            self.max_activities as f32 * ActivityBlob::SIZE.y +
            (self.max_activities - 1) as f32 * Self::CHILD_GAP
        );
        let (_, full_rect) = ui.allocate_space(size);
        let rect = full_rect.shrink(Self::PADDING);
        let builder = UiBuilder::new()
            .sense(Sense::hover())
            .layout(Layout::top_down(egui::Align::Center))
            .max_rect(rect);
        let mut child_ui = ui.new_child(builder);

        let response = egui::Frame::new()
            .corner_radius(4.0)
            .stroke(ui.style().visuals.window_stroke)
            .inner_margin(Self::PADDING)
            .fill(ui.style().visuals.window_fill)
            .show(&mut child_ui, |ui| {
                ui.set_width(size.x - 2.0 * Self::PADDING);
                ui.set_height(size.y - 2.0 * Self::PADDING);
                ui.label(format!("{:0>2}:00", self.hour));
                for _ in 0..self.max_activities {
                    if let Some(activity) = self.activities.next() {
                        ui.add(ActivityBlob::new(*activity, self.map));
                    }
                }
            }).response;

        response
    }
}

#[derive(Debug)]
struct MutHourBlock<'a, T: Iterator<Item = &'a mut ActivityCategory>> {
    /// the hour in 24-hour time
    hour: u32,

    /// the maximum number of activities meant to be held
    max_activities: usize,

    /// the activities held
    activities: T,

    /// a map describing the color of each activity
    map: &'a ActivityColorMap,

    /// the activity to draw to the contained blocks
    drawing_activity: Option<ActivityCategory>,
}

impl<'a, T: Iterator<Item = &'a mut ActivityCategory>> MutHourBlock<'a, T> {
    const ADDITIONAL_SIZE: Vec2 = Vec2::new(8.0, 30.0);
    const CHILD_GAP: f32 = 4.0;
    const PADDING: f32 = 4.0;

    pub fn new(
        hour: u32,
        max_activities: usize,
        activities: T,
        map: &'a ActivityColorMap,
        drawing_activity: Option<ActivityCategory>
    ) -> Self {
        Self {
            hour,
            max_activities,
            activities,
            map,
            drawing_activity
        }
    }
}

impl<'a, T: Iterator<Item = &'a mut ActivityCategory>> Widget for MutHourBlock<'a, T> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = Self::ADDITIONAL_SIZE + Vec2::new(
            ActivityBlob::SIZE.x,
            self.max_activities as f32 * ActivityBlob::SIZE.y +
            (self.max_activities - 1) as f32 * Self::CHILD_GAP
        );
        let (_, full_rect) = ui.allocate_space(size);
        let rect = full_rect.shrink(Self::PADDING);
        let builder = UiBuilder::new()
            .sense(Sense::hover())
            .layout(Layout::top_down(egui::Align::Center))
            .max_rect(rect);
        let mut child_ui = ui.new_child(builder);

        let response = egui::Frame::new()
            .corner_radius(4.0)
            .stroke(ui.style().visuals.window_stroke)
            .inner_margin(Self::PADDING)
            .fill(ui.style().visuals.window_fill)
            .show(&mut child_ui, |ui| {
                ui.set_width(size.x - 2.0 * Self::PADDING);
                ui.set_height(size.y - 2.0 * Self::PADDING);
                ui.label(format!("{:0>2}:00", self.hour));
                for _ in 0..self.max_activities {
                    if let Some(activity) = self.activities.next() {
                        if ui.add(ActivityBlob::new(*activity, self.map)).contains_pointer()
                        && ui.ctx().input(|input| input.pointer.primary_down()) {
                            if let Some(drawn_activity) = self.drawing_activity {
                                *activity = drawn_activity;
                            }
                        }
                    }
                }
            }).response;

        response
    }
}

#[derive(Debug)]
struct ActivityBlob<'a> {
    activity: ActivityCategory,
    map: &'a ActivityColorMap,
}

impl<'a> ActivityBlob<'a> {
    const SIZE: Vec2 = Vec2::new(150.0, 24.0);
    const PADDING: f32 = 4.0;

    pub fn new(
        activity: ActivityCategory,
        map: &'a ActivityColorMap
    ) -> Self {
        Self {
            activity,
            map,
        }
    }
}

impl<'a> Widget for ActivityBlob<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (_, full_rect) = ui.allocate_space(Self::SIZE);
        let rect = full_rect.shrink(Self::PADDING);
        let builder = UiBuilder::new()
            .sense(Sense::all())
            .max_rect(rect);
        let mut child_ui = ui.new_child(builder);

        egui::Frame::new()
            .corner_radius(4.0)
            .stroke(ui.style().visuals.window_stroke)
            .inner_margin(Self::PADDING)
            .fill(self.map[self.activity.into_code() as usize])
            .show(&mut child_ui, |ui| {
                ui.set_width(Self::SIZE.x - 2.0 * Self::PADDING);
                ui.set_height(Self::SIZE.y - 2.0 * Self::PADDING);
                if self.activity != ActivityCategory::MissingData {
                    ui.label(self.activity.into_str());
                }
            });

        child_ui.response()
    }
}


