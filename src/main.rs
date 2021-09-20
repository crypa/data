/*
 * Crypa data collection script.
 *
 * Schedule:
 * - Every day at 00:00, collect data from all top 500 cryptocurrencies.
 */

pub mod historic_data;
pub mod processing;

use std::path::Path;

use chrono::{Date, Duration, Utc};
use std::mem;

pub struct DateRange(Date<Utc>, Date<Utc>);

impl Iterator for DateRange {
    type Item = Date<Utc>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 >= self.1 {
            let next = self.0 - Duration::days(1);
            Some(mem::replace(&mut self.0, next))
        } else {
            None
        }
    }
}

fn main() {
    historic_data::download_data();
    processing::process_data();
}
