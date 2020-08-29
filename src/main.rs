use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use ics::{properties, Event, ICalendar};
use serde_json::Value;
use std::{env, io::Result, net::SocketAddr};

#[macro_use]
extern crate log;

fn check_autologin(new: &str) -> bool {
    // prepare regex
    let rule = "^([a-z0-9]{40})$";
    let re = match regex::Regex::new(rule) {
        Ok(re) => re,
        Err(_) => return false,
    };

    // regex check
    re.is_match(new)
}

fn get_registration(event: &Value) -> Option<bool> {
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

fn get_time(event: &Value, to_get: &str) -> Option<String> {
    let time =
        match chrono::NaiveDateTime::parse_from_str(event[to_get].as_str()?, "%Y-%m-%d %H:%M:%S") {
            Ok(time) => time,
            Err(_) => return None,
        };
    Some(time.format("%Y%m%dT%H%M%S").to_string())
}

fn construct_intra_url(event: &Value) -> Option<String> {
    let year = event["scolaryear"].as_str()?;
    let code_module = event["codemodule"].as_str()?;
    let code_instance = event["codeinstance"].as_str()?;
    let code_acti = event["codeacti"].as_str()?;

    Some(format!(
        "https://intra.epitech.eu/module/{}/{}/{}/{}",
        year, code_module, code_instance, code_acti
    ))
}

async fn weekly(req: HttpRequest) -> impl Responder {
    let autologin = match req.match_info().get("autologin") {
        Some(autologin) => autologin,
        None => return HttpResponse::BadRequest().body("no autologin provided"),
    };

    if !check_autologin(autologin) {
        return HttpResponse::BadRequest().body("invalid autologin provided");
    }

    let today = chrono::Local::today();
    let start_date = today - chrono::Duration::weeks(1);
    let end_date = today + chrono::Duration::weeks(1);

    let url = format!(
        "https://intra.epitech.eu/auth-{}/planning/load?format=json&start={}&end={}",
        autologin,
        start_date.format("%Y-%m-%d").to_string(),
        end_date.format("%Y-%m-%d").to_string()
    );

    let mut calendar = ICalendar::new("2.0", "-//epitech-ics//NONSGML Epitech Calendar//EN");

    let raw_json = match epitok::intra::get_array_obj(&url).await {
        Ok(raw_json) => raw_json,
        Err(e) => {
            return match e {
                epitok::intra::Error::Empty => HttpResponse::Ok().finish(),
                _ => HttpResponse::InternalServerError().body(e.to_string()),
            };
        }
    };

    for event in &raw_json {
        match get_registration(event) {
            Some(status) => {
                // if user is not registered, skip to next event
                if !status {
                    continue;
                }
            }
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get registration status of an event");
            }
        }

        let mut cal_event = Event::new(
            uuid::Uuid::new_v4().to_string(),
            chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string(),
        );

        // title
        let title = match event["acti_title"].as_str() {
            Some(title) => title.to_string(),
            None => {
                return HttpResponse::InternalServerError().body("could not get title of an event");
            }
        };
        cal_event.push(properties::Summary::new(title));

        // start
        let start = match get_time(&event, "start") {
            Some(start) => start,
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get start time of an event");
            }
        };
        cal_event.push(properties::DtStart::new(start));

        // end
        let end = match get_time(&event, "end") {
            Some(start) => start,
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get end time of an event");
            }
        };
        cal_event.push(properties::DtEnd::new(end));

        // location
        let location = match get_location(&event) {
            Some(location) => location,
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get location of an event");
            }
        };
        cal_event.push(properties::Location::new(location));

        // URL to intra
        let url = match construct_intra_url(&event) {
            Some(url) => url,
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not construct url of an event");
            }
        };
        cal_event.push(properties::URL::new(url));

        calendar.add_event(cal_event);
    }

    HttpResponse::Ok()
        .content_type("text/calendar; charset=utf-8")
        .body(calendar.to_string())
}

async fn root() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("index.html"))
}

#[actix_rt::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let port: u16 = match env::var("PORT") {
        Ok(port_str) => port_str.parse().expect("Could not use provided port."),
        Err(_) => 4343,
    };

    let app = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("[RETURNED HTTP %s] [TOOK %Dms]"))
            .route("/", web::get().to(root))
            .route("/{autologin}/weekly.ics", web::get().to(weekly))
    });

    info!("starting server on http://localhost:{}", port);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    app.bind(addr)?.run().await
}
