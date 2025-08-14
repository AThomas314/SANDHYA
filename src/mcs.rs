use crate::distributions::{DistributionInputs, Distributions};
use crate::errors::DistributionError;
use crate::message::SimulationMessage;
use ndarray::*;
use ndarray_rand::{
    RandomExt,
    rand_distr::{Bernoulli, Normal, Pert, Triangular, Uniform},
};
use polars::prelude::*;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
    sync::mpsc::Sender,
};

pub fn start_simulation(
    data: &HashMap<String, (Distributions, DistributionInputs)>,
    progress_sender: Option<Sender<SimulationMessage>>,
) -> Result<(), PolarsError> {
    if !data.is_empty() {
        let sale_val = col("units") * col("price") * col("was_converted");
        let commission = sale_val.clone() * col("commission_rate");
        let transport_bonus = data.get("Transport_Bonus").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Transport_Bonus").into()))?.1.constant_val;
        let lf = create_data(data, &progress_sender)?
            .with_columns([
                sale_val.alias("Sale Value"),
                commission.alias("Commissions"),
                (col("was_converted").sum().over([col("distributor_id")])
                    / col("was_converted").len().over([col("distributor_id")]))
                .alias("Conversion Probability"),
            ])
            .group_by(["distributor_id", "month"])
            .agg([
                col("Commissions").sum(),
                col("Sale Value").sum(),
                col("units").sum(),
                col("Conversion Probability").unique().get(0),
            ]).with_column((col("Commissions")+(col("units")*lit(transport_bonus))).alias("Comission with bonus"));
        let save = save_dataframe(lf);
        match save {
            Ok(path) => {
                if let Some(sender) = progress_sender {
                    let _ = sender.send(SimulationMessage::Success(path));
                }
            }
            Err(e) => {
                if let Some(sender) = progress_sender {
                    let _ = sender.send(SimulationMessage::Error(e.to_string()));
                }
            }
        };
    }
    Ok(())
}

fn save_dataframe(lf: LazyFrame) -> Result<String, PolarsError> {
    let mut df = lf
        .collect()?
        .sort(
            ["distributor_id", "month"],
            SortMultipleOptions {
                descending: vec![false],
                nulls_last: vec![false],
                multithreaded: true,
                maintain_order: true,
                limit: None,
            },
        )?;
    
    let output_dir = "mcs_data";
    let output_filename = "output.parquet";
    let mut full_path = PathBuf::from(output_dir);
    full_path.push(output_filename);
    if let Some(parent_dir) = full_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    let file = File::create(&full_path)?;
    ParquetWriter::new(file).finish(&mut df)?;
    let absolute_path = fs::canonicalize(&full_path)?.to_string_lossy().to_string();
    Ok(absolute_path)
}

fn create_data(
    data: &HashMap<String, (Distributions, DistributionInputs)>,
    progress_sender: &Option<Sender<SimulationMessage>>,
) -> Result<LazyFrame, PolarsError> {
    let n = data.get("Trials").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Trials").into()))?.1.constant_val as i64 + 1;

    let commissions = data.get("Commission_Rate").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Commission_Rate").into()))?.1.constant_val / 100.0;
    
    let prices = data.get("Prices").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Prices").into()))?;
    
    let retailers_per_day = data.get("Retailers_per_Day").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Retailers_per_Day").into()))?;
    
    let workdays_per_month = data.get("Workdays_per_Month").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Workdays_per_Month").into()))?;
    
    let conversion_rate = data.get("Conversion_Rate").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Conversion_Rate").into()))?;
    
    let units = data.get("Units").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Units").into()))?;
    
    let num_months = data.get("Number_of_Months").ok_or_else(|| PolarsError::ComputeError(format!("'{}' parameter not found", "Number_of_Months").into()))?.1.constant_val.round() as usize;
    let range = 1..n;

    let lfs: Result<Vec<LazyFrame>, PolarsError> = range
        .into_par_iter()
        .map(|i| {
            create_lazyframes(
                workdays_per_month,
                conversion_rate,
                retailers_per_day,
                num_months,
                prices,
                units,
                i,
                n,
                commissions,
                progress_sender,
            )
        })
        .collect();
    let lf = concat(lfs?, UnionArgs::default())?;
    Ok(lf)
}

