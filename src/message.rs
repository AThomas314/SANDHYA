pub enum SimulationMessage {
    Progress(f32),
    Error(String),
    Success(String),
}
