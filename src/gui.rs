use crate::distributions::{DistributionInputStrings, Distributions};
use crate::mcs::run_simulation;
use eframe::egui;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct MyEguiApp {
    commissionstr: DistributionInputStrings,
    error_message: String,
    show_error_popup: bool,

    price_distr: Distributions,
    price_inputs: DistributionInputStrings,

    retailers_day_distr: Distributions,
    retailers_day_inputs: DistributionInputStrings,

    workdays_month_distr: Distributions,
    workdays_month_inputs: DistributionInputStrings,

    conversion_rate_distr: Distributions,
    conversion_rate_inputs: DistributionInputStrings,

    number_of_trials: DistributionInputStrings,
    data: HashMap<String, (Distributions, DistributionInputStrings)>,
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    fn gather_values(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let start_button = egui::Button::new("Start Simulation");
        if ui.add(start_button).clicked() {
            let commision = (Distributions::Constant, self.commissionstr.clone());
            let prices: (Distributions, DistributionInputStrings) =
                (self.price_distr, self.price_inputs.clone());
            let retailers_per_day: (Distributions, DistributionInputStrings) =
                (self.retailers_day_distr, self.retailers_day_inputs.clone());
            let workdays_per_month: (Distributions, DistributionInputStrings) = (
                self.workdays_month_distr,
                self.workdays_month_inputs.clone(),
            );
            let conversion_rate: (Distributions, DistributionInputStrings) = (
                self.conversion_rate_distr,
                self.conversion_rate_inputs.clone(),
            );
            let trials: (Distributions, DistributionInputStrings) =
                (Distributions::Constant, self.number_of_trials.clone());

            if !&commision.1.is_any_field_filled()
                || !&prices.1.is_any_field_filled()
                || !&retailers_per_day.1.is_any_field_filled()
                || !&workdays_per_month.1.is_any_field_filled()
                || !&conversion_rate.1.is_any_field_filled()
                || !&trials.1.is_any_field_filled()
            {
                return Some("Please fill all fields before running the simulation".to_string());
            } else {
                let mut hm: HashMap<String, (Distributions, DistributionInputStrings)> =
                    HashMap::new();
                hm.insert("Prices".into(), prices);
                hm.insert("Retailers_per_Day".into(), retailers_per_day);
                hm.insert("Workdays_per_Month".into(), workdays_per_month);
                hm.insert("Conversion_Rate".into(), conversion_rate);
                hm.insert("Commission_Rate".into(), commision);
                hm.insert("Trials".into(), trials);
                self.data = hm;
                return None;
            };
        }
        None
    }

    fn input_distributions(
        ui: &mut egui::Ui,
        distribution: Distributions,
        inputs: &mut DistributionInputStrings,
    ) -> Option<String> {
        match distribution {
            Distributions::Bernoulli => {
                ui.label("Probability");
                ui.add(egui::TextEdit::singleline(&mut inputs.bernoulli_prob_str));

                if !inputs.bernoulli_prob_str.is_empty() {
                    match inputs.bernoulli_prob_str.parse::<f64>() {
                        Ok(p_val) => {
                            if p_val < 0.0 || p_val > 1.0 {
                                return Some(
                                    "Probability must be between 0.0 and 1.0.".to_string(),
                                );
                            }
                        }
                        Err(_) => {
                            return Some(format!(
                                "Invalid number for probability: {}",
                                inputs.bernoulli_prob_str
                            ));
                        }
                    };
                }
            }
            Distributions::Normal => {
                ui.label("Mean");
                ui.add(egui::TextEdit::singleline(&mut inputs.normal_mean_str));
                ui.label("Standard Deviation");
                ui.add(egui::TextEdit::singleline(&mut inputs.normal_std_str));

                if !inputs.normal_mean_str.is_empty() {
                    if let Err(e) = inputs.normal_mean_str.parse::<f64>() {
                        return Some(format!("Invalid number for mean: {}", e));
                    }
                }
                if !inputs.normal_std_str.is_empty() {
                    if let Err(e) = inputs.normal_std_str.parse::<f64>() {
                        return Some(format!("Invalid number for standard deviation: {}", e));
                    }
                }
            }
            Distributions::Uniform => {
                ui.label("Min");
                ui.add(egui::TextEdit::singleline(&mut inputs.uniform_min_str));
                ui.label("Max");
                ui.add(egui::TextEdit::singleline(&mut inputs.uniform_max_str));

                if !inputs.uniform_min_str.is_empty() {
                    if let Err(e) = inputs.uniform_min_str.parse::<f64>() {
                        return Some(format!("Invalid number for minimum value: {}", e));
                    }
                }
                if !inputs.uniform_max_str.is_empty() {
                    if let Err(e) = inputs.uniform_max_str.parse::<f64>() {
                        return Some(format!("Invalid number for maximum value: {}", e));
                    }
                }
            }
            Distributions::Constant => {
                ui.label("Value");
                ui.add(egui::TextEdit::singleline(&mut inputs.constant_val_str));

                if !inputs.constant_val_str.is_empty() {
                    if let Err(e) = inputs.constant_val_str.parse::<f64>() {
                        return Some(format!("Invalid number for constant value: {}", e));
                    }
                }
            }
        }
        None
    }

    fn show_trials_input(&mut self, ui: &mut egui::Ui) -> Option<String> {
        ui.horizontal(|ui| {
            ui.label("Number of trials");
            ui.add(egui::TextEdit::singleline(
                &mut self.number_of_trials.constant_val_str,
            ));
        });

        if !self.number_of_trials.constant_val_str.is_empty() {
            if let Err(e) = self.number_of_trials.constant_val_str.parse::<f64>() {
                return Some(format!("Invalid number for constant value: {}", e));
            } else {
                None
            }
        } else {
            None
        }
    }
    fn show_commission_input(&mut self, ui: &mut egui::Ui) -> Option<String> {
        ui.horizontal(|ui| {
            ui.label("Commission rate");
            ui.add(egui::TextEdit::singleline(
                &mut self.commissionstr.constant_val_str,
            ));
            ui.label("%");
        });
        if !self.commissionstr.constant_val_str.is_empty() {
            if let Err(e) = self.commissionstr.constant_val_str.parse::<f64>() {
                return Some(format!("Invalid number for constant value: {}", e));
            } else {
                None
            }
        } else {
            None
        }
    }

    fn show_distribution_controls(
        ui: &mut egui::Ui,
        label_text: &str,
        distribution: &mut Distributions,
        inputs: &mut DistributionInputStrings,
        options: &Vec<Distributions>,
    ) -> Option<String> {
        let mut error = None;
        ui.horizontal(|ui| {
            ui.label(label_text);
            egui::ComboBox::from_label(format!("Select a distribution for {}", label_text))
                .selected_text(distribution.to_string())
                .show_ui(ui, |ui| {
                    for option in options {
                        ui.selectable_value(distribution, *option, option.to_string());
                    }
                });
            error = Self::input_distributions(ui, *distribution, inputs);
        });
        error
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let probability_distributions: Vec<Distributions> = Distributions::iter().collect();
        let mut errors: Vec<String> = Vec::new();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SANDHYA");

            if let Some(err) = self.show_commission_input(ui) {
                errors.push(err);
            }

            if let Some(err) = Self::show_distribution_controls(
                ui,
                "Price to retailer",
                &mut self.price_distr,
                &mut self.price_inputs,
                &probability_distributions,
            ) {
                errors.push(err);
            }
            if let Some(err) = Self::show_distribution_controls(
                ui,
                "Retailers/Day",
                &mut self.retailers_day_distr,
                &mut self.retailers_day_inputs,
                &probability_distributions,
            ) {
                errors.push(err);
            }
            if let Some(err) = Self::show_distribution_controls(
                ui,
                "Workdays/Month",
                &mut self.workdays_month_distr,
                &mut self.workdays_month_inputs,
                &probability_distributions,
            ) {
                errors.push(err);
            }
            if let Some(err) = Self::show_distribution_controls(
                ui,
                "Conversion Rate",
                &mut self.conversion_rate_distr,
                &mut self.conversion_rate_inputs,
                &probability_distributions,
            ) {
                errors.push(err);
            }
            if let Some(err) = self.show_trials_input(ui) {
                errors.push(err);
            }

            if let Some(err) = self.gather_values(ui) {
                errors.push(err);
            } else {
                run_simulation(&self.data)
            };
        });
        if !errors.is_empty() {
            self.error_message = errors.join("\n");
            self.show_error_popup = true;
        }

        if self.show_error_popup {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.show_error_popup = false;
                        }
                    });
                });
        }
    }
}
