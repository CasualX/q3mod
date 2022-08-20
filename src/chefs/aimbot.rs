use super::*;

#[derive(Default)]
pub struct TargetInfo {
	pub origin: [f32; 3],
	pub velocity: [f32; 3],

	pub hit: [f32; 3],
	pub aim: [f32; 3],
	pub distance: f32,
	pub time: f32,

	pub angle: f32,
	pub pitch: f32,
	pub yaw: f32,
}

struct Config {
	enable: bool,
	tickrate: f32,
	aim_fov: f32,
	aim_key: u8,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enable: true,
			tickrate: 500.0,
			aim_fov: 10.0,
			aim_key: /*XBUTTON1*/0x05,
		}
	}
}

#[derive(Default)]
pub struct Aimbot {
	timer: base::Timer,
	config: Config,

	target_entnum: i32,
	target_locked: bool,
}

impl cvar::IVisit for Aimbot {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		let default = Config::default();
		f(&mut cvar::Property(s!("enable"), &mut self.config.enable, &default.enable));
		f(&mut cvar::Property(s!("rate"), &mut self.config.tickrate, &default.tickrate));
		f(&mut cvar::Property(s!("aim_fov"), &mut self.config.aim_fov, &default.aim_fov));
	}
}

impl Aimbot {
	pub fn run(&mut self, api: &mut Api, ctx: &RunContext) {
		if !self.config.enable {
			self.target_entnum = -1;
			return;
		}

		// Run the aimbot at specified rate
		if !self.timer.has_elapsed(ctx.state.time, 1.0 / self.config.tickrate as f64) {
			return;
		}

		if !api.is_vkey_down(self.config.aim_key) {
			self.target_entnum = -1;
			self.target_locked = false;
			return;
		}

		if !self.target_locked {
			self.find_target(ctx.state);
		}

		let mut info = TargetInfo::default();

		if let Some(target_ent) = ctx.state.get_entity(self.target_entnum) {
			if self.validate(ctx.state, target_ent, &mut info) {
				self.target_locked = true;
				ctx.set_view_angles(api, info.aim);
				return;
			}
			// else {
			// 	self.target_client = -1;
			// }
		}
	}

	fn find_target(&mut self, state: &GameState) {
		let mut target_ent = None;
		let mut target_angle = 999.0;
		let mut info = TargetInfo::default();
		for ent in state.entities() {
			if self.validate(state, ent, &mut info) {
				if info.angle < target_angle {
					target_ent = Some(ent);
					target_angle = info.angle;
				}
			}
		}

		if let Some(target_ent) = target_ent {
			self.target_entnum = target_ent.number;
			self.target_locked = true;
		}
	}

	fn validate(&self, state: &GameState, ent: &sdk::entityState_t, info: &mut TargetInfo) -> bool {
		let ps = state.player_state();

		let mut shoot_pos = ps.origin;
		shoot_pos[2] += ps.viewheight as f32;

		if ent.clientNum == ps.clientNum {
			return false;
		}
		if ent.eType != sdk::ET_PLAYER {
			return false;
		}
		if ent.eFlags & 1 != 0 {
			return false;
		}

		info.origin = ent.pos.trBase;
		info.velocity = ent.pos.trDelta;

		let pspeed = match ps.weapon {
			4 => 700.0, // grenade
			8 => 2000.0, // plasma
			5 => 900.0, // rl
			9 => 2000.0, // bfg
			_ => 0.0,
		};

		info.distance = math::dist(shoot_pos, info.origin);

		if pspeed > 0.0 {
			info.time = info.distance / pspeed;
			info.hit = math::add(info.origin, math::muls(info.velocity, info.time));
		}
		else {
			info.time = 0.0;
			info.hit = info.origin;
		}
		info.aim = math::qnorm(math::qangle(math::sub(info.hit, shoot_pos)));

		let va = state.view_angles();
		info.pitch = info.aim[0] - va[0];
		info.yaw = info.aim[1] - va[1];
		while info.yaw > 180.0 { info.yaw -= 360.0; }
		while info.yaw <= -180.0 { info.yaw += 360.0; }
		info.angle = (info.yaw * info.yaw + info.pitch * info.pitch).sqrt();
		if !self.target_locked && info.angle > self.config.aim_fov {
			return false;
		}

		return true;
	}
}
