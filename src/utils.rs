use serde_json::Value;

pub fn check_autologin(new: &str) -> bool {
    // prepare regex
    let rule = "^([a-z0-9]{40})$";
    let re = match regex::Regex::new(rule) {
        Ok(re) => re,
        Err(_) => return false,
    };

    // regex check
    re.is_match(new)
}

pub fn get_registration(event: &Value) -> Option<bool> {
    match event["event_registered"].as_str() {
        Some(event_registered) => {
            if event_registered == "registered" || event_registered == "present" {
                Some(true)
            } else {
                Some(false)
            }
        }
        None => match event["event_registered"].as_bool() {
            Some(event_unregistered) => {
                if !event_unregistered {
                    Some(false)
                } else {
                    None
                }
            }
            None => None,
        },
    }
}

pub fn get_location(event: &Value) -> Option<String> {
    // Raw room format: "Country/City/Location/Room-Name"
    let mut location = match event["room"]["code"].as_str() {
        Some(location) => location.to_string(),
        None => return Some("At the bar 🍺".into()),
    };

    // Start by finding if there is Country and City in the room string
    let re = match regex::Regex::new("^([a-zA-Z]+/[a-zA-Z]+/)") {
        Ok(re) => re,
        Err(_) => return None,
    };

    // Remove them if they are present
    location = re.replace(&location, "").to_string();

    // Replace the `/` by arrows for prettiness
    location = location.replace("/", " → ");

    // Replace the `-` by spaces for room name
    location = location.replace("-", " ");

    // We are done, return the freshly formatted room
    Some(location)
}

pub fn get_time(event: &Value, to_get: &str) -> Option<String> {
    let time =
        match chrono::NaiveDateTime::parse_from_str(event[to_get].as_str()?, "%Y-%m-%d %H:%M:%S") {
            Ok(time) => time,
            Err(_) => return None,
        };
    Some(time.format("%Y%m%dT%H%M%S").to_string())
}

pub fn construct_intra_url(event: &Value) -> Option<String> {
    let year = event["scolaryear"].as_str()?;
    let code_module = event["codemodule"].as_str()?;
    let code_instance = event["codeinstance"].as_str()?;
    let code_acti = event["codeacti"].as_str()?;

    Some(format!(
        "https://intra.epitech.eu/module/{}/{}/{}/{}",
        year, code_module, code_instance, code_acti
    ))
}
