use rouille::{Request, Response};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::SystemTime};

use crate::{lib, PGPool};

#[derive(Serialize, Deserialize)]
struct EventObject {
    name: String,
    location: String,
    start: String,
    duration: String,
    description: String,
    id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CreateEventResponse {
    id: i32,
}

#[derive(Serialize, Deserialize)]
struct GetEventRequest {
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
struct GetEventsResponse {
    events: Vec<EventObject>
}

pub fn create_event(request: &Request, pool: PGPool) -> Response {
    let mut conn = pool.get().unwrap();
    let event = rouille::input::json_input(request);
    if event.is_err() {
        log::error!("Error parsing event: {:?}", event.err());
        return lib::utils::return_result(lib::enums::ERROR_RESPONSE_INVALID_JSON.to_string());
    }
    let event: EventObject = event.unwrap();
    let creator = lib::utils::get_header_value(request, "X-User-Email");
    if creator.is_none() {
        log::error!("No creator found");
        return lib::utils::return_result(lib::enums::ERROR_RESPONSE_ACCESS_ERROR.to_string());
    }

    let level = lib::utils::get_user_level(request);
    if level < 2 {
        log::error!("User level too low");
        return lib::utils::return_result(lib::enums::ERROR_RESPONSE_ACCESS_ERROR.to_string());
    }

    let creator = creator.unwrap();
    
    let date = chrono::NaiveDateTime::parse_from_str(&event.start, "%Y-%m-%d %H:%M:%S").unwrap();
    let date: SystemTime = date.and_local_timezone(chrono::Local).unwrap().into();
    let result = conn.query(
        "INSERT INTO platform_events (name, location, date, duration, description, creator_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        &[&event.name, &event.location, &date, &event.duration, &event.description, &creator]
    );
    match result {
        Ok(result) => {
            let row = result.get(0);
            if row.is_none() {
                log::error!("No event found");
                return lib::utils::return_result(lib::enums::ERROR_RESPONSE_DOESNT_EXISTS.to_string());
            }
            let id: i32 = row.unwrap().get("id");

            return Response::json(&CreateEventResponse { id });
        }
        Err(err) => {
            log::error!("Error creating event: {}", err);
            return lib::utils::return_result(lib::enums::ERROR_RESPONSE_INVALID_JSON.to_string());
        }
    }
}


pub fn get_event(request: &Request, pool: PGPool) -> Response {
    let mut conn = pool.get().unwrap();
    let data = lib::utils::request_to_bytes(request);
    let request = serde_json::from_slice::<GetEventRequest>(&data);
    if request.is_err() {
        log::error!("Error parsing request: {:?}", request.err());
        return lib::utils::return_result(lib::enums::ERROR_RESPONSE_INVALID_JSON.to_string());
    }
    let request = request.unwrap();
    let result = conn.query(
        "SELECT name, location, date, duration, description FROM platform_events WHERE id = $1",
        &[&request.id]
    );
    if result.is_err() {
        log::error!("Error querying event: {:?}", result.err());
        return lib::utils::return_result(lib::enums::ERROR_RESPONSE_INVALID_JSON.to_string());
    } else {
        let rows = result.unwrap();
        let row = rows.get(0);
        if row.is_none() {
            log::error!("No event found");
            return lib::utils::return_result(lib::enums::ERROR_RESPONSE_INVALID_JSON.to_string());
        }
        let record = row.unwrap();
        let date: SystemTime = record.get("date");
        let datetime = chrono::DateTime::<chrono::Local>::from(date);
        let event = EventObject {
            name: record.get("name"),
            location: record.get("location"),
            start: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
            duration: record.get("duration"),
            description: record.get("description"),
            id: Some(request.id.to_string()),
        };
        return Response::json(&event);
    }
}


pub fn get_all_events(request: &Request, pool: PGPool) -> Response {
    let mut conn = pool.get().unwrap();
    let result = conn.query(
        "SELECT id, name, location, date, duration, description FROM platform_events",
        &[]
    );
    if result.is_err() {
        log::error!("Error querying events: {:?}", result.err());
        return lib::utils::return_result(lib::enums::ERROR_RESPONSE_INVALID_JSON.to_string());
    } else {
        let rows = result.unwrap();
        let mut events = Vec::new();
        for row in rows.iter() {
            let date: SystemTime = row.get("date");
            let id: i32 = row.get("id");
            let datetime = chrono::DateTime::<chrono::Local>::from(date);
            let event = EventObject {
                name: row.get("name"),
                location: row.get("location"),
                start: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                duration: row.get("duration"),
                description: row.get("description"),
                id: Some(id.to_string()),
            };
            events.push(event);
        }
        return Response::json(&GetEventsResponse{events:events});
    }
}