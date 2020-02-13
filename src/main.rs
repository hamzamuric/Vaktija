use std::error::Error;
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
            .unwrap();

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


fn main() -> MainResult {
    let body = reqwest::get("https://api.vaktija.ba/vaktija/v1/110")?.text()?;
    let v: Vaktija = serde_json::from_str(&body)?;
    v.show_vaktija();

    Ok(())
}
