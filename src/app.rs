use egui::debug_text::print;

use crate::{day_forecaster::DayForecaster, encodings::ActivityCategory};

pub struct ActivityForecastApp<F: DayForecaster<BLOCK_DURATION>, const BLOCK_DURATION: u32> {
    forecaster: Box<F>,
    filled_activities: Vec<ActivityCategory>,
}

impl<F, const BLOCK_DURATION: u32> ActivityForecastApp<F, BLOCK_DURATION>
where F: DayForecaster<BLOCK_DURATION> {
    pub fn new(_cc: &eframe::CreationContext<'_>, forecaster: Box<F>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Self {
            forecaster: forecaster,
            filled_activities: Vec::new(),
        }
    }
}

impl<F, const BLOCK_DURATION: u32> eframe::App for ActivityForecastApp<F, BLOCK_DURATION>
where F: DayForecaster<BLOCK_DURATION> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // the activity to add at the end of the list
        let mut added_activity: Option<ActivityCategory> = None;

        // whether to delete the last added activity
        let mut pop_activity = false;

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
                        if ui.button(activity.into_str()).clicked() {
                            added_activity = Some(activity);
                        }
                    }
                }
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("delete").clicked() {
                    pop_activity = true;
                }
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("hi lol");
        });

        if let Some(activity) = added_activity {
            self.filled_activities.push(activity);
        }

        if pop_activity {
        	self.filled_activities.pop();
        }
    }
}
