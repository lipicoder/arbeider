//! Security returns.

use std::fs::File;
use std::io;
use arrow_array::Float64Array;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

pub struct Returns {
    /// Price array
    prices: Float64Array
}

impl Returns {
    /// Create a new Returns struct.
    pub fn new(prices: Float64Array) -> Self {
        Self { prices }
    }

    /// Get prices array from parquet file.
    pub fn from_parquet(path: &str) -> Self {
        let prices = read_price(path).unwrap();
        Self { prices }
    }

    /// Calculate returns.
    pub fn day_returns(&self) -> Float64Array {
        // New a builder
        let mut builder = Float64Array::builder(0);

        // Calculate returns from index 1
        for i in 1..self.prices.len() {
            let pre_price = self.prices.value(i - 1);
            let price = self.prices.value(i);
            let ret = (price - pre_price) / pre_price;
            builder.append_value(ret);
        }
        builder.finish()
    }

    /// Cumulative returns.
    pub fn cumulative_returns(&self) -> f64{
        // Calculate cumulative returns from index 1
        let mut cum_ret = 0.0;
        for i in 1..self.prices.len() {
            let pre_price = self.prices.value(i - 1);
            let price = self.prices.value(i);
            let ret = (price - pre_price) / pre_price;
            cum_ret += ret;
        }
        cum_ret
    }

    /// Max drawdown.
    pub fn max_drawdown(&self) -> f64 {
        // Calculate max drawdown from index 1
        let mut max_drawdown = 0.0;
        let mut max_price = self.prices.value(0);
        for i in 1..self.prices.len() {
            let price = self.prices.value(i);
            if price > max_price {
                max_price = price;
            }
            let drawdown = (max_price - price) / max_price;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        max_drawdown
    }

    /// Square returns.
    pub fn square_returns(&self) -> Float64Array {
        // New a builder
        let mut builder = Float64Array::builder(0);

        // Calculate square returns from index 1
        for i in 1..self.prices.len() {
            let pre_price = self.prices.value(i - 1);
            let price = self.prices.value(i);
            let ret = (price - pre_price) / pre_price;
            builder.append_value(ret * ret);
        }
        builder.finish()
    }

    /// Volatility.
    pub fn volatility(&self) -> f64 {
        // Calculate day return.
        let day_returns = self.day_returns();

        // Calculate square returns.
        let square_returns = self.square_returns();

        // Calculate volatility.
        let sum_day = day_returns.iter().sum::<f64>();
        let sum_square = square_returns.iter().sum::<f64>();

        // Calculate volatility.
        let mean = sum_day / (self.prices.len() - 1) as f64;
        (sum_square / (self.prices.len() - 1) as f64 - mean * mean).sqrt()
    }

    /// Sharpe ratio.
    pub fn sharpe_ratio(&self) -> f64 {
        // Calculate sharpe ratio from index 1
        let mut cum_ret = 0.0;
        let mut cum_ret2 = 0.0;
        for i in 1..self.prices.len() {
            let pre_price = self.prices.value(i - 1);
            let price = self.prices.value(i);
            let ret = (price - pre_price) / pre_price;
            cum_ret += ret;
            cum_ret2 += ret * ret;
        }
        // Calculate day return sum.
        let sum_day = self.day_returns().iter().sum::<f64>();

        // Calculate returns square sum.
        let sum_square = self.square_returns().iter().sum::<f64>();

        // Calculate mean.
        let mean = sum_day / (self.prices.len() - 1) as f64;

        // Calculate std.
        let std = (sum_square / (self.prices.len() - 1) as f64 - mean * mean).sqrt();

        // Calculate sharpe ratio.
        mean / std
    }

    /// Sortino ratio.
    pub fn sortino_ratio(&self) -> f64 {
        // Calculate sortino ratio from index 1
        let mut cum_ret = 0.0;
        let mut cum_ret2 = 0.0;
        for i in 1..self.prices.len() {
            let pre_price = self.prices.value(i - 1);
            let price = self.prices.value(i);
            let ret = (price - pre_price) / pre_price;
            cum_ret += ret;
            if ret < 0.0 {
                cum_ret2 += ret * ret;
            }
        }
        let mean = cum_ret / (self.prices.len() - 1) as f64;
        let std = (cum_ret2 / (self.prices.len() - 1) as f64 - mean * mean).sqrt();
        mean / std
    }
}

/// Read prices array from parquet.
fn read_price(path: &str) -> Result<Float64Array, io::Error> {
    let file = File::open(path).unwrap();

    // New a reader builder
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();

    // New a reader
    let mut reader = builder.build().unwrap();

    // New a builder
    let mut builder = Float64Array::builder(0);

    // Read a batch and push to price vector
    while let Some(batch) = reader.next() {

        let batch_data = batch.unwrap();

        // column 1 is close price: code-price-date
        let close = batch_data.column(1).as_any().downcast_ref::<Float64Array>()
                              .unwrap();

        for i in 0..close.len() {
            builder.append_value(close.value(i));
        }
    }

    Ok(builder.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_day_returns() {
        // read a stock price from parquet file
        let path = "examples/data/600878.parquet";


        let mut start = Instant::now();
        let returns = Returns::from_parquet(path);
        let mut duration = start.elapsed();
        println!("read parquet time: {:?}", duration);

        // calculate returns
        start = Instant::now();
        let result = returns.day_returns();
        duration = start.elapsed();
        println!("returns length:{:?}", result.len());
        println!("cal_returns time: {:?}", duration);

        assert!(result.len() > 0);
    }

    #[test]
    fn test_read_time(){
        // read a stock price from parquet file
        let mut path = "examples/data/600.parquet";

        let mut start = Instant::now();
        let mut returns = Returns::from_parquet(path);
        let mut duration = start.elapsed();
        println!("read {} parquet time: {:?}", path, duration);
        println!("returns length:{:?}", returns.prices.len());

        path = "examples/data/600601.parquet";
        start = Instant::now();
        returns = Returns::from_parquet(path);
        duration = start.elapsed();
        println!("read {} parquet time: {:?}", path, duration);

        assert!(returns.prices.len() > 0);
        // result:
        // read examples/data/600.parquet parquet time: 466.52525ms
        // returns length:1855143
        // read examples/data/600601.parquet parquet time: 1.749459ms
    }
}
