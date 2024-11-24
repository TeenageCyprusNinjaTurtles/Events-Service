use std::io::Error;

use r2d2_postgres::{postgres, PostgresConnectionManager};

use crate::PGPool;

pub fn init_table(pool: PGPool) -> Option<Error> {
    let mut conn = pool.get().unwrap();
    let result = conn.execute(
        "CREATE TABLE IF NOT EXISTS platform_events (
                id SERIAL PRIMARY KEY,
                date TIMESTAMP NOT NULL,
                name VARCHAR(255) NOT NULL,
                location VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                duration VARCHAR(255) NOT NULL,
                creator_id VARCHAR(255) NOT NULL
            )",
        &[],
    );
    match result {
        Ok(_) => {
            log::info!("Table platform_events created");
        }
        Err(err) => {
            log::error!("Error creating table: {}", err);
            return Some(Error::new(
                std::io::ErrorKind::Other,
                "Error with table events_revisions creation",
            ));
        }
    }

    match conn.execute(
        "CREATE TABLE IF NOT EXISTS events_revisions(
        date TIMESTAMP NOT NULL,
        revision_id VARCHAR(255) NOT NULL
    )",
        &[],
    ) {
        Ok(_) => {
            log::info!("Table events_revisions created");
        }
        Err(err) => {
            log::error!("Error creating table: {}", err);
            return Some(Error::new(
                std::io::ErrorKind::Other,
                "Error with table events_revisions creation",
            ));
        }
    }

    return None;
}

pub fn migration(pool: PGPool) -> Option<Error> {
    let mut conn = pool.get().unwrap();
    let last_revision = conn
        .query("SELECT MAX(revision_id) FROM events_revisions", &[])
        .unwrap();
    if last_revision.len() == 0 {
        log::info!("No revision found");
        return None;
    }
    let last_revision_id: String = match last_revision.get(0) {
        Some(row) => {
            let revision_id: Option<String> = row.get(0);
            if revision_id.is_none() {
                log::info!("Revision ID is NULL");
                return None;
            }
            revision_id.unwrap()
        }
        None => {
            log::error!("No revision found");
            return Some(Error::new(std::io::ErrorKind::Other, "No revision found"));
        }
    };
    log::info!("Last revision: {}", last_revision_id);
    None
}

pub fn put_current_revision(pool: PGPool, revision_id: &str) {
    let mut conn = pool.get().unwrap();
    let result = conn.execute(
        "INSERT INTO events_revisions (date, revision_id) VALUES (NOW(), $1)",
        &[&revision_id],
    );
    match result {
        Ok(_) => {
            log::info!("Current revision inserted");
        }
        Err(err) => {
            log::error!("Error inserting current revision: {}", err);
        }
    }
}
