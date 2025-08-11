use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Clone, Copy, Default, EnumIter)]
pub enum Distributions {
    Uniform,
    #[default]
    Normal,
    Bernoulli,
    Constant,
}

impl std::fmt::Display for Distributions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distributions::Uniform => write!(f, "Uniform"),
            Distributions::Normal => write!(f, "Normal"),
            Distributions::Bernoulli => write!(f, "Bernoulli"),
            Distributions::Constant => write!(f, "Constant"),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct DistributionInputStrings {
    pub bernoulli_prob_str: String,
    pub normal_mean_str: String,
    pub normal_std_str: String,
    pub uniform_min_str: String,
    pub uniform_max_str: String,
    pub constant_val_str: String,
}
impl DistributionInputStrings {
    pub fn is_any_field_filled(&self) -> bool {
        !self.bernoulli_prob_str.is_empty()
            || !self.normal_mean_str.is_empty()
            || !self.normal_std_str.is_empty()
            || !self.uniform_min_str.is_empty()
            || !self.uniform_max_str.is_empty()
            || !self.constant_val_str.is_empty()
    }
}