fn create_lazyframes(
    workdays_per_month: &(Distributions, DistributionInputs),
    conversion_rate: &(Distributions, DistributionInputs),
    retailers_per_day: &(Distributions, DistributionInputs),
    num_months: usize,
    prices: &(Distributions, DistributionInputs),
    units: &(Distributions, DistributionInputs),
    i: i64,
    n: i64,
    commissions: f64,
    progress_sender: &Option<Sender<SimulationMessage>>,
) -> Result<LazyFrame, PolarsError> {    
    let err_msg = |name: &str| PolarsError::ComputeError(format!("{} array was empty", name).into());

    let workdays_per_month = *create_array(workdays_per_month, 1)?.round().get(0).ok_or_else(|| err_msg("Workdays"))? as usize;
    let mut conversion_rate = *create_array(conversion_rate, 1)?.get(0).ok_or_else(|| err_msg("Workdays"))? / 100.0;
    conversion_rate = conversion_rate.clamp(0.01, 0.99);
    let conversion_rate = DistributionInputs {
        bernoulli_prob: conversion_rate,
        ..Default::default()
    };

    let retailers_per_day = *create_array(retailers_per_day, 1)?.round().get(0).ok_or_else(|| err_msg("Workdays"))?  as usize;
    let len = *&workdays_per_month * &retailers_per_day * &num_months;
    let dist_ids = Array1::<i64>::ones(len) * i;
    let commissions = Array1::<f64>::ones(len) * commissions;
    let months = create_months_array(&workdays_per_month, &retailers_per_day, &num_months);
    let conversions = create_array(&(Distributions::Bernoulli, conversion_rate), len);
    let prices = create_array(prices, len);
    let units = create_array(units, len)?.round();
    let lf = df! (
    "distributor_id" => dist_ids.to_vec(),
    "month" => months.to_vec(),
    "commission_rate" => commissions.to_vec(),
    "was_converted" => conversions?.to_vec(),
    "price" => prices?.to_vec(),
    "units"=>units.to_vec(),
    )?
    .lazy();
    if let Some(sender) = progress_sender {
        let _ = sender.send(SimulationMessage::Progress(i as f32 / n as f32));
    } else {
    }
    Ok(lf)
}

fn create_months_array(
    workdays_per_month: &usize,
    retailers_per_day: &usize,
    num_months: &usize,
) -> Array1<i64> {
    let block_size = workdays_per_month * retailers_per_day;
    let total_size = block_size * *num_months;
    Array1::from_shape_fn(total_size, |i| ((i / block_size) + 1) as i64)
}

fn create_array(
    params: &(Distributions, DistributionInputs),
    n: usize,
) -> Result<Array1<f64>, DistributionError> {
    let (distribution, input_strings) = params;
    let arr: Array1<f64> = match distribution {
        Distributions::Bernoulli => {
            Array1::random(n, Bernoulli::new(input_strings.bernoulli_prob)?)
                .mapv(|x| if x { 1.0 } else { 0.0 })
        }
        Distributions::Normal => Array1::random(
            n,
            Normal::new(input_strings.normal_mean, input_strings.normal_std)?,
        ),
        Distributions::Pert => Array1::random(
            n,
            Pert::new(
                input_strings.pert_min,
                input_strings.pert_max,
                input_strings.pert_mode,
            )?,
        ),
        Distributions::Triangular => Array1::random(
            n,
            Triangular::new(
                input_strings.triangular_min,
                input_strings.triangular_max,
                input_strings.triangular_mode,
            )?,
        ),
        Distributions::Uniform => Array1::random(
            n,
            Uniform::new(input_strings.uniform_min, input_strings.uniform_max),
        ),
        Distributions::Constant => Array1::<f64>::ones(n) * input_strings.constant_val,
    };
    Ok(arr)
}
