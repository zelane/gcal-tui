//! Google Calendar API

use std::error::Error;

use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EventsResponse {
    #[serde(default)]
    items: Vec<ApiEvent>,
}

#[derive(Debug, Deserialize)]
struct ApiEvent {
    summary: Option<String>,
    location: Option<String>,
    start: ApiTime,
    end: Option<ApiTime>,
}

#[derive(Debug, Deserialize)]
struct ApiTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CalEvent {
    pub date: NaiveDate,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
    pub summary: String, 
    pub location: Option<String>,
}

impl CalEvent {
    fn from_api(ev: ApiEvent) -> Option<Self> {
        let summary = ev.summary.unwrap_or_else(|| "(no title)".to_string());

        if let Some(dt) = ev.start.date_time {
            let parsed = DateTime::parse_from_rfc3339(&dt)
                .ok()?
                .with_timezone(&Local);
            let end = ev
                .end
                .and_then(|t| t.date_time)
                .and_then(|dt| DateTime::parse_from_rfc3339(&dt).ok())
                .map(|dt| dt.with_timezone(&Local));
            Some(CalEvent {
                date: parsed.date_naive(),
                start: Some(parsed),
                end,
                summary,
                location: ev.location,
            })
        } else {
            let d = NaiveDate::parse_from_str(ev.start.date.as_deref()?, "%Y-%m-%d").ok()?;
            Some(CalEvent {
                date: d,
                start: None,
                end: None,
                summary,
                location: ev.location,
            })
        }
    }
}

pub fn fetch_events(
    token: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CalEvent>, Box<dyn Error>> {
    let time_min = Local
        .from_local_datetime(&from.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("ambiguous local start time")?;
    let time_max = Local
        .from_local_datetime(&(to + Duration::days(1)).and_hms_opt(0, 0, 0).unwrap())
        .single()
        .ok_or("ambiguous local end time")?;

    let resp: EventsResponse =
        ureq::get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
            .set("Authorization", &format!("Bearer {token}"))
            .query("timeMin", &time_min.to_rfc3339())
            .query("timeMax", &time_max.to_rfc3339())
            // Necessary - expands recurrence server-side
            .query("singleEvents", "true")
            .query("orderBy", "startTime")
            .query("maxResults", "250")
            .call()?
            .into_json()?;

    let mut events: Vec<CalEvent> = resp
        .items
        .into_iter()
        .filter_map(CalEvent::from_api)
        .collect();
    events.sort_by_key(|e| (e.date, e.start));
    Ok(events)
}
