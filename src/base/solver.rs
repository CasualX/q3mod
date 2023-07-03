/*!
Projectile solver.
!*/

pub trait TargetPredictor {
	fn predict_position(&self, time: f32) -> [f32; 3];
}
pub trait ProjectileWeapon {
	fn projectile_speed(&self) -> f32;
	fn projectile_gravity(&self) -> f32;
	fn projectile_fire_setup(&self, origin: &[f32; 3], target: &[f32; 3]) -> [f32; 3] {
		let dx = target[0] - origin[0];
		let dy = target[1] - origin[1];
		let dz = target[2] - origin[2];
		[dx, dy, dz]
	}
}

/// Simple linear extrapolation.
pub struct LinearPredictor {
	pub origin: [f32; 3],
	pub velocity: [f32; 3],
}
impl TargetPredictor for LinearPredictor {
	fn predict_position(&self, time: f32) -> [f32; 3] {
		let x = self.origin[0] + self.velocity[0] * time;
		let y = self.origin[1] + self.velocity[1] * time;
		let z = self.origin[2] + self.velocity[2] * time;
		[x, y, z]
	}
}

pub struct GravityPredictor {
	pub origin: [f32; 3],
	pub velocity: [f32; 3],
	pub gravity: f32,
}
impl TargetPredictor for GravityPredictor {
	fn predict_position(&self, time: f32) -> [f32; 3] {
		let x = self.origin[0] + self.velocity[0] * time;
		let y = self.origin[1] + self.velocity[1] * time;
		let z = self.origin[2] + self.velocity[2] * time - 0.5 * self.gravity * time * time;
		[x, y, z]
	}
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Solution {
	pub pitch: f32,
	pub yaw: f32,
	pub time: f32,
}

const MAX_TIME: f32 = 1.5;
const TIME_STEP: f32 = 1.0 / 256.0;

pub fn solve(player: &[f32; 3], weapon: &dyn ProjectileWeapon, target: &dyn TargetPredictor) -> Option<Solution> {
	let mut target_time = 0.0;
	while target_time <= MAX_TIME {
		let target_pos = target.predict_position(target_time);
		let sol = solve2d(player, weapon, &target_pos)?;
		if sol.time < target_time {
			return Some(sol);
		}
		target_time += TIME_STEP;
	}
	None
}
fn solve2d(player: &[f32; 3], weapon: &dyn ProjectileWeapon, target: &[f32; 3]) -> Option<Solution> {
	let v0 = weapon.projectile_speed();
	let g = weapon.projectile_gravity();
	let v = weapon.projectile_fire_setup(player, target);
	let dx = f32::sqrt(v[0] * v[0] + v[1] * v[1]);
	let dy = v[2];
	if g == 0.0 {
		let pitch = f32::atan2(dy, dx);
		let time = f32::sqrt(dx * dx + dy * dy) / v0;
		let yaw = f32::atan2(v[1], v[0]);
		Some(Solution { pitch, yaw, time })
	}
	else {
		let pitch = optimal(dx, dy, v0, g)?;
		let time = dx / (f32::cos(pitch) * v0);
		let yaw = f32::atan2(v[1], v[0]);
		Some(Solution { pitch, yaw, time })
	}
}
fn optimal(x: f32, y: f32, v0: f32, g: f32) -> Option<f32> {
	let root = v0 * v0 * v0 * v0 - g * (g * x * x + 2.0 * y * v0 * v0);
	if root < 0.0 {
		return None;
	}
	let root = f32::sqrt(root);
	let angle = f32::atan((v0 * v0 - root) / (g * x));
	Some(angle)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn units() {
		struct Player([f32; 3]);
		struct Weapon(f32, f32);
		impl ProjectileWeapon for Weapon {
			fn projectile_speed(&self) -> f32 { self.0 }
			fn projectile_gravity(&self) -> f32 { self.1 }
		}
		impl TargetPredictor for Player {
			fn predict_position(&self, _time: f32) -> [f32; 3] { self.0 }
		}

		let target = Player([1000.0, 2000.0, 0.0]);
		let player = Player([500.0, 100.0, 100.0]);
		let weapon = Weapon(10000.0, 750.0);

		let sol = solve(&player.0, &weapon, &target);
		panic!("{:?}", sol);
	}
}
