/*
 * Crypa data collection script.
 *
 * Schedule:
 * - Every day at 00:00, collect data from all top 500 cryptocurrencies.
 */

use serde::{Deserialize, Serialize};
use serde_json::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::thread::sleep;

use chrono::{Date, Datelike, Duration, TimeZone, Utc};
use curl::easy::Easy;
use curl::easy::List;
use std::collections::HashMap;
use std::io::Write;
use std::mem;

struct DateRange(Date<Utc>, Date<Utc>);

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

#[derive(Serialize, Deserialize, Debug)]
struct HistoricalData {
    id: u32,
    name: String,
    symbol: String,
    slug: String,
    num_market_pairs: Option<u32>,
    date_added: String,
    tags: Vec<String>,
    max_supply: Option<f64>,
    circulating_supply: f64,
    total_supply: f64,
    platform: Option<Platform>,
    cmc_rank: u32,
    last_updated: String,
    quote: HashMap<String, Quote>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Platform {
    id: u32,
    name: String,
    symbol: String,
    slug: String,
    token_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Quote {
    price: f64,
    volume_24h: f64,
    #[serde(default)]
    percent_change_1h: Option<f64>,
    #[serde(default)]
    percent_change_24h: Option<f64>,
    #[serde(default)]
    percent_change_7d: Option<f64>,
    market_cap: f64,
    last_updated: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    timestamp: String,
    error_code: u32,
    error_message: Option<String>,
    elapsed: u32,
    credit_count: u32,
    notice: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    status: Status,
    data: Vec<HistoricalData>,
}

fn download_data(date: Date<Utc>) -> Result<String> {
    let mut handle = Easy::new();

    let mut data: Vec<HistoricalData> = Vec::new();

    // While loop if last element in result datas is not length of 100
    let mut length = 100;
    let mut i = 0;
    while length >= 100 {
        i += 1;
        if i > 10 {
            break;
        }
        let url = format!(
                "https://web-api.coinmarketcap.com/v1/cryptocurrency/listings/historical?date={}-{}-{}&limit=100&start={}",
                date.year(),
                date.month(),
                date.day(),
                i
            );

        println!("{} : Pulling data from {}", date, url);
        handle.url(&url).unwrap();
        let mut list = List::new();
        list.append("User-Agent: PostmanRuntime/7.28.4").unwrap();
        handle.http_headers(list).unwrap();
        let mut response_buffer = Vec::new();
        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|data| {
                    response_buffer.extend_from_slice(data);
                    Ok(data.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }
        let response = String::from_utf8_lossy(response_buffer.as_slice());

        let json: Request = serde_json::from_str(&response).unwrap();

        length = json.data.len();
        data.extend(json.data);
        
        sleep(std::time::Duration::from_millis(500));
    }

    // Write json data to file inside folder data
    let writer = BufWriter::new(
        File::create(format!(
            "data/{}-{}-{}.json",
            date.year(),
            date.month(),
            date.day()
        ))
        .unwrap(),
    );
    serde_json::to_writer(writer, &data)?;

    Ok(format!(
        "Sucessfully downloaded {}-{}-{}, with {} entries",
        date.year(),
        date.month(),
        date.day(),
        data.len()
    ))
}

fn main() {
    let start_date = Utc.ymd(2013, 4, 28);
    // let end_date = Utc.ymd(2021, 1, 1);
    // let start_date = Utc::now().date() - Duration::days(2);
    let end_date = Utc::now().date() - Duration::days(1);

    println!(
        "Looping trough all dates from {:?} to {:?}",
        start_date, end_date
    );

    for date in DateRange(end_date, start_date) {
        println!("{:?}", date);

        // Skip if file already exists
        if Path::new(&format!(
            "data/{}-{}-{}.json",
            date.year(),
            date.month(),
            date.day()
        ))
        .exists()
        {
            println!("Data for {:?} already exists, skipping", date);
            continue;
        }
        println!(
            "{}",
            download_data(date).unwrap_or(format!("Error downloading data for {}", date))
        );

        // Wait for 1 second
        sleep(std::time::Duration::from_millis(1000));
    }
}
