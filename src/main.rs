use rouille::{input::post, Request, Response};
use std::env;
use r2d2_postgres::{PostgresConnectionManager, postgres};
use log::{info, error};

type PGPool = r2d2::Pool<PostgresConnectionManager<postgres::NoTls>>;


mod events_management;
mod events_controller;
mod lib;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();
    let server = env::var("LISTEN_HOST").unwrap_or("localhost".to_string());
    let port = env::var("LISTEN_PORT").unwrap_or("5001".to_string());
    

    let db_host: String = env::var("DATABASE_HOST").unwrap_or("localhost".to_string());
    let db_port: String = env::var("DATABASE_PORT").unwrap_or("5432".to_string());
    let db_user: String = env::var("DATABASE_USER").unwrap_or("postgres".to_string());
    let db_pass: String = env::var("DATABASE_PASS").unwrap_or("cHt0UFBbszX0YK7".to_string());
    let db_pool: u32 = env::var("DATABASE_POOL").unwrap_or("1".to_string()).parse().unwrap();
    let connection_string = format!("postgres://{}:{}@{}:{}", db_user, db_pass, db_host, db_port);
    info!("Connection String: {}", connection_string);
    let connection_manager = PostgresConnectionManager::new(
        connection_string.parse().unwrap(),
        postgres::NoTls
    );

    let pool = r2d2::Pool::builder().max_size(db_pool).build(connection_manager).unwrap();
    match events_management::init_table(pool.clone()) {
        Some(err) => {
            error!("Error creating table: {}", err);
            return;
        },
        None => {},
    }
    info!("Database connection established");

    let addr = format!("{}:{}", server, port);
    info!("Server listening on {}", addr);
    
    events_management::init_table(pool.clone());
    events_management::migration(pool.clone());
    events_management::put_current_revision(pool.clone(), "1");


    rouille::start_server(addr, move |request| {
        if request.url() == "/event/create" {
            events_controller::create_event(request, pool.clone())
        } else if request.url() == "/event/get_one" {
            events_controller::get_event(request, pool.clone())
        } else if request.url() == "/event/get_many" {
            events_controller::get_all_events(request, pool.clone())
        } else 
        {
            Response::empty_404()
        }

    });
    
    
    

}