/*!
# Quake3 external cheat
*/

use std::fmt;
use obfstr::obfstr as s;
use fmtools::fmt as f;
use dataview::Pod;

#[macro_use]
mod base;
use self::base::hash;
use self::base::math;

mod interface;
pub use self::interface::*;

mod api;
mod sdk;
mod process;
mod data;
mod config;
mod state;
mod chefs;
mod offsets;

use self::api::*;
use self::process::GameProcess;
use self::data::*;
use self::config::ConfigLoader;
use self::state::*;
use self::chefs::*;

pub const SETTINGSUI: &str = include_str!("settingsui.json");

/// Cheat instance.
#[derive(Default)]
pub struct Instance {
	process: GameProcess,
	data: GameData,
	state: GameState,
	cheats: CheatManager,
	config: ConfigLoader,
}

impl Instance {
	fn load_gamedata(&mut self, api: &mut Api, offsets: &str) -> bool {
		let mut parser = ini_core::Parser::new(offsets);
		while let Some(item) = parser.next() {
			use ini_core::*;
			match item {
				Item::Section(name) if hash(name) == hash!("Offsets") => {
					self.data.load(api, &mut parser);

					if self.data.TimeDateStamp == self.process.time_date_stamp {
						return true;
					}
				}
				_ => {}
			}
		}
		return false;
	}

	/// Try to attach with specified gamedata.
	///
	/// Returns `false` on failure, details are logged.
	pub fn attach(&mut self, api: &mut dyn Interface, offsets: &str) -> bool {
		let api = Api(api);

		if !self.process.attach(api) {
			return false;
		}

		// First chance load the user's offsets
		if !self.load_gamedata(api, offsets) {
			// Second chance load known good offsets
			if !self.load_gamedata(api, s!(offsets::OFFSETS)) {
				// Third chance scan for offsets
				api.log(s!("Scanning for offsets..."));
				if let Ok(image) = self.process.read_image(api) {
					let mut s = String::new();
					if offsets::offsets(api, &image, &mut s) {
						self.load_gamedata(api, &s);
					}
					api.log(f!("\n"{s}));
				}
			}
		}

		if self.data.TimeDateStamp == self.process.time_date_stamp {
			api.log(s!("Attached!"));
			return true;
		}
		else {
			api.log(s!("Gamedata mismatch!"));
			api.log(f!({self.data:#x?}));
			self.data = Default::default();
			return false;
		}
	}

	/// Ticks the instance.
	///
	/// Must call [`attach`](Self::attach) before trying to tick the instance.
	/// If attach is not successful, this method does nothing.
	pub fn tick(&mut self, api: &mut dyn Interface) {
		let api = Api(api);
		let time = api.get_time();

		// Check if process is valid
		if !self.process.is_valid(api) {
			return;
		}
		// Check if gamedata is valid for this process
		if self.process.time_date_stamp != self.data.TimeDateStamp {
			return;
		}

		// Update our state of the game world
		{
			let mut ctx = UpdateContext {
				time,
				process: &self.process,
				data: &self.data,
				wallhack: &self.cheats.wallhack,
			};
			self.state.update(api, &mut ctx);
		}

		// Load the current weapon settings
		let config_section = self.state.get_config_section();
		self.config.run(api, config_section, &mut self.cheats);

		// Run the cheat modules
		{
			let mut ctx = RunContext {
				process: &self.process,
				data: &self.data,
				state: &self.state,
				screen: [0, 0],
			};
			self.cheats.run(api, &mut ctx);
		}
	}

	/// Loads a config string.
	pub fn load_config(&mut self, api: &mut dyn Interface, config: &str) {
		let api = Api(api);
		self.config.loads(api, config, &mut self.cheats);
	}
}

impl cvar::IVisit for Instance {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		self.cheats.visit(f);
		f(&mut cvar::List(s!("config"), &mut self.config));
	}
}
