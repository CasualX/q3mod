use super::*;

#[derive(Copy, Clone, Default)]
struct Powerups {
	quad: bool,
	suit: bool,
	haste: bool,
	invis: bool,
	regen: bool,
	flight: bool,
	scout: bool,
	guard: bool,
	doubler: bool,
	ammoregen: bool,
	invuln: bool,
}

impl cvar::IVisit for Powerups {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		f(&mut cvar::Property(s!("quad"), &mut self.quad, &false));
		f(&mut cvar::Property(s!("suit"), &mut self.suit, &false));
		f(&mut cvar::Property(s!("haste"), &mut self.haste, &false));
		f(&mut cvar::Property(s!("invis"), &mut self.invis, &false));
		f(&mut cvar::Property(s!("regen"), &mut self.regen, &false));
		f(&mut cvar::Property(s!("flight"), &mut self.flight, &false));
		f(&mut cvar::Property(s!("scout"), &mut self.scout, &false));
		f(&mut cvar::Property(s!("guard"), &mut self.guard, &false));
		f(&mut cvar::Property(s!("doubler"), &mut self.doubler, &false));
		f(&mut cvar::Property(s!("ammoregen"), &mut self.ammoregen, &false));
		f(&mut cvar::Property(s!("invuln"), &mut self.invuln, &false));
	}
}

struct Config {
	enable: bool,
	ready: bool,
	infammo: bool,
	godmode: bool,
	givewp: bool,
	numweapons: u8,
	fastfire: bool,
	interval: f32,
	powerups: Powerups,
}

impl Default for Config {
	fn default() -> Config {
		Config {
			enable: true,
			ready: false,
			infammo: true,
			godmode: false,
			givewp: false,
			numweapons: 11,
			fastfire: false,
			interval: 1.0 / 60.0,
			powerups: Powerups {
				quad: false,
				suit: false,
				haste: true,
				invis: false,
				regen: false,
				flight: false,
				scout: false,
				guard: false,
				doubler: false,
				ammoregen: false,
				invuln: false,
			}
		}
	}
}

#[derive(Default)]
pub struct Trainer {
	timer: base::Timer,

	// Detect when powerups are disabled
	powerups: Powerups,

	config: Config,
}

impl cvar::IVisit for Trainer {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		let default = Config::default();
		f(&mut cvar::Property(s!("enable"), &mut self.config.enable, &default.enable));
		f(&mut cvar::ReadOnlyProp(s!("ready"), &self.config.ready, &default.ready));
		f(&mut cvar::Property(s!("infammo"), &mut self.config.infammo, &default.infammo));
		f(&mut cvar::Property(s!("godmode"), &mut self.config.godmode, &default.godmode));
		f(&mut cvar::Property(s!("givewp"), &mut self.config.givewp, &default.givewp));
		f(&mut cvar::Property(s!("numweapons"), &mut self.config.numweapons, &default.numweapons));
		f(&mut cvar::Property(s!("fastfire"), &mut self.config.fastfire, &default.fastfire));
		f(&mut cvar::Property(s!("interval"), &mut self.config.interval, &default.interval));
		f(&mut cvar::List(s!("powerup"), &mut self.config.powerups));
	}
}

impl Trainer {
	fn sv_game_client(&self, api: &mut Api, ctx: &RunContext) -> Option<sdk::Ptr<sdk::playerState_t>> {
		let client = ctx.state.player_state().clientNum;
		if ctx.data.sv_gameClients == 0 || client < 0 {
			return None;
		}

		// Small part of the server_t structure
		#[derive(dataview::Pod)]
		#[repr(C)]
		struct SvGameClients {
			game_clients: sdk::Ptr,
			game_client_size: u32,
			_pad: u32,
		}

		let gc = api.vm_read::<SvGameClients>(ctx.process.base.field(ctx.data.sv_gameClients)).ok()?;
		if gc.game_clients.is_null() {
			return None;
		}

		Some(gc.game_clients.field(gc.game_client_size * client as u32))
	}

	pub fn run(&mut self, api: &mut Api, ctx: &RunContext) {
		self.config.ready = ctx.data.sv_gameClients != 0;

		if !self.config.enable {
			return;
		}

		let state = ctx.state;

		// The fastfire effect needs to be applied as often as possible
		if self.config.fastfire {
			let Some(client_ptr) = self.sv_game_client(api, ctx) else { return };
			let _ = api.vm_write(client_ptr.field(dataview::offset_of!(sdk::playerState_t.weaponTime) as u32), &0i32);
		}

		// The rest can be applied far less often
		if self.timer.has_elapsed(state.time, self.config.interval as f64) {
			let Some(client_ptr) = self.sv_game_client(api, ctx) else { return };
			let Ok(mut client) = api.vm_read(client_ptr) else { return };

			if self.config.godmode {
				client.stats[3] = 200;
				client.stats[0] = 200;
			}
			if self.config.givewp {
				client.stats[2] = (1 << self.config.numweapons) - 1 - (1 << sdk::WP_NONE);
			}
			let _ = api.vm_write(client_ptr.field(dataview::offset_of!(sdk::playerState_t.stats) as u32), &client.stats[..4]);

			// Infinite ammo:
			// Just set your ammo to -1
			if self.config.infammo {
				let ammo = [-1; sdk::MAX_WEAPONS];
				let _ = api.vm_write(client_ptr.field(dataview::offset_of!(sdk::playerState_t.ammo) as u32), &ammo);
			}

			// Modify your powerups

			let powerups = [
				(self.config.powerups.quad, self.powerups.quad, sdk::PW_QUAD),
				(self.config.powerups.suit, self.powerups.suit, sdk::PW_BATTLESUIT),
				(self.config.powerups.haste, self.powerups.haste, sdk::PW_HASTE),
				(self.config.powerups.invis, self.powerups.invis, sdk::PW_INVIS),
				(self.config.powerups.regen, self.powerups.regen, sdk::PW_REGEN),
				(self.config.powerups.flight, self.powerups.flight, sdk::PW_FLIGHT),
				(self.config.powerups.scout, self.powerups.scout, sdk::PW_SCOUT),
				(self.config.powerups.guard, self.powerups.guard, sdk::PW_GUARD),
				(self.config.powerups.doubler, self.powerups.doubler, sdk::PW_DOUBLER),
				(self.config.powerups.ammoregen, self.powerups.ammoregen, sdk::PW_AMMOREGEN),
				(self.config.powerups.invuln, self.powerups.invuln, sdk::PW_INVULNERABILITY),
			];

			let mut powerups_changed = false;
			let server_time = state.snap.raw.serverTime;
			for &(config, changed, index) in powerups.iter() {
				if config {
					// The powerup value is the time when the powerup expires
					// Just keep setting it to a higher value than the server time
					client.powerups[index as usize] = server_time + 100000;
					powerups_changed = true;
				}
				else if changed {
					// Let the powerup expire naturally by the game
					client.powerups[index as usize] = server_time;
					powerups_changed = true;
				}
			}

			if powerups_changed {
				let _ = api.vm_write(client_ptr.field(0x138), &client.powerups);
			}

			// Keep track of changes in powerup config
			self.powerups = self.config.powerups;
		}
	}
}
