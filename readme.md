Stochastic And Numerical Distributor Hypothetical Income Yield Analyzer (SANDHIYA)

This is a monte carlo simulation built to model distributor profitability, based on assumed probability distributions of key parameters.
gui.rs contains the gui, and uses channels to communicate with mcs.rs, ensuring non-blocking gui.
mcs.rs generates the data by creating sample distributions and arrays using the ndarray crate in rust, and provides the data to the polars lazyframe, which saves the data to a parquet.
Final output is generated in PowerBI.
Other .rs files are essentially boilerplate.

TODO : sink parquet in batches instead of collecting everything and writing it at once, leaving so much data in RAM.
