use super::*;

#[derive(Copy, Clone, Default, Pod)]
#[repr(C)]
pub struct entityState_t {
	pub number: int,
	pub eType: int,
	pub eFlags: int,

	pub pos: trajectory_t,
	pub apos: trajectory_t,

	pub time: int,
	pub time2: int,

	pub origin: vec3_t,
	pub origin2: vec3_t,

	pub angles: vec3_t,
	pub angles2: vec3_t,

	pub otherEntityNum: int,
	pub otherEntityNum2: int,

	pub groundEntityNum: int,

	pub constantLight: int,
	pub loopSound: int,

	pub modelindex: int,
	pub modelindex2: int,
	pub clientNum: int,
	pub frame: int,

	pub solid: int,

	pub event: int,
	pub eventParm: int,

	pub powerups: int,
	pub weapon: int,
	pub legsAnim: int,
	pub torsoAnim: int,

	pub generic1: int,
}
