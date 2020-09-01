use crate::utils::{
    check_autologin, construct_intra_url, get_location, get_registration, get_time,
};
use actix_web::{HttpRequest, HttpResponse, Responder};
use ics::{components::Property, properties, Event, ICalendar};

pub async fn weekly(req: HttpRequest) -> impl Responder {
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

    // Create calendar
    let mut calendar = ICalendar::new("2.0", "-//epitech-ics//NONSGML Epitech Calendar//EN");

    // Add timezone information
    // X-WR-TIMEZONE can be used to represent timezones, but is not in the RFC.
    // Google Calendar uses X-WR-TIMEZONE, since I use it, it's supported here.
    // Some information: https://blog.jonudell.net/2011/10/17/x-wr-timezone-considered-harmful/
    // TODO: Add the VTIMEZONE property to have timezones conforming to the RFC
    calendar.push(Property::new("X-WR-TIMEZONE", "Europe/Paris"));

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
        match event["acti_title"].as_str() {
            Some(title) => cal_event.push(properties::Summary::new(title)),
            None => {
                return HttpResponse::InternalServerError().body("could not get title of an event");
            }
        }

        // start
        match get_time(&event, "start") {
            Some(start) => cal_event.push(properties::DtStart::new(start)),
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get start time of an event");
            }
        }

        // end
        match get_time(&event, "end") {
            Some(end) => cal_event.push(properties::DtEnd::new(end)),
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get end time of an event");
            }
        }

        // location
        match get_location(&event) {
            Some(location) => cal_event.push(properties::Location::new(location)),
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not get location of an event");
            }
        };

        // URL to intra
        let url = match construct_intra_url(&event) {
            Some(url) => url,
            None => {
                return HttpResponse::InternalServerError()
                    .body("could not construct url of an event");
            }
        };
        // Some calendars don't show URLs, so showing the URL as a description
        cal_event.push(properties::URL::new(url.clone()));
        cal_event.push(properties::Description::new(url));

        calendar.add_event(cal_event);
    }

    HttpResponse::Ok()
        .content_type("text/calendar; charset=utf-8")
        .body(calendar.to_string())
}
