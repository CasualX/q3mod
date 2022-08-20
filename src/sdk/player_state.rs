use super::*;

#[derive(Debug, Default, Pod)]
#[repr(C)]
pub struct playerState_t {
	pub commandTime: int,
	pub pm_type: int,
	pub bobCycle: int,
	pub pm_flags: int,
	pub pm_time: int,

	pub origin: vec3_t,
	pub velocity: vec3_t,
	pub weaponTime: int,
	pub gravity: int,
	pub speed: int,
	pub delta_angles: [int; 3],

	pub groundEntityNum: int,

	pub legsTimer: int,
	pub legsAnim: int,

	pub torsoTimer: int,
	pub torsoAnim: int,

	pub movementDir: int,

	pub grapplePoint: vec3_t,

	pub eFlags: int,

	pub eventSequence: int,
	pub events: [int; MAX_PS_EVENTS],
	pub eventParms: [int; MAX_PS_EVENTS],

	pub externalEvent: int,
	pub externalEventParm: int,
	pub externalEventTime: int,

	pub clientNum: int,
	pub weapon: int,
	pub weaponstate: int,

	pub viewangles: vec3_t,
	pub viewheight: int,

	pub damageEvent: int,
	pub damageYaw: int,
	pub damagePitch: int,
	pub damageCount: int,

	pub stats: [int; MAX_STATS],
	pub persistant: [int; MAX_PERSISTANT],
	pub powerups: [int; MAX_POWERUPS],
	pub ammo: [int; MAX_WEAPONS],

	pub generic1: int,
	pub loopSound: int,
	pub jumppad_ent: int,

	pub ping: int,
	pub pmove_framecount: int,
	pub jumppad_frame: int,
	pub entityEventSequence: int,
}
