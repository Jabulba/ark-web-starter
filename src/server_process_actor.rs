use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use actix::{Actor, Context, Handler};
use actix::prelude::*;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use subprocess::{Exec, Popen};

use crate::Settings;

pub(crate) struct MessageStatus;

impl Message for MessageStatus {
	type Result = Vec<(MapId, State)>;
}

pub(crate) struct MessageStart {
	pub(crate) map_id: MapId,
}

impl Message for MessageStart {
	type Result = bool;
}

pub(crate) struct MessageStop {
	pub(crate) map_id: MapId,
}

impl Message for MessageStop {
	type Result = bool;
}

pub(crate) struct ProcessActor {
	maps: HashMap<MapId, Exec>,
	processes: DashMap<MapId, Popen>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum MapId {
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

impl Display for MapId {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			MapId::Aberration => write!(f, "Aberration"),
			MapId::TheIsland => write!(f, "TheIsland"),
			MapId::ScorchedEarth => write!(f, "ScorchedEarth"),
			MapId::TheCenter => write!(f, "TheCenter"),
			MapId::Ragnarok => write!(f, "Ragnarok"),
			MapId::Extinction => write!(f, "Extinction"),
			MapId::Valguero => write!(f, "Valguero"),
			MapId::GenesisPart1 => write!(f, "GenesisPart1"),
			MapId::CrystalIsles => write!(f, "CrystalIsles"),
			MapId::GenesisPart2 => write!(f, "GenesisPart2"),
		}
	}
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) enum State {
	Running,
	Stopped,
}

impl Actor for ProcessActor {
	type Context = Context<ProcessActor>;

	fn started(&mut self, _ctx: &mut Self::Context) {
		log::info!("Ark Instance Process Actor has started.");
	}

	fn stopped(&mut self, _ctx: &mut Self::Context) {
		log::info!("Ark Instance Process Actor has stopped.");
	}
}

impl ProcessActor {
	pub fn new(settings: &Settings) -> ProcessActor {
		let mut maps = HashMap::with_capacity(10);

		// TODO: change insert_map to construct/build Exec? do insert here and not in fn?
		Self::insert_map(&mut maps, MapId::Aberration, &settings);
		Self::insert_map(&mut maps, MapId::TheIsland, &settings);
		Self::insert_map(&mut maps, MapId::ScorchedEarth, &settings);
		Self::insert_map(&mut maps, MapId::TheCenter, &settings);
		Self::insert_map(&mut maps, MapId::Ragnarok, &settings);
		Self::insert_map(&mut maps, MapId::Extinction, &settings);
		Self::insert_map(&mut maps, MapId::Valguero, &settings);
		Self::insert_map(&mut maps, MapId::GenesisPart1, &settings);
		Self::insert_map(&mut maps, MapId::CrystalIsles, &settings);
		Self::insert_map(&mut maps, MapId::GenesisPart2, &settings);

		ProcessActor {
			maps,
			processes: DashMap::with_capacity(10),
		}
	}

	fn insert_map(maps: &mut HashMap<MapId, Exec>, map: MapId, settings: &Settings) -> () {
		let map_settings = match settings.maps.get(&map) {
			None => { panic!("Settings for map '{}' are missing from settings.conf", map) }
			Some(map_settings) => { map_settings }
		};

		let exec = Exec::cmd(format!("{}/{}", &settings.working_dir, &settings.executable_name))
			.arg(&map_settings.cmd_opts)
			.args(&map_settings.cmd_args)
			.args(&settings.common_cmd_args)
			.cwd(&settings.working_dir);

		maps.insert(map, exec);
	}
}

impl Handler<MessageStatus> for ProcessActor {
	type Result = Vec<(MapId, State)>;

	fn handle(&mut self, _msg: MessageStatus, _ctx: &mut Context<Self>) -> Self::Result {
		let maps = &self.maps;
		let processes = &self.processes;
		let mut map_states = Vec::with_capacity(10);

		maps.keys().map(|map_id| { map_id.clone() })
			.for_each(|map_id| {
				let state = get_map_state(&map_id, processes);
				map_states.push((map_id, state));
			});

		map_states
	}
}

impl Handler<MessageStart> for ProcessActor {
	type Result = bool;

	fn handle(&mut self, msg: MessageStart, _ctx: &mut Self::Context) -> Self::Result {
		let map_id = &msg.map_id;
		let maps = &self.maps;
		let processes = &self.processes;

		let mut running_maps = 0;
		processes.iter_mut().for_each(|mut process| {
			match process.poll() {
				None => {running_maps += 1;}
				Some(_) => {}
			}
		});

		if running_maps >= 2 {
			log::error!("Failed to start map '{}': Too many maps running.", map_id);
			return false;
		}

		let state = get_map_state(map_id, processes);
		if state != State::Stopped {
			log::error!("Failed to start map '{}': The map is already running.", map_id);
			return false;
		}

		match maps.get(map_id) {
			None => {
				log::error!("Unable to start map '{}': No PopenConfig has been defined!", map_id);
				false
			}
			Some(map_exec) => {
				match map_exec.clone().popen() {
					Ok(popen) => {
						log::info!("Starting map '{}'!", map_id);
						processes.insert(map_id.clone(), popen);
						true
					}
					Err(err) => {
						log::error!("Failed to start map '{}': {}", map_id, err);
						false
					}
				}
			}
		}
	}
}


impl Handler<MessageStop> for ProcessActor {
	type Result = bool;

	fn handle(&mut self, msg: MessageStop, _ctx: &mut Self::Context) -> Self::Result {
		let map_id = &msg.map_id;
		let processes = &self.processes;

		let state = get_map_state(map_id, processes);
		match state {
			State::Running => {
				match processes.get_mut(map_id) {
					None => {
						log::error!("Failed to stop map '{}': Popen was not found in DashMap.", map_id);
						false
					}
					Some(mut popen) => {
						log::info!("Sending SIGTERM to process for map '{}'.", map_id);
						match popen.terminate() {
							Ok(_) => { true }
							Err(err) => {
								log::error!("Failed to stop process for map '{}': {}", map_id, err);
								false
							}
						}
					}
				}
			}
			State::Stopped => {
				log::error!("Failed to stop map '{}': The map is already stopped.", map_id);
				false
			}
		}
	}
}

fn get_map_state(map_id: &MapId, processes: &DashMap<MapId, Popen>) -> State {
	match processes.get_mut(&map_id) {
		None => { State::Stopped }
		Some(mut popen) => {
			match popen.poll() {
				None => { State::Running }
				Some(_) => {
					State::Stopped
				}
			}
		}
	}
}