use std::path::Path;
use crate::DateRange;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::thread::sleep;

use chrono::{Datelike, Duration, TimeZone, Utc};
use curl::easy::Easy;
use curl::easy::List;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoricalData {
    pub id: u32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub num_market_pairs: Option<u32>,
    pub date_added: String,
    pub tags: Vec<String>,
    pub max_supply: Option<f64>,
    pub circulating_supply: f64,
    pub total_supply: f64,
    pub platform: Option<Platform>,
    pub cmc_rank: u32,
    pub last_updated: String,
    pub quote: HashMap<String, Quote>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Platform {
    pub id: u32,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub token_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quote {
    pub price: f64,
    pub volume_24h: f64,
    #[serde(default)]
    pub percent_change_1h: Option<f64>,
    #[serde(default)]
    pub percent_change_24h: Option<f64>,
    #[serde(default)]
    pub percent_change_7d: Option<f64>,
    pub market_cap: f64,
    pub last_updated: String,
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

pub fn download_data() -> Result<String, String> {
    let start_date = Utc.ymd(2013, 4, 28);
    let end_date = Utc::now().date() - Duration::days(1);

    for date in DateRange(end_date, start_date) {
        // Skip if file already exists
        if Path::new(&format!(
            "cmc_data/{}-{}-{}.json",
            date.year(),
            date.month(),
            date.day()
        ))
        .exists()
        {
            println!("Data for {:?} already exists, skipping", date);
            continue;
        }
        
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
                (i * 100) - 100
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

            let json = serde_json::from_str(&response);

            if json.is_err() {
                return Err(format!("{}", json.unwrap_err()));
            }

            let json: Request = json.unwrap();

            length = json.data.len();
            data.extend(json.data);
            sleep(std::time::Duration::from_millis(500));
        }

        // Write json data to file inside folder data
        let writer = BufWriter::new(
            File::create(format!(
                "cmc_data/{}-{}-{}.json",
                date.year(),
                date.month(),
                date.day()
            ))
            .unwrap(),
        );

        serde_json::to_writer(writer, &data).unwrap();

        println!(
            "Sucessfully downloaded {}-{}-{}, with {} entries",
            date.year(),
            date.month(),
            date.day(),
            data.len()
        );

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Ok("".to_string())
}
