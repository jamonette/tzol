use chrono::prelude::*;
use chrono_tz::Tz;
use console::Color;
use console::Style;

mod generated_city_data;

fn main() {

    let args: Vec<String> = std::env::args().collect();
    let cities = &args[1..args.len()];

    // This map is stored as <str, str> in order to save the runtime
    // cost of parsing each of the thousands of strings to a Tz.
    //
    // Instead, we can parse only the Tz's for the cities provided as input.

    let city_to_timezone_map = generated_city_data::get_city_to_timezone_map();

    let mut cities_with_timezones = Vec::<(&str, &str)>::new();
    for city in cities {
        match city_to_timezone_map.get(city.as_str()) {
            Some(&tz_str) => cities_with_timezones.push((city, tz_str)),
            None => {
                println!("Timezone not found for city: {}", city);
                std::process::exit(1);
            }
        }
    }

    let local_time = Local::now();
    let local_offset_vs_utc = local_time.offset().fix().local_minus_utc() / 3600;

    print_clock_line("local time", 0, local_time);

    for (city, timezone_str) in cities_with_timezones {
        // We verify that the string can be parsed to a Tz during
        // the build/codgen phase, so unwrap should never panic here.
        let tz = timezone_str.to_string().parse::<Tz>().unwrap();
        let date_time = Utc::now().with_timezone(&tz);

        let offset_vs_utc = date_time.offset().fix().local_minus_utc() / 3600;
        let offset_vs_local = offset_vs_utc - local_offset_vs_utc;

        print_clock_line(city, offset_vs_local, date_time);
    }
}

const WORKDAY_START: u32 = 9;
const WORKDAY_END: u32 = 17;

fn print_clock_line<T: TimeZone>(tz_str: &str, offset: i32, dt: DateTime<T>) {

    let mut hours: Vec<u32> = (0..24).collect();
    if offset >= 0 {
        hours.rotate_left(offset as usize);
    } else {
        hours.rotate_right(-offset as usize);
    }

    let style = Style::new().fg(Color::Blue);
    print!("{:20}", style.apply_to(tz_str));

    for hour in hours {

        let style =
            match ((hour >= WORKDAY_START && hour <= WORKDAY_END), (hour == dt.hour())) {
                (true, true) => Style::new().fg(Color::Red).bg(Color::Blue),
                (true, false) => Style::new().fg(Color::White).bg(Color::Blue),
                (false, true) =>  Style::new().fg(Color::Red),
                _ => Style::new()
            };

        let hour_str = with_leading_zero(hour);

        // drop highlighting on segment trailing end of workday
        if hour == WORKDAY_END {
            print!("{}  ", style.apply_to(hour_str));
        } else {
            print!(
                "{}{}",
                style.apply_to(hour_str),
                style.apply_to("  ")
            );
        };
    }

    print!("\n");
}

fn with_leading_zero(n: u32) -> String {
    if n <= 9 {
        format!("0{}", n)
    } else {
        n.to_string()
    }
}
