use std::collections::VecDeque;

use super::*;
use crate::base::pid::{PidConfig, PidController};
use crate::base::solver;

#[derive(Debug, Default)]
pub struct TargetInfo {
	pub origin: [f32; 3],
	pub velocity: [f32; 3],
	pub grounded: bool,
	pub spine: [f32; 2],
	pub distance: f32,

	pub hit: [f32; 3],
	pub aim: [f32; 3],
	pub ptime: f32,

	// Relative angles to aim at the target
	pub angle: f32,
	pub pitch: f32,
	pub yaw: f32,
}

/// A target position in the past.
///
/// Used to modulate aim strength on unpredictable targets.
struct TargetPos {
	time: f64,
	origin: [f32; 3],
	velocity: [f32; 3],
}

struct Config {
	enable: bool,
	debug: bool,
	tickrate: f32,
	aim_key: Button,
	aim_pitch: f32,
	aim_yaw: f32,
	aim_ramp: f32,
	aim_idletime: f32,
	aim_mintime: f32,
	aim_cooldown_pow: f32,
	aim_react: f32,
	aim_react_dist: f32,
	aim_react_pow: f32,
	fov_radius: f32,
	fov_aim: f32,
	fov_min: f32,
	teledist: f32,
	pid: PidConfig,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enable: true,
			debug: false,
			tickrate: 144.0,
			aim_key: Button::MouseX1,
			aim_pitch: 0.3,
			aim_yaw: 1.0,
			aim_ramp: 0.3,
			aim_idletime: 0.3,
			aim_mintime: 0.3,
			aim_cooldown_pow: 3.5,
			aim_react: 0.1,
			aim_react_dist: 40.0,
			aim_react_pow: 4.0,
			fov_radius: 40.0,
			fov_aim: 10.0,
			fov_min: 0.5,
			teledist: 72.0,
			pid: PidConfig {
				kp: 1.2,
				ki: 10.0,
				kd: 0.0,
				damp: 0.9,
			},
		}
	}
}

#[derive(Default)]
pub struct Aimbot {
	timer: base::Timer,

	// Aiming state
	addx: f32,
	addy: f32,
	pidx: PidController,
	pidy: PidController,

	// Weapon state
	weaponstate: i32,
	weapon_tfiring: f64,

	// Modulate the aim strength
	aim_mod: f32,

	// Current aim target
	target_entnum: i32,
	target_locked: bool,
	target_tstart: f64,
	target_tend: f64,
	target_pos: [f32; 3],
	target_prederror: f32,
	target_history: VecDeque<TargetPos>,

	config: Config,
}

impl cvar::IVisit for Aimbot {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		let default = Config::default();
		f(&mut cvar::Property(s!("enable"), &mut self.config.enable, &default.enable));
		f(&mut cvar::Property(s!("debug"), &mut self.config.debug, &default.debug));
		f(&mut cvar::Property(s!("rate"), &mut self.config.tickrate, &default.tickrate));
		f(&mut cvar::Property(s!("aim_pitch"), &mut self.config.aim_pitch, &default.aim_pitch));
		f(&mut cvar::Property(s!("aim_yaw"), &mut self.config.aim_yaw, &default.aim_yaw));
		f(&mut cvar::Property(s!("aim_ramp"), &mut self.config.aim_ramp, &default.aim_ramp));
		f(&mut cvar::Property(s!("aim_idletime"), &mut self.config.aim_idletime, &default.aim_idletime));
		f(&mut cvar::Property(s!("aim_mintime"), &mut self.config.aim_mintime, &default.aim_mintime));
		f(&mut cvar::Property(s!("aim_cooldown_pow"), &mut self.config.aim_cooldown_pow, &default.aim_cooldown_pow));
		f(&mut cvar::Property(s!("aim_react"), &mut self.config.aim_react, &default.aim_react));
		f(&mut cvar::Property(s!("aim_react_dist"), &mut self.config.aim_react_dist, &default.aim_react_dist));
		f(&mut cvar::Property(s!("aim_react_pow"), &mut self.config.aim_react_pow, &default.aim_react_pow));
		f(&mut cvar::Property(s!("fov_radius"), &mut self.config.fov_radius, &default.fov_radius));
		f(&mut cvar::Property(s!("fov_aim"), &mut self.config.fov_aim, &default.fov_aim));
		f(&mut cvar::Property(s!("fov_min"), &mut self.config.fov_min, &default.fov_min));
		f(&mut cvar::Property(s!("teledist"), &mut self.config.teledist, &default.teledist));
		f(&mut cvar::List(s!("pid"), &mut self.config.pid));
	}
}

