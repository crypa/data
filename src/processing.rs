use crate::historic_data::Quote;
use crate::historic_data::HistoricalData;
use serde::{Deserialize, Serialize};

use crate::Path;
use chrono::Datelike;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;

use crate::DateRange;

// Input Data Example:
// [{
//     "id": 1,
//     "name": "Bitcoin",
//     "symbol": "BTC",
//     "slug": "bitcoin",
//     "num_market_pairs": 9258,
//     "date_added": "2013-04-28T00:00:00.000Z",
//     "tags": [
//         "mineable",
//         "pow",
//         "sha-256",
//         "store-of-value",
//         "state-channels"
//     ],
//     "max_supply": 21000000.0,
//     "circulating_supply": 18528168.0,
//     "total_supply": 18528168.0,
//     "platform": null,
//     "cmc_rank": 1,
//     "last_updated": "2020-10-01T23:00:00.000Z",
//     "quote": {
//         "USD": {
//             "price": 10619.451907664665,
//             "volume_24h": 40023134099.56767,
//             "percent_change_1h": -0.019247131155,
//             "percent_change_24h": -1.530342613652,
//             "percent_change_7d": -1.306816758802,
//             "market_cap": 196758989013.13135,
//             "last_updated": "2020-10-01T23:00:00.000Z"
//         }
//     }
// }, ...]

// Output Data Example:
// {
//     "BTC": {
//         "rank": 1,

//         "name": "Bitcoin",
//         "symbol": "BTC",
//         "slug": "bitcoin",

//         "age": 100,
//         "max_supply": 21000000.0,
//         "circulating_supply": 18528168.0,
//         "total_supply": 18528168.0,

//         "volume": 40023134099.56767,
//         "market_cap": 196758989013.13135
//     },
//     ...
// }
// Note: Calculate the age from the date_added field

#[derive(Serialize, Deserialize, Debug)]
struct Symbol {
    rank: u32,

    name: String,
    symbol: String,
    slug: String,

    age: i64,
    max_supply: f64,
    circulating_supply: f64,
    total_supply: f64,

    volume: f64,
    market_cap: f64,
}

pub fn process_data() {
    let start_date = Utc.ymd(2013, 4, 28);
    let end_date = Utc::now().date() - Duration::days(1);

    for date in DateRange(end_date, start_date) {
        let input_path_str = format!(
            "cmc_data/{}-{}-{}.json",
            &date.year(),
            &date.month(),
            &date.day()
        );

        let input_path = Path::new(&input_path_str);

        if !input_path.exists() {
            continue;
        }

        println!("Processing: {}", &input_path_str);

        let output_path_str = format!(
            "data/{}-{}-{}.json",
            &date.year(),
            &date.month(),
            &date.day()
        );

        let output_path = Path::new(&output_path_str);

        if output_path.exists() {
            println!("Output is already created, skipping: {}", &input_path_str);
            continue;
        }

        let input_content = std::fs::read_to_string(input_path).unwrap();

        let input: Vec<HistoricalData> = serde_json::from_str(&input_content).unwrap();

        let mut output: Vec<Symbol> = Vec::new();

        for historical_data in input {
            let default = Quote {
                price: 0.0,
                volume_24h: 0.0,
                percent_change_1h: None,
                percent_change_24h: None,
                percent_change_7d: None,
                market_cap: 0.0,
                last_updated: "1970-01-01T00:00:00.000Z".to_string(),
            };

            let quote = historical_data.quote.get("USD").unwrap_or(&default);

            // Calculate the age from the date_added field
            let date_added_parts: Vec<&str> =
                historical_data.date_added.split("T").collect::<Vec<&str>>()[0]
                    .split("-")
                    .collect::<Vec<&str>>();

            let date_added = Utc.ymd(
                date_added_parts[0].parse::<i32>().unwrap(),
                date_added_parts[1].parse::<u32>().unwrap(),
                date_added_parts[2].parse::<u32>().unwrap(),
            );

            let age = Utc::now().date() - date_added;

            let data = Symbol {
                rank: historical_data.cmc_rank.clone(),

                name: historical_data.name.clone(),
                symbol: historical_data.symbol.clone(),
                slug: historical_data.slug.clone(),
                age: age.num_days(),
                max_supply: historical_data.max_supply.unwrap_or(0.0),
                circulating_supply: historical_data.circulating_supply,
                total_supply: historical_data.total_supply,

                volume: quote.volume_24h,
                market_cap: quote.market_cap,
            };

            output.push(data);
        }

        let output_content = serde_json::to_string(&output).unwrap();

        std::fs::write(output_path, output_content).unwrap();
    }
}
