use super::*;

struct Config {
	all: bool,
	players: bool,
	weapons: bool,
	powerups: bool,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			all: false,
			players: true,
			weapons: true,
			powerups: true,
		}
	}
}

#[derive(Default)]
pub struct Wallhack {
	config: Config,
}

impl cvar::IVisit for Wallhack {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		let default = Config::default();
		f(&mut cvar::Property(s!("all"), &mut self.config.all, &default.all));
		f(&mut cvar::Property(s!("players"), &mut self.config.players, &default.players));
		f(&mut cvar::Property(s!("weapons"), &mut self.config.weapons, &default.weapons));
		f(&mut cvar::Property(s!("powerups"), &mut self.config.powerups, &default.powerups));
	}
}

impl Wallhack {
	pub fn run(&mut self, _api: &mut Api, _ctx: &RunContext) {}

	fn filter(&self, models: &state::Models, r_ent: &sdk::refEntity_t) -> bool {
		if self.config.all {
			return true;
		}
		let Some(model_name) = models.get_name(r_ent.hModel) else { return false };
		if self.config.players && model_name.starts_with(s!("models/player")) {
			return true;
		}
		if self.config.weapons && model_name.starts_with(s!("models/weapon")) {
			return true;
		}
		if self.config.powerups && model_name.starts_with(s!("models/powerup")) {
			return true;
		}
		return false;
	}
	pub fn poke(&self, api: &mut Api, models: &state::Models, ptr: sdk::Ptr<sdk::refEntity_t>, r_ent: &sdk::refEntity_t) {
		if self.filter(models, r_ent) {
			let _ = api.vm_write(ptr.field(4), &(r_ent.renderfx | sdk::RF_DEPTHHACK | sdk::RF_NOSHADOW));
		}
	}
}
