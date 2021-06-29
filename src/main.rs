use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex};
use subprocess::{Popen, Exec};

#[derive(Serialize, Deserialize)]
struct ClusterStatus {
	aberration: Mutex<MapState>,
	the_island: Mutex<MapState>,
	scorched_earth: Mutex<MapState>,
	the_center: Mutex<MapState>,
	ragnarok: Mutex<MapState>,
	extinction: Mutex<MapState>,
	valguero: Mutex<MapState>,
	genesis_part_1: Mutex<MapState>,
	crystal_isles: Mutex<MapState>,
	genesis_part_2: Mutex<MapState>,
}

struct ClusterProcesses {
	aberration: Mutex<MapProcess>,
	the_island: Mutex<MapProcess>,
	scorched_earth: Mutex<MapProcess>,
	the_center: Mutex<MapProcess>,
	ragnarok: Mutex<MapProcess>,
	extinction: Mutex<MapProcess>,
	valguero: Mutex<MapProcess>,
	genesis_part_1: Mutex<MapProcess>,
	crystal_isles: Mutex<MapProcess>,
	genesis_part_2: Mutex<MapProcess>,
}

struct MapProcess {
	command: String,
	arguments: [String; 3],
	child: Mutex<Option<Popen>>,
}

#[derive(Serialize, Deserialize, Clone)]
enum MapState {
	Stopped,
	Running,
}

#[derive(Serialize, Deserialize, Debug)]
enum MapId {
	Aberration,
	TheIsland,
	ScorchedEarth,
	TheCenter,
	Ragnarok,
	Extinction,
	Valguero,
	GenesisPart1,
	CrystalIsles,
	GenesisPart2,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let cluster_processes_data = web::Data::new(ClusterProcesses {
		aberration: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		the_island: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		scorched_earth: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		the_center: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		ragnarok: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		extinction: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		valguero: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		genesis_part_1: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		crystal_isles: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
		genesis_part_2: Mutex::new(MapProcess { command: "ping".to_string(), arguments: ["-n".to_string(), "30".to_string(), "127.0.0.1".to_string()], child: Mutex::new(None) }),
	});

	HttpServer::new(move || {
		App::new()
			.app_data(cluster_processes_data.clone())
			.service(status)
			.service(startup)
	})
		.bind("127.0.0.1:7776")?
		.run()
		.await
}

#[get("/")]
async fn status(cluster_processes: web::Data<ClusterProcesses>) -> impl Responder {
	HttpResponse::Ok().json(get_cluster_status(&cluster_processes))
}

#[get("/start/{map_id}")]
async fn startup(map_id: web::Path<MapId>, cluster_processes: web::Data<ClusterProcesses>) -> impl Responder {
	let i = count_running_maps(&cluster_processes);

	if i >= 2 {
		return HttpResponse::Conflict().json(get_cluster_status(&cluster_processes));
	}

	let id = map_id.into_inner();
	let map_process_mutex = match id {
		MapId::Aberration => { &cluster_processes.aberration }
		MapId::TheIsland => { &cluster_processes.the_island }
		MapId::ScorchedEarth => { &cluster_processes.scorched_earth }
		MapId::TheCenter => { &cluster_processes.the_center }
		MapId::Ragnarok => { &cluster_processes.ragnarok }
		MapId::Extinction => { &cluster_processes.extinction }
		MapId::Valguero => { &cluster_processes.valguero }
		MapId::GenesisPart1 => { &cluster_processes.genesis_part_1 }
		MapId::CrystalIsles => { &cluster_processes.crystal_isles }
		MapId::GenesisPart2 => { &cluster_processes.genesis_part_2 }
	};

	// curly bracket used to unlock cluster_processes mutex before it is used by HttpResponse.json
	{
		let process = map_process_mutex.lock().unwrap();
		let mut child = process.child.lock().unwrap();

		if is_map_running(&map_process_mutex) {
			HttpResponse::Conflict()
		} else {
			match Exec::cmd(&process.command).args(&process.arguments).popen() {
				Ok(popen) => {
					*child = Some(popen);
					HttpResponse::Ok()
				}
				Err(_) => {
					HttpResponse::InternalServerError()
				}
			}
		}
	}.json(get_cluster_status(&cluster_processes))
}

fn count_running_maps(cluster_processes: &web::Data<ClusterProcesses>) -> u8
{
	let mut i = 0;

	if is_map_running(&cluster_processes.aberration) { i += 1; }
	if is_map_running(&cluster_processes.the_island) { i += 1; }
	if is_map_running(&cluster_processes.scorched_earth) { i += 1; }
	if is_map_running(&cluster_processes.the_center) { i += 1; }
	if is_map_running(&cluster_processes.ragnarok) { i += 1; }
	if is_map_running(&cluster_processes.extinction) { i += 1; }
	if is_map_running(&cluster_processes.valguero) { i += 1; }
	if is_map_running(&cluster_processes.genesis_part_1) { i += 1; }
	if is_map_running(&cluster_processes.crystal_isles) { i += 1; }
	if is_map_running(&cluster_processes.genesis_part_2) { i += 1; }

	i
}

fn get_cluster_status(cluster_processes: &web::Data<ClusterProcesses>) -> ClusterStatus {
	ClusterStatus {
		aberration: Mutex::new(if is_map_running(&cluster_processes.aberration) { MapState::Running } else { MapState::Stopped }),
		the_island: Mutex::new(if is_map_running(&cluster_processes.the_island) { MapState::Running } else { MapState::Stopped }),
		scorched_earth: Mutex::new(if is_map_running(&cluster_processes.scorched_earth) { MapState::Running } else { MapState::Stopped }),
		the_center: Mutex::new(if is_map_running(&cluster_processes.the_center) { MapState::Running } else { MapState::Stopped }),
		ragnarok: Mutex::new(if is_map_running(&cluster_processes.ragnarok) { MapState::Running } else { MapState::Stopped }),
		extinction: Mutex::new(if is_map_running(&cluster_processes.extinction) { MapState::Running } else { MapState::Stopped }),
		valguero: Mutex::new(if is_map_running(&cluster_processes.valguero) { MapState::Running } else { MapState::Stopped }),
		genesis_part_1: Mutex::new(if is_map_running(&cluster_processes.genesis_part_1) { MapState::Running } else { MapState::Stopped }),
		crystal_isles: Mutex::new(if is_map_running(&cluster_processes.crystal_isles) { MapState::Running } else { MapState::Stopped }),
		genesis_part_2: Mutex::new(if is_map_running(&cluster_processes.genesis_part_2) { MapState::Running } else { MapState::Stopped }),
	}
}

fn is_map_running(_process: &Mutex<MapProcess>) -> bool {
	// let process_mutex = process.lock().unwrap();
	// let child_mutex = process_mutex.child.lock().unwrap();
	// let mut popen = child_mutex.unwrap();
	// match popen.poll() {
	// 	None => { true }
	// 	Some(_) => { false }
	// }
	true
}
