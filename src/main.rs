use std::collections::HashMap;

use actix::{Actor, Addr};
use actix_web::{App, get, HttpResponse, HttpServer, Responder, web};
use hocon::HoconLoader;
use serde::Deserialize;

use crate::server_process_actor::{MapId, MessageStart, MessageStatus, MessageStop, ProcessActor};
use log::LevelFilter;

mod server_process_actor;

#[derive(Deserialize)]
struct Settings {
	log_level: String,
	working_dir: String,
	executable_name: String,
	web_port: u32,
	common_cmd_args: Vec<String>,
	maps: HashMap<MapId, MapSettings>,
}

#[derive(Deserialize)]
struct MapSettings {
	cmd_opts: String,
	cmd_args: Vec<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let settings: Settings = HoconLoader::new()
		.load_file("settings.conf").expect("Error reading settings.conf")
		.resolve().expect("Error parsing settings.conf, please check the file for errors");

	let log_level = match settings.log_level.as_str() {
		"Off" => LevelFilter::Off,
		"Error" => LevelFilter::Error,
		"Warn" => LevelFilter::Warn,
		"Info" => LevelFilter::Info,
		"Debug" => LevelFilter::Debug,
		"Trace" => LevelFilter::Trace,
		_ => { panic!("Value for log_level is invalid. Allowed values are: Off, Error, Warn, Info, Debug, Trace") }
	};

	pretty_env_logger::formatted_timed_builder()
		.filter_level(log_level)
		.init();

	let actor = server_process_actor::ProcessActor::new(&settings);
	let addr_data = web::Data::new(actor.start());

	HttpServer::new(move || {
		App::new()
			.app_data(addr_data.clone())
			.service(status)
			.service(start)
			.service(stop)
	})
		.bind(format!("127.0.0.1:{}", &settings.web_port))?
		.run()
		.await
}

#[get("/")]
async fn status(process_actor: web::Data<Addr<ProcessActor>>) -> impl Responder {
	match process_actor.send(MessageStatus {}).await {
		Ok(status_vec) => {
			let mut response = HashMap::with_capacity(10);
			status_vec.iter().for_each(|(map_id, status)| {
				response.insert(map_id, status);
			});
			HttpResponse::Ok().json(response)
		}
		Err(err) => {
			log::error!("MessageStatus Actor error: {}", err);
			HttpResponse::InternalServerError().finish()
		}
	}
}

#[get("/start/{map_id}")]
async fn start(map_id: web::Path<MapId>, process_actor: web::Data<Addr<ProcessActor>>) -> impl Responder {
	match process_actor.send(MessageStart { map_id: map_id.into_inner() }).await {
		Ok(success) => {
			HttpResponse::Ok().json(success)
		}
		Err(err) => {
			log::error!("MessageStart Actor error: {}", err);
			HttpResponse::InternalServerError().finish()
		}
	}
}

#[get("/stop/{map_id}")]
async fn stop(map_id: web::Path<MapId>, process_actor: web::Data<Addr<ProcessActor>>) -> impl Responder {
	match process_actor.send(MessageStop { map_id: map_id.into_inner() }).await {
		Ok(success) => {
			HttpResponse::Ok().json(success)
		}
		Err(err) => {
			log::error!("MessageStart Actor error: {}", err);
			HttpResponse::InternalServerError().finish()
		}
	}
}
