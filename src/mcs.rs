use crate::distributions::{DistributionInputs, Distributions};
use ndarray::*;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::{Bernoulli, Normal, Pert, Triangular, Uniform};
use polars::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;

pub fn start_simulation(data: &HashMap<String, (Distributions, DistributionInputs)>) {
    if !data.is_empty() {
        let lf = create_data(data)
            .with_columns([
                (col("units") * col("price") * col("was_converted")).alias("Sale Value"),
                (col("units") * col("price") * col("was_converted") * col("commission_rate"))
                    .alias("Commissions"),
                (col("was_converted").sum().over([col("distributor_id")])
                    / col("was_converted").len().over([col("distributor_id")]))
                .alias("Conversion Probability"),
            ])
            .group_by(["distributor_id", "month"])
            .agg([
                col("Commissions").sum(),
                col("Sale Value").sum(),
                col("Conversion Probability").unique().get(0),
            ]);
        save_dataframe(lf).unwrap();
    } else {
        println!("{:#?}", "Data is empty".to_string())
    }
}
fn save_dataframe(lf: LazyFrame)-> Result<(), PolarsError> {
    let mut df = lf
        .collect()
        .unwrap()
        .sort(
            ["distributor_id", "month"],
            SortMultipleOptions {
                descending: vec![false],
                nulls_last: vec![false],
                multithreaded: true,
                maintain_order: true,
                limit: None,
            },
        )
        .unwrap();
    let file = File::create("output.parquet").expect("Could not create file.");
    ParquetWriter::new(file)
        .finish(&mut df)?;
    Ok(())

}
fn create_data(data: &HashMap<String, (Distributions, DistributionInputs)>) -> LazyFrame {
    let n = data.get("Trials").unwrap().1.constant_val as i64 + 1;
    let commissions = data.get("Commission_Rate").unwrap().1.constant_val / 100.0;
    let prices = data.get("Prices").unwrap();
    let retailers_per_day = data.get("Retailers_per_Day").unwrap();
    let workdays_per_month = data.get("Workdays_per_Month").unwrap();
    let conversion_rate = data.get("Conversion_Rate").unwrap();
    let units = data.get("Units").unwrap();
    let num_months = data.get("Number_of_Months").unwrap().1.constant_val.round() as usize;
    let range = 1..n;

    let lfs: Vec<LazyFrame> = range
        .into_par_iter()
        .map(|i| {
            create_dataframes(
                workdays_per_month,
                conversion_rate,
                retailers_per_day,
                num_months,
                prices,
                units,
                i,
                commissions,
            )
            .lazy()
        })
        .collect();
    let lf = concat(lfs, UnionArgs::default()).unwrap();
    lf
}
fn create_dataframes(
    workdays_per_month: &(Distributions, DistributionInputs),
    conversion_rate: &(Distributions, DistributionInputs),
    retailers_per_day: &(Distributions, DistributionInputs),
    num_months: usize,
    prices: &(Distributions, DistributionInputs),
    units: &(Distributions, DistributionInputs),
    i: i64,
    commissions: f64,
) -> DataFrame {
    let workdays_per_month = *create_array(workdays_per_month, 1).round().get(0).unwrap() as usize;
    let mut conversion_rate = *create_array(conversion_rate, 1).get(0).unwrap() / 100.0;
    conversion_rate = conversion_rate.clamp(0.01, 0.99);
    let conversion_rate = DistributionInputs {
        bernoulli_prob: conversion_rate,
        ..Default::default()
    };

    let retailers_per_day = *create_array(retailers_per_day, 1).round().get(0).unwrap() as usize;
    let len = *&workdays_per_month * &retailers_per_day * &num_months;
    let dist_ids = Array1::<i64>::ones(len) * i;
    let commissions = Array1::<f64>::ones(len) * commissions;
    let months = create_months_array(&workdays_per_month, &retailers_per_day, &num_months);
    let conversions = create_array(&(Distributions::Bernoulli, conversion_rate), len);
    let prices = create_array(prices, len);
    let units = create_array(units, len).round();
    let df = df! (
    "distributor_id" => dist_ids.to_vec(),
    "month" => months.to_vec(),
    "commission_rate" => commissions.to_vec(),
    "was_converted" => conversions.to_vec(),
    "price" => prices.to_vec(),
    "units"=>units.to_vec(),
    )
    .unwrap();
    df
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

fn create_array(params: &(Distributions, DistributionInputs), n: usize) -> Array1<f64> {
    let (distribution, input_strings) = params;
    let arr: Array1<f64> = match distribution {
        Distributions::Bernoulli => {
            Array1::random(n, Bernoulli::new(input_strings.bernoulli_prob).unwrap())
                .mapv(|x| if x { 1.0 } else { 0.0 })
        }
        Distributions::Normal => Array1::random(
            n,
            Normal::new(input_strings.normal_mean, input_strings.normal_std).unwrap(),
        ),
        Distributions::Pert => Array1::random(
            n,
            Pert::new(
                input_strings.pert_min,
                input_strings.pert_max,
                input_strings.pert_mode,
            )
            .unwrap(),
        ),
        Distributions::Triangular => Array1::random(
            n,
            Triangular::new(
                input_strings.triangular_min,
                input_strings.triangular_max,
                input_strings.triangular_mode,
            )
            .unwrap(),
        ),
        Distributions::Uniform => Array1::random(
            n,
            Uniform::new(input_strings.uniform_min, input_strings.uniform_max),
        ),
        Distributions::Constant => Array1::<f64>::ones(n) * input_strings.constant_val,
    };
    arr
}
