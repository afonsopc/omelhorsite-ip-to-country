use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use envconfig::Envconfig;
use maxminddb::{geoip2::Country, Reader};
use serde::Serialize;
use std::{net::IpAddr, sync::Mutex};
use tokio::{task, time};

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_PATH")]
    pub database_path: String,
    #[envconfig(from = "RELOAD_DATABASE_INTERVAL")]
    pub reload_database_interval: u64,
    #[envconfig(from = "HOST")]
    pub host: String,
    #[envconfig(from = "PORT")]
    pub port: u16,
}

#[derive(Debug)]
struct AppState {
    database: Mutex<Reader<Vec<u8>>>,
}

#[derive(Debug)]
enum GetCountryFromIpError {
    MaxMindDbError(maxminddb::MaxMindDBError),
    AddrParseError(std::net::AddrParseError),
    NoCountryFound,
}

fn get_country_from_ip(
    ip: &str,
    database: &Reader<Vec<u8>>,
) -> Result<String, GetCountryFromIpError> {
    let ip: IpAddr = ip.parse().map_err(GetCountryFromIpError::AddrParseError)?;
    let country: Option<Country> = database
        .lookup(ip)
        .map_err(GetCountryFromIpError::MaxMindDbError)?;
    match country {
        Some(c) => Ok(c.country.unwrap().iso_code.unwrap().to_string()),
        None => Err(GetCountryFromIpError::NoCountryFound),
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    let cool_response = "Se Deus quiser, há-de brilhar
De novo a Coroa sobre as Lusas armas
Que a nossa Pátria soube, sempre honrar
Que a nossa Pátria soube, sempre honrar";
    HttpResponse::Ok().body(cool_response)
}

#[derive(Serialize)]
struct CountryResponse {
    country: String,
}

#[get("/{ip}")]
async fn echo(ip: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let database = match state.database.lock() {
        Ok(value) => value,
        Err(err) => {
            println!("Error getting database: {:?}", err);
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    };

    match get_country_from_ip(&ip.to_string(), &database) {
        Ok(country) => {
            let response = CountryResponse { country };
            HttpResponse::Ok().json(response)
        }
        Err(GetCountryFromIpError::AddrParseError(_)) => {
            HttpResponse::NotFound().body("IP is not valid")
        }
        Err(err) => {
            println!(
                "Error finding IP country. IP: {}, Error: {:?}",
                &ip.to_string(),
                err
            );
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting");

    dotenv().ok();
    let config = Config::init_from_env().unwrap();

    let app_state = web::Data::new(AppState {
        database: Mutex::new(maxminddb::Reader::open_readfile(&config.database_path).unwrap()),
    });

    // CHECK IF DATABASE IS LOADED
    let ip = "1.1.1.1";
    match get_country_from_ip(ip, &app_state.database.lock().unwrap()) {
        Ok(country) => println!("Country for IP \"{ip}\": \"{country}\""),
        Err(_) => panic!("Country for IP \"{ip}\" not found"),
    }

    // START A THREAD THAT RELOADS THE DATABASE
    // AND UPDATES THE APP STATE
    let app_state_clone = app_state.clone();
    task::spawn(async move {
        loop {
            time::sleep(std::time::Duration::from_secs(
                config.reload_database_interval,
            ))
            .await;
            println!("Reloading database");
            let new_database = maxminddb::Reader::open_readfile(&config.database_path).unwrap();
            let mut database = app_state_clone.database.lock().unwrap();
            *database = new_database;
        }
    });

    // START THE SERVER
    println!("Listening on {}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(echo)
            .app_data(app_state.clone())
    })
    .bind((config.host, config.port))?
    .run()
    .await
}
