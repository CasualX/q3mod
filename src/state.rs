use super::*;

mod client;
mod refdef;
mod snapshot;
mod entities;
mod refents;
mod models;

pub use self::models::Models;

pub struct UpdateContext<'a> {
	pub time: f64,
	pub process: &'a GameProcess,
	pub data: &'a GameData,

	// Wallhack needs to work while updating refEntities
	pub wallhack: &'a chefs::Wallhack,
}

#[derive(Default)]
pub struct GameState {
	pub time: f64,
	pub client: client::Client,
	pub refdef: refdef::RefDef,
	pub snap: snapshot::ClSnapshot,
	pub ents: entities::Entities,
	pub refents: refents::RefEntities,
	pub models: models::Models,
}

impl GameState {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
		self.time = ctx.time;
		self.client.update(api, ctx);
		self.refdef.update(api, ctx);
		self.snap.update(api, ctx);
		if self.snap.is_parse_entities_changed() {
			self.ents.update(api, ctx, &self.snap.raw);
		}
		self.refents.update(api, ctx, &self.models, self.client.state == 8);
		self.models.update(api, ctx);
	}

	pub fn get_config_section(&self) -> u32 {
		if !self.snap.is_valid() {
			return 0;
		}

		let ps = self.player_state();
		match ps.weapon {
			1 => hash!("Gauntlet"),
			2 => hash!("Machinegun"),
			3 => hash!("Shotgun"),
			4 => hash!("GrenadeLauncher"),
			5 => hash!("RocketLauncher"),
			6 => hash!("LightningGun"),
			7 => hash!("Railgun"),
			8 => hash!("PlasmaGun"),
			9 => hash!("BFG10K"),
			_ => hash!(""),
		}
	}
}
