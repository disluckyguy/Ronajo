pub fn seconds_to_timestamp(time: f64) -> String {
    let mut hours = 0;
    let mut minutes = 0;
    let mut seconds = time as i32;

    if time >= 60f64 {
        let new_minutes = seconds as f64 / 60f64;
        minutes = new_minutes as i32;
        seconds = ((new_minutes - minutes as f64) * 60 as f64) as i32;

        if minutes >= 60 {
            let new_hours = minutes as f64 / 60f64;
            hours = new_hours as i32;
            minutes = ((new_hours - hours as f64) * 60 as f64) as i32;
        }
    }

    if hours != 0 {
        return format!("{}:{:0>2}:{:0>2}", hours, minutes, seconds);
    }

    return format!("{}:{:0>2}", minutes, seconds);
}