impl Aimbot {
	pub fn run(&mut self, api: &mut Api, ctx: &RunContext) {
		if !self.config.enable {
			self.reset_aim();
			self.reset_target();
			return;
		}

		let state = ctx.state;

		// Run the aimbot at specified rate
		// This is necessary because the aim smoothing code cannot handle variable timedeltas
		if !self.timer.has_elapsed(state.time, 1.0 / self.config.tickrate as f64) {
			return;
		}

		// Keep track of the weapon is firing
		let ps = state.player_state();
		let mut weaponstate = ps.weaponstate;
		if ps.weaponTime == 0 {
			weaponstate = sdk::WEAPON_READY;
		}
		if weaponstate != self.weaponstate {
			self.weaponstate = weaponstate;
			if weaponstate == sdk::WEAPON_FIRING {
				self.weapon_tfiring = state.time;
			}
		}

		if !api.is_button_down(self.config.aim_key) {
			self.reset_aim();
			self.reset_target();
			return;
		}

		// If we're target_locked find a new target when sufficient time has passed
		let new_target = if self.target_locked {
			state.time > self.target_tend + self.config.aim_idletime as f64
		}
		// If we're not target_locked find a new target if we don't already have one
		else {
			state.time > self.target_tstart + self.config.aim_mintime as f64
		};

		// Find a new target if requested
		if new_target {
			self.find_target(state);
		}

		// Aim at the target if we have one
		let mut target_valid = false;
		if let Some(target) = state.get_entity(self.target_entnum) {

			let mut info = TargetInfo::default();
			if self.validate(state, target, &mut info) && (new_target || self.keep_target(&info)) {

				self.set_aim_mod(state, &info);
				self.aim(api, &info);

				// Update the target as long as it's valid
				self.target_tend = state.time;
				self.target_pos = info.origin;
				target_valid = true;

				if self.config.debug {
					self.debug(api, &info);
				}
			}
		}

		// Drop the target once no longer valid
		if !target_valid {
			self.target_entnum = -1;
			self.reset_aim();
		}
	}

	fn reset_aim(&mut self) {
		self.addx = 0.0;
		self.addy = 0.0;
		self.pidx.reset();
		self.pidy.reset();
	}

	fn reset_target(&mut self) {
		self.target_entnum = -1;
		self.target_locked = false;
		self.target_history.clear();
	}

	/// Returns if the current target has teleported.
	fn keep_target(&mut self, info: &TargetInfo) -> bool {
		// use origin because projectile prediction causes a lot of jitter
		math::dist(self.target_pos, info.origin) <= self.config.teledist
	}

	/// Dynamically adjust the aim strength down based on the situation.
	fn set_aim_mod(&mut self, state: &GameState, info: &TargetInfo) {
		let ps = state.player_state();

		let mut aim_mod = self.update_target_history(state, info);

		// Ramp up aim strength over a short period of time
		if self.config.aim_ramp > 0.0 {
			let ramp = f32::min((state.time - self.target_tstart) as f32 / self.config.aim_ramp, 1.0);
			let ramp = math::smoothstep(ramp);
			aim_mod = f32::min(aim_mod, ramp);
		}

		// Reduce aim strength after shooting
		if let Some(weapon) = state::WEAPON_INFO.get(ps.weapon as usize) {
			if weapon.cooldown > 0.0 {
				let x = (state.time - self.weapon_tfiring) as f32;
				let x = f32::min(x / weapon.cooldown, 1.0);
				let x = f32::powf(x, self.config.aim_cooldown_pow);
				aim_mod *= x;
			}
		}

		self.aim_mod = aim_mod;
	}

	/// Keep track of the target's position and velocity over time.
	///
	/// Returns a factor between 0 and 1 that represents how well the target's position can be predicted.
	fn update_target_history(&mut self, state: &GameState, info: &TargetInfo) -> f32 {
		self.target_history.push_front(TargetPos {
			time: state.time,
			origin: info.origin,
			velocity: info.velocity,
		});

		// Clear out old entries
		while let Some(back) = self.target_history.back() {
			if state.time - back.time >= self.config.aim_react as f64 {
				self.target_history.pop_back();
			}
			else {
				break;
			}
		}

		let Some(back) = self.target_history.back() else { return 1.0 };

		let dt = (state.time - back.time) as f32;

		let predicted_origin1 = math::add(back.origin, math::muls(back.velocity, dt));
		let predicted_error1 = math::dist(predicted_origin1, info.origin);

		let predicted_origin2 = math::add(math::add(back.origin, math::muls(back.velocity, dt)), math::muls([0.0, 0.0, -sdk::DEFAULT_GRAVITY], 0.5 * dt * dt));
		let predicted_error2 = math::dist(predicted_origin2, info.origin);

		// Calculate the prediction error twice for linear and quadratic prediction to account for gravity
		// Just use the minimum error as a good enough approximation
		let predicted_error = f32::min(predicted_error1, predicted_error2);
		self.target_prederror = predicted_error;

		// Measure the predictability by comparing it against the react dist config
		let react_dist = self.config.aim_react_dist.max(1.0);
		let measure = (1.0 - predicted_error / react_dist).clamp(0.0, 1.0);

		// Apply a power curve to make the reaction more of a snap response
		measure.powf(self.config.aim_react_pow)
	}

