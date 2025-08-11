use ndarray::prelude::*;
use rayon::prelude::*;
use crate::distributions::{DistributionInputStrings,Distributions};
use std::collections::HashMap;

pub fn run_simulation(data: & HashMap<String, (Distributions, DistributionInputStrings)>){
    if !data.is_empty(){
        parse_data(data)
    }
}

fn parse_data(data: &HashMap<String, (Distributions, DistributionInputStrings)>)->(){//HashMap<String,Array1<f64>>{
    let n: f64 = data.get(&"Trials".to_string()).unwrap().1.constant_val_str.parse::<f64>().unwrap();
    let commissionstr: f64 = data.get(&"Commission_Rate".to_string()).unwrap().1.constant_val_str.parse::<f64>().unwrap();

    println!("{:#?},{:#?}",n,commissionstr);
}