use std::error::Error;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::prelude::*;
use std::env;

use serde::Deserialize;
use chrono::prelude::*;
use chrono::Duration;
use colored::*;

type MainResult = Result<(), Box<dyn Error>>;

#[derive(Debug, Deserialize)]
struct Vaktija {
    id: u32,
    lokacija: String,
    datum: [String; 2],
    vakat: [String; 6]
}

impl Vaktija {
    fn show_vaktija(&self) {
        println!(
            "Lokacija:   {}\nDatum:      {}\n---------------------------------------\n",
            self.lokacija, self.datum[1]);

        let (next, until_next) = self.next_index_and_until_next();

        let hours = until_next.num_hours();
        let minutes = until_next.num_minutes() % 60;
        let seconds = until_next.num_seconds() % 60;

        let next_time = format!("<- za {}h {}m {}s", hours, minutes, seconds);

        let current = if next == 0 { 5 } else { next - 1 };

        for (i, time) in self.vakat.iter().enumerate() {
            let namaz_output = format!("{}\t{}", namaz_name(i), time);

            if i == current {
                println!("{}", namaz_output.yellow());
            } else if i == next { 
                println!("{}\t{}", namaz_output, next_time.blue());
            } else {
                println!("{}", namaz_output);
            }
        }
    }

    fn next_index_and_until_next(&self) -> (usize, Duration) {
        let now = Local::now();
        let now = NaiveTime::from_hms(now.hour(), now.minute(), now.second());
        let idx = self.vakat.iter()
            .map(|x| NaiveTime::parse_from_str(x, "%H:%M").unwrap())
            .position(|x| x > now)
            .unwrap_or(0);

        let until_next = NaiveTime::parse_from_str(&self.vakat[idx], "%H:%M").unwrap() - now;
        
        (idx, until_next)
    }
}

fn namaz_name(namaz: usize) -> String {
    match namaz {
        0 => "Zora         ",
        1 => "Izlazak Sunca",
        2 => "Podne        ",
        3 => "Ikindija     ",
        4 => "Aksam        ",
        5 => "Jacija       ",
        _ => "Greska       ",
    }.to_owned()
}

fn get_network_data() -> Result<String, Box<dyn Error>> {
    let body = reqwest::get("https://api.vaktija.ba/vaktija/v1/110")?.text()?;
    Ok(body)
}

fn get_data(cache_file: &Path) -> Result<String, Box<dyn Error>> {
    match OpenOptions::new().read(true).write(true).open(cache_file) {
        Err(_) => {
            let mut f = match File::create(cache_file) {
                Ok(file) => file,
                Err(why) => panic!("file error {}", why),
            };
            let n_data = get_network_data()?;
            f.write_all(n_data.clone().as_bytes())?;
            return Ok(n_data);
        }
        Ok(mut f) => {
            let metadata = f.metadata()?;
            let last_modified = metadata.modified()?;
            let last_modified: DateTime<Utc> = DateTime::from(last_modified);
            let last_modified_date = NaiveDate::from_ymd(last_modified.year(), last_modified.month(), last_modified.day() + 1);
            let now = Local::now();
            let today = NaiveDate::from_ymd(now.year(), now.month(), now.day());

            if last_modified_date != today {
                let n_data = get_network_data()?;
                f.write_all(n_data.clone().as_bytes())?;
                return Ok(n_data);
            }

            let mut data = String::new();
            f.read_to_string(&mut data)?;

            Ok(data)
        }
    }
}


fn main() -> MainResult {
    let path_str = env::var("HOME")?;
    let path = Path::new(&path_str).join(".vaktija_cache");
    let data = get_data(path.as_path())?;
    let v: Vaktija = serde_json::from_str(&data)?;
    v.show_vaktija();

    Ok(())
}