	/// Finds a new target.
	fn find_target(&mut self, state: &GameState) {
		let mut target = None;
		let mut priority = f32::MAX;
		let mut info = TargetInfo::default();
		for ent in state.entities() {
			if self.validate(state, ent, &mut info) {
				if info.angle < priority {
					priority = info.angle;
					target = Some(ent);
				}
			}
		}

		if let Some(target) = target {
			// Only switch if we change our mind about the target
			if target.number != self.target_entnum {
				self.target_entnum = target.number;
				self.target_locked = true;
				self.target_tstart = state.time;
				self.target_pos = target.origin;
				self.target_history.clear();
			}
		}
	}

	/// Checks if the target is valid.
	///
	/// Detailed information about the target is provided in `info`.
	fn validate(&self, state: &GameState, target: &sdk::entityState_t, info: &mut TargetInfo) -> bool {
		if !self.rules(state, target, info) {
			return false;
		}

		// hitpoint
		info.origin = target.pos.trBase;
		info.velocity = target.pos.trDelta;
		info.grounded = target.groundEntityNum != (sdk::MAX_GENTITIES - 1) as i32;
		info.spine = [-10.0, 30.0];

		if !self.compute(state, target, info) {
			return false;
		}

		if !self.check(state, target, info) {
			return false;
		}

		return true;
	}

	/// Decides if this is a valid target to aim at based on game rules.
	fn rules(&self, state: &GameState, target: &sdk::entityState_t, _info: &mut TargetInfo) -> bool {
		let ps = state.player_state();
		if target.clientNum == ps.clientNum {
			return false;
		}
		if target.eType != sdk::ET_PLAYER {
			return false;
		}
		if target.eFlags & sdk::EF_DEAD != 0 {
			return false;
		}
		return true;
	}

	/// Computes the aim angle to hit the target.
	fn compute(&self, state: &GameState, _target: &sdk::entityState_t, info: &mut TargetInfo) -> bool {
		if !self.solve(state, info, 0) {
			return false;
		}
		let [pitch2, yaw2, _] = info.aim;
		let time2 = info.ptime;
		let hit2 = info.hit;

		if !self.solve(state, info, 1) {
			return false;
		}
		let [pitch1, yaw1, _] = info.aim;
		let time1 = info.ptime;
		let hit1 = info.hit;

		let va = state.view_angles();

		// If aiming above spine[0] aim at spine[0]
		if va[0] < pitch1 || pitch2 - pitch1 < 0.1 {
			info.ptime = time1;
			info.hit = hit1;
			info.aim = [pitch1, yaw1, 0.0];
		}
		// If aiming below spine[1] aim at spine[1]
		else if va[0] > pitch2 {
			info.ptime = time2;
			info.hit = hit2;
			info.aim = [pitch2, yaw2, 0.0];
		}
		// Otherwise interpolate between the spine endpoints
		else {
			let ratio = (va[0] - pitch1) / (pitch2 - pitch1);
			info.ptime = time1 + (time2 - time1) * ratio;
			info.hit = math::lerp(hit1, hit2, ratio);
			let mut dyaw = yaw2 - yaw1;
			if dyaw > 180.0 { dyaw -= 360.0; }
			else if dyaw < -180.0 { dyaw += 360.0; }
			info.aim = [va[0], yaw1 + dyaw * ratio, 0.0];
		}

		return true;
	}

