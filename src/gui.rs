use crate::distributions::{DistributionInputs, Distributions};
use crate::mcs::start_simulation;
use crate::message::SimulationMessage;
use eframe::egui;
use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver},
    thread,
};
use strum::IntoEnumIterator; 

#[derive(Default)]
pub struct MyEguiApp {
    commission: f64,
    number_of_months: f64,
    number_of_trials: f64,
    transport_bonus: f64,
    error_message: String,
    show_error_popup: bool,

    price_distr: Distributions,
    price_inputs: DistributionInputs,

    retailers_day_distr: Distributions,
    retailers_day_inputs: DistributionInputs,

    units_sale_distr: Distributions,
    units_sale_inputs: DistributionInputs,

    workdays_month_distr: Distributions,
    workdays_month_inputs: DistributionInputs,

    conversion_rate_distr: Distributions,
    conversion_rate_inputs: DistributionInputs,

    data: HashMap<String, (Distributions, DistributionInputs)>,
    probability_distributions: Vec<Distributions>,
    is_simulating: bool,
    progress: f32,
    simulation_receiver: Option<Receiver<SimulationMessage>>,
    simulation_result: Option<SimulationMessage>,
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.animation_time = 0.0;
        cc.egui_ctx.set_style(style);
        Self {
            number_of_months: 12.0,
            number_of_trials: 1000.0,
            probability_distributions: Distributions::iter()
                .filter(|&dist| dist != Distributions::Bernoulli)
                .collect(),
            ..Default::default()
        }
    }

    /// Gathers and validates all user inputs.
    /// This is now called only when the "Start Simulation" button is clicked.
    fn gather_and_validate_values(&mut self) -> Result<(), String> {
        // --- Validation Logic ---
        // This helper function reduces code duplication for validation.
        let validate =
            |dist: &Distributions, inputs: &DistributionInputs, name: &str| -> Result<(), String> {
                match dist {
                    Distributions::Uniform => {
                        if inputs.uniform_min > inputs.uniform_max {
                            return Err(format!("For {}, ensure min <= max.", name));
                        }
                    }
                    Distributions::Triangular => {
                        if !(inputs.triangular_min <= inputs.triangular_mode
                            && inputs.triangular_mode <= inputs.triangular_max)
                        {
                            return Err(format!("For {}, ensure min <= mode <= max.", name));
                        }
                    }
                    Distributions::Pert => {
                        if !(inputs.pert_min <= inputs.pert_mode
                            && inputs.pert_mode <= inputs.pert_max)
                        {
                            return Err(format!("For {}, ensure min <= mode <= max.", name));
                        }
                    }
                    _ => {} // Other distributions have no logical constraints here.
                }
                Ok(())
            };

        // Validate all inputs that have logical constraints.
        validate(&self.price_distr, &self.price_inputs, "Price")?;
        validate(
            &self.retailers_day_distr,
            &self.retailers_day_inputs,
            "Retailers/Day",
        )?;
        validate(
            &self.workdays_month_distr,
            &self.workdays_month_inputs,
            "Workdays/Month",
        )?;
        validate(
            &self.units_sale_distr,
            &self.units_sale_inputs,
            "Units/Sale",
        )?;
        validate(
            &self.conversion_rate_distr,
            &self.conversion_rate_inputs,
            "Conversion Rate",
        )?;

        // --- Data Gathering ---
        let mut hm: HashMap<String, (Distributions, DistributionInputs)> = HashMap::new();
        hm.insert(
            "Prices".into(),
            (self.price_distr, self.price_inputs.clone()),
        );
        hm.insert(
            "Retailers_per_Day".into(),
            (self.retailers_day_distr, self.retailers_day_inputs.clone()),
        );
        hm.insert(
            "Workdays_per_Month".into(),
            (
                self.workdays_month_distr,
                self.workdays_month_inputs.clone(),
            ),
        );
        hm.insert(
            "Conversion_Rate".into(),
            (
                self.conversion_rate_distr,
                self.conversion_rate_inputs.clone(),
            ),
        );
        hm.insert(
            "Units".into(),
            (self.units_sale_distr, self.units_sale_inputs.clone()),
        );

        let commission_inputs = DistributionInputs {
            constant_val: self.commission,
            ..Default::default()
        };
        let transport_bonus_inputs = DistributionInputs {
            constant_val: self.transport_bonus,
            ..Default::default()
        };
        let trials_inputs = DistributionInputs {
            constant_val: self.number_of_trials,
            ..Default::default()
        };
        let months_inputs = DistributionInputs {
            constant_val: self.number_of_months,
            ..Default::default()
        };

        hm.insert(
            "Commission_Rate".into(),
            (Distributions::Constant, commission_inputs),
        );
        hm.insert(
            "Transport_Bonus".into(),
            (Distributions::Constant, transport_bonus_inputs),
        );
        hm.insert("Trials".into(), (Distributions::Constant, trials_inputs));
        hm.insert(
            "Number_of_Months".into(),
            (Distributions::Constant, months_inputs),
        );

        self.data = hm;
        Ok(())
    }

    /// Renders the UI for selecting a distribution and its parameters.
    fn input_distributions(
        ui: &mut egui::Ui,
        distribution: Distributions,
        inputs: &mut DistributionInputs,
    ) {
        match distribution {
            Distributions::Bernoulli => {
                ui.label("Probability");
                ui.add(
                    egui::DragValue::new(&mut inputs.bernoulli_prob)
                        .speed(0.01)
                        .range(0.0..=1.0),
                );
            }
            Distributions::Normal => {
                ui.label("Mean");
                ui.add(egui::DragValue::new(&mut inputs.normal_mean).speed(0.1));
                ui.label("Standard Deviation");
                ui.add(
                    egui::DragValue::new(&mut inputs.normal_std)
                        .speed(0.1)
                        .range(0.0..=f64::INFINITY),
                );
            }
            Distributions::Uniform => {
                ui.label("Min");
                ui.add(egui::DragValue::new(&mut inputs.uniform_min).speed(0.1));
                ui.label("Max");
                ui.add(egui::DragValue::new(&mut inputs.uniform_max).speed(0.1));
            }
            Distributions::Constant => {
                ui.label("Value");
                ui.add(egui::DragValue::new(&mut inputs.constant_val).speed(0.1));
            }
            Distributions::Triangular => {
                ui.label("Min");
                ui.add(egui::DragValue::new(&mut inputs.triangular_min).speed(0.1));
                ui.label("Mode");
                ui.add(egui::DragValue::new(&mut inputs.triangular_mode).speed(0.1));
                ui.label("Max");
                ui.add(egui::DragValue::new(&mut inputs.triangular_max).speed(0.1));
            }
            Distributions::Pert => {
                ui.label("Min");
                ui.add(egui::DragValue::new(&mut inputs.pert_min).speed(0.1));
                ui.label("Mode");
                ui.add(egui::DragValue::new(&mut inputs.pert_mode).speed(0.1));
                ui.label("Max");
                ui.add(egui::DragValue::new(&mut inputs.pert_max).speed(0.1));
            }
        }
    }

    /// A helper function to create a row for a distribution selector.
    fn show_distribution_controls(
        ui: &mut egui::Ui,
        label_text: &str,
        distribution: &mut Distributions,
        inputs: &mut DistributionInputs,
        options: &Vec<Distributions>,
    ) {
        ui.horizontal(|ui| {
            ui.label(label_text);
            egui::ComboBox::from_label(format!("Select a distribution for {}", label_text))
                .selected_text(distribution.to_string())
                .show_ui(ui, |ui| {
                    for option in options {
                        ui.selectable_value(distribution, *option, option.to_string());
                    }
                });
            Self::input_distributions(ui, *distribution, inputs);
        });
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monte Carlo Simulation");
            ui.add_space(10.0);

            // --- DRAWING PHASE ---
            // The UI is drawn here. No validation or logic is performed in this phase.
            ui.add_enabled_ui(!self.is_simulating, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Commission rate");
                    ui.add(egui::DragValue::new(&mut self.commission).range(0.0..=100.0));
                    ui.label("%");
                });
                ui.horizontal(|ui| {
                    ui.label("Transport Bonus");
                    ui.add(egui::DragValue::new(&mut self.transport_bonus).range(0.0..=100.0));
                    ui.label("%");
                });

                Self::show_distribution_controls(
                    ui,
                    "Price to retailer",
                    &mut self.price_distr,
                    &mut self.price_inputs,
                    &self.probability_distributions,
                );
                Self::show_distribution_controls(
                    ui,
                    "Retailers/Day",
                    &mut self.retailers_day_distr,
                    &mut self.retailers_day_inputs,
                    &self.probability_distributions,
                );
                Self::show_distribution_controls(
                    ui,
                    "Workdays/Month",
                    &mut self.workdays_month_distr,
                    &mut self.workdays_month_inputs,
                    &self.probability_distributions,
                );
                Self::show_distribution_controls(
                    ui,
                    "Units/Sale",
                    &mut self.units_sale_distr,
                    &mut self.units_sale_inputs,
                    &self.probability_distributions,
                );
                Self::show_distribution_controls(
                    ui,
                    "Conversion Rate",
                    &mut self.conversion_rate_distr,
                    &mut self.conversion_rate_inputs,
                    &self.probability_distributions,
                );

                ui.horizontal(|ui| {
                    ui.label("Number of Months");
                    ui.add(
                        egui::DragValue::new(&mut self.number_of_months).range(1.0..=f64::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Number of Trials");
                    ui.add(
                        egui::DragValue::new(&mut self.number_of_trials).range(1.0..=f64::INFINITY),
                    );
                });

                ui.add_space(10.0);

                // --- EVENT HANDLING PHASE ---
                // Logic is only executed when the user clicks the button.
                let start_button = ui.button("Start Simulation");
                if start_button.clicked() && !self.is_simulating {
                    match self.gather_and_validate_values() {
                        Ok(()) => {
                            self.is_simulating = true;
                            self.simulation_result = None;
                            let (sender, receiver) = mpsc::channel::<SimulationMessage>();
                            self.simulation_receiver = Some(receiver);
                            let simulation_data = self.data.clone();
                            let progress_sender = sender.clone();

                            thread::spawn(move || {
                                let result =
                                    start_simulation(&simulation_data, Some(progress_sender));
                                match result {
                                    Ok(_) => {
                                        sender.send(SimulationMessage::Success("".into())).ok()
                                    }
                                    Err(e) => {
                                        sender.send(SimulationMessage::Error(e.to_string())).ok()
                                    }
                                };
                            });
                        }
                        Err(err) => {
                            self.error_message = err;
                            self.show_error_popup = true;
                        }
                    }
                }
            });

            if self.is_simulating {
                if let Some(receiver) = &self.simulation_receiver {
                    if let Ok(result) = receiver.try_recv() {
                        self.simulation_result = Some(result);
                        self.is_simulating = false; // The simulation is done
                        self.simulation_receiver = None; // Clean up the channel
                    }
                }
                ui.add_space(10.0);
                ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                ui.add_space(5.0);
                ui.spinner();
            }
        });

        // --- POPUP DISPLAY ---
        // This reads the state set in the event handling phase. It doesn't modify state itself.
        if self.show_error_popup {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            if ui.button("OK").clicked() {
                                self.show_error_popup = false;
                            }
                        });
                    });
                });
        }
    }
}
