use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Clone, Copy, Default, EnumIter)]
pub enum Distributions {
    Uniform,
    #[default]
    Normal,
    Bernoulli,
    Constant,
    Triangular,
    Pert,
}

impl std::fmt::Display for Distributions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distributions::Uniform => write!(f, "Uniform"),
            Distributions::Normal => write!(f, "Normal"),
            Distributions::Bernoulli => write!(f, "Bernoulli"),
            Distributions::Constant => write!(f, "Constant"),
            Distributions::Triangular => write!(f, "Triangular"),
            Distributions::Pert => write!(f, "Pert"),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DistributionInputs {
    pub bernoulli_prob: f64,
    pub normal_mean: f64,
    pub normal_std: f64,
    pub uniform_min: f64,
    pub uniform_max: f64,
    pub constant_val: f64,
    pub triangular_max: f64,
    pub triangular_min: f64,
    pub triangular_mode: f64,
    pub pert_max: f64,
    pub pert_min: f64,
    pub pert_mode: f64,
}