	/// Checks if the target can be aimed at with the current weapon.
	fn solve(&self, state: &GameState, info: &mut TargetInfo, index: usize) -> bool {
		let ps = state.player_state();

		let mut shoot_pos = ps.origin;
		shoot_pos[2] += ps.viewheight as f32;
		info.distance = math::dist(shoot_pos, info.origin);

		let Some(&offset) = info.spine.get(index) else { return false };
		let origin = math::add(info.origin, [0.0, 0.0, offset]);

		let Some(weapon) = state::WEAPON_INFO.get(ps.weapon as usize) else { return false };

		if weapon.projectile_speed > 0.0 {
			let linear_predictor: solver::LinearPredictor;
			let gravity_predictor: solver::GravityPredictor;

			let predictor = if !info.grounded {
				gravity_predictor = solver::GravityPredictor {
					origin,
					velocity: info.velocity,
					gravity: sdk::DEFAULT_GRAVITY,
				};
				&gravity_predictor as &dyn solver::TargetPredictor
			}
			else {
				linear_predictor = solver::LinearPredictor {
					origin,
					velocity: info.velocity,
				};
				&linear_predictor as &dyn solver::TargetPredictor
			};

			// info.ptime = info.distance / pspeed;
			// info.hit = math::add(info.origin, math::muls(info.velocity, info.ptime));
			let Some(sol) = solver::solve(&shoot_pos, weapon, predictor) else { return false };
			info.ptime = sol.time;
			info.hit = predictor.predict_position(sol.time);
			info.aim = [-sol.pitch.to_degrees(), sol.yaw.to_degrees(), 0.0];
		}
		else {
			info.ptime = 0.0;
			info.hit = info.origin;
			info.aim = math::qnorm(math::qangle(math::sub(math::add(info.hit, [0.0, 0.0, offset]), shoot_pos)));
		}

		return true;
	}

	/// Checks if the target is within the configured field of view.
	fn check(&self, state: &GameState, target: &sdk::entityState_t, info: &mut TargetInfo) -> bool {
		let va = state.view_angles();

		info.pitch = info.aim[0] - va[0];
		info.yaw = info.aim[1] - va[1];
		while info.yaw > 180.0 { info.yaw -= 360.0; }
		while info.yaw <= -180.0 { info.yaw += 360.0; }
		info.angle = (info.yaw * info.yaw + info.pitch * info.pitch).sqrt();

		// Ignore FOV check when aiming at the current target
		if self.target_entnum == target.number {
			return true;
		}

		let fov_aim = self.config.fov_aim;
		let fov = if info.distance < 10.0 { fov_aim }
		else { f32::min(fov_aim, f32::atan(self.config.fov_radius / info.distance).to_degrees()) };

		return info.angle < fov;
	}

	fn aim(&mut self, api: &mut Api, info: &TargetInfo) {
		let (dx, dy) = {
			// Avoid aim jitter with a minimum angle
			let (mut yaw, mut pitch) = (info.yaw, info.pitch);
			if info.angle < self.config.fov_min {
				yaw = 0.0;
				pitch = 0.0;
			}

			// Modulate the aim strength
			let strength = self.aim_mod;
			yaw *= strength * self.config.aim_yaw;
			pitch *= strength * self.config.aim_pitch;

			let dt = 1.0 / self.config.tickrate;
			let dx = -self.pidx.step(yaw, dt, &self.config.pid);
			let dy = self.pidy.step(pitch, dt, &self.config.pid);

			(dx, dy)
		};
		let dx = dx + self.addx;
		let dy = dy + self.addy;
		let mdx = dx as i32;
		let mdy = dy as i32;
		self.addx = dx - mdx as f32;
		self.addy = dy - mdy as f32;
		if mdx != 0 || mdy != 0 {
			api.mouse_move(mdx, mdy);
		}
	}

	fn debug(&self, api: &mut Api, info: &TargetInfo) {
		api.visualize(s!("TargetInfo"), format_xml::xfmt! {
			<pre>
			"origin:   "{info.origin:?}"\n"
			"velocity: "{info.velocity:?}"\n"
			"grounded: "{info.grounded}"\n"
			"spine:    "{info.spine:?}"\n"
			"distance: "{info.distance}"\n"
			"\n"
			"hit:      "{info.hit:?}"\n"
			"aim:      "{info.aim:?}"\n"
			"ptime:    "{info.ptime}"\n"
			"\n"
			"pitch:    "{info.pitch}"\n"
			"yaw:      "{info.yaw}"\n"
			"angle:    "{info.angle}"\n"
			"\n"
			"target_entnum: "{self.target_entnum}"\n"
			"target_locked: "{self.target_locked}"\n"
			"target_tstart: "{self.target_tstart}"\n"
			"target_tend:   "{self.target_tend}"\n"
			"target_pos:    "{self.target_pos:?}"\n"
			"\n"
			"prederror: "{self.target_prederror}"\n"
			"aim_mod:   "{self.aim_mod}"\n"
			</pre>
		});
	}
}
