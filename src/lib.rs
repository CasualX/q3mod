/*!
# Quake3 external cheat
*/

use std::{fmt, mem};
use obfstr::obfstr as s;
use fmtools::fmt as f;
use dataview::Pod;

#[macro_use]
mod base;
use self::base::hash;
use self::base::math;

mod interface;
pub use self::interface::Interface;

mod api;
mod sdk;
mod process;
mod data;
mod config;
mod state;
mod chefs;

use self::api::*;
use self::process::GameProcess;
use self::data::*;
use self::config::ConfigLoader;
use self::state::*;
use self::chefs::*;

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
	/// Try to attach with specified gamedata.
	///
	/// Returns `false` on failure, details are logged.
	pub fn attach(&mut self, api: &mut dyn Interface) -> bool {
		let api = Api(api);

		if !self.process.attach(api) {
			return false;
		}

		for gd in &OFFSETS {
			if gd.TimeDateStamp == self.process.time_date_stamp {
				self.data = *gd;
				api.log(f!("Attached!"));
				return true;
			}
		}

		api.log(f!("Gamedata mismatch!"));
		return false;
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
