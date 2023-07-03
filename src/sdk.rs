#![allow(non_camel_case_types, non_snake_case, dead_code)]

pub use intptr::IntPtr64 as Ptr;
pub use dataview::Pod;
use crate::math;

pub type int = i32;
pub type vec3_t = [f32; 3];
pub type qboolean = i32;
pub type qhandle_t = i32;
pub const MAX_PS_EVENTS: usize = 2;
pub const MAX_STATS: usize = 16;
pub const MAX_PERSISTANT: usize = 16;
pub const MAX_POWERUPS: usize = 16;
pub const MAX_WEAPONS: usize = 16;
pub const MAX_CLIENTS: usize = 64;
pub const MAX_LOCATIONS: usize = 64;
pub const MAX_MAP_AREA_BYTES: usize = 32;
pub const PACKET_BACKUP: usize = 32;
pub const MAX_SNAPSHOT_ENTITIES: usize = 256;
pub const MAX_REFENTITIES: usize = (1 << 12) - 1;

pub const GENTITYNUM_BITS: usize = 10;
pub const MAX_GENTITIES: usize = 1 << GENTITYNUM_BITS;

pub const CA_UNINITIALIZED: i32 = 0;
pub const CA_DISCONNECTED: i32 = 1;  // not talking to a server
pub const CA_AUTHORIZING: i32 = 2;   // not used any more, was checking cd key
pub const CA_CONNECTING: i32 = 3;    // sending request packets to the server
pub const CA_CHALLENGING: i32 = 4;   // sending challenge packets to the server
pub const CA_CONNECTED: i32 = 5;     // netchan_t established, getting gamestate
pub const CA_LOADING: i32 = 6;       // only during cgame initialization, never during main loop
pub const CA_PRIMED: i32 = 7;        // got gamestate, waiting for first frame
pub const CA_ACTIVE: i32 = 8;        // game views should be displayed
pub const CA_CINEMATIC: i32 = 9;     // playing a cinematic or a static pic, not connected to a server

pub const DEFAULT_GRAVITY: f32 = 800.0;

// pmtype_t
pub const PM_NORMAL: i32 = 0; // can accelerate and turn
pub const PM_NOCLIP: i32 = 1; // noclip movement
pub const PM_SPECTATOR: i32 = 2; // still run into walls
pub const PM_DEAD: i32 = 3; // no acceleration or turning, but free falling
pub const PM_FREEZE: i32 = 4; // stuck in place with no control
pub const PM_INTERMISSION: i32 = 5; // no movement or status bar
pub const PM_SPINTERMISSION: i32 = 6; // no movement or status bar

pub const WEAPON_READY: i32 = 0;
pub const WEAPON_RAISING: i32 = 1;
pub const WEAPON_DROPPING: i32 = 2;
pub const WEAPON_FIRING: i32 = 3;

// pmove->pm_flags
pub const PMF_DUCKED: i32 = 1;
pub const PMF_JUMP_HELD: i32 = 2;
pub const PMF_BACKWARDS_JUMP: i32 = 8; // go into backwards land
pub const PMF_BACKWARDS_RUN: i32 = 16; // coast down to backwards run
pub const PMF_TIME_LAND: i32 = 32; // pm_time is time before rejump
pub const PMF_TIME_KNOCKBACK: i32 = 64; // pm_time is an air-accelerate only time
pub const PMF_TIME_WATERJUMP: i32 = 256; // pm_time is waterjump
pub const PMF_RESPAWNED: i32 = 512; // clear after attack and jump buttons come up
pub const PMF_USE_ITEM_HELD: i32 = 1024;
pub const PMF_GRAPPLE_PULL: i32 = 2048; // pull towards grapple location
pub const PMF_FOLLOW: i32 = 4096; // spectate following another player
pub const PMF_SCOREBOARD: i32 = 8192; // spectate as a scoreboard
pub const PMF_INVULEXPAND: i32 = 16384; // invulnerability sphere set to full size

pub const PW_NONE: i32 = 0;
pub const PW_QUAD: i32 = 1;
pub const PW_BATTLESUIT: i32 = 2;
pub const PW_HASTE: i32 = 3;
pub const PW_INVIS: i32 = 4;
pub const PW_REGEN: i32 = 5;
pub const PW_FLIGHT: i32 = 6;
pub const PW_REDFLAG: i32 = 7;
pub const PW_BLUEFLAG: i32 = 8;
pub const PW_NEUTRALFLAG: i32 = 9;
pub const PW_SCOUT: i32 = 10;
pub const PW_GUARD: i32 = 11;
pub const PW_DOUBLER: i32 = 12;
pub const PW_AMMOREGEN: i32 = 13;
pub const PW_INVULNERABILITY: i32 = 14;

pub const HI_NONE: i32 = 0;
pub const HI_TELEPORTER: i32 = 1;
pub const HI_MEDKIT: i32 = 2;
pub const HI_KAMIKAZE: i32 = 3;
pub const HI_PORTAL: i32 = 4;
pub const HI_INVULNERABILITY: i32 = 5;

pub const WP_NONE: i32 = 0;
pub const WP_GAUNTLET: i32 = 1;
pub const WP_MACHINEGUN: i32 = 2;
pub const WP_SHOTGUN: i32 = 3;
pub const WP_GRENADE_LAUNCHER: i32 = 4;
pub const WP_ROCKET_LAUNCHER: i32 = 5;
pub const WP_LIGHTNING: i32 = 6;
pub const WP_RAILGUN: i32 = 7;
pub const WP_PLASMAGUN: i32 = 8;
pub const WP_BFG: i32 = 9;
pub const WP_GRAPPLING_HOOK: i32 = 10;
pub const WP_NAILGUN: i32 = 11;
pub const WP_PROX_LAUNCHER: i32 = 12;
pub const WP_CHAINGUN: i32 = 13;

// entityState_t->eFlags
pub const EF_DEAD: i32 =             0x00000001; // don't draw a foe marker over players with EF_DEAD
pub const EF_TICKING: i32 =          0x00000002; // used to make players play the prox mine ticking sound
pub const EF_TELEPORT_BIT: i32 =     0x00000004; // toggled every time the origin abruptly changes
pub const EF_AWARD_EXCELLENT: i32 =  0x00000008; // draw an excellent sprite
pub const EF_PLAYER_EVENT: i32 =     0x00000010;
pub const EF_BOUNCE: i32 =           0x00000010; // for missiles
pub const EF_BOUNCE_HALF: i32 =      0x00000020; // for missiles
pub const EF_AWARD_GAUNTLET: i32 =   0x00000040; // draw a gauntlet sprite
pub const EF_NODRAW: i32 =           0x00000080; // may have an event, but no model (unspawned items)
pub const EF_FIRING: i32 =           0x00000100; // for lightning gun
pub const EF_KAMIKAZE: i32 =         0x00000200;
pub const EF_MOVER_STOP: i32 =       0x00000400; // will push otherwise
pub const EF_AWARD_CAP: i32 =        0x00000800; // draw the capture sprite
pub const EF_TALK: i32 =             0x00001000; // draw a talk balloon
pub const EF_CONNECTION: i32 =       0x00002000; // draw a connection trouble sprite
pub const EF_VOTED: i32 =            0x00004000; // already cast a vote
pub const EF_AWARD_IMPRESSIVE: i32 = 0x00008000; // draw an impressive sprite
pub const EF_AWARD_DEFEND: i32 =     0x00010000; // draw a defend sprite
pub const EF_AWARD_ASSIST: i32 =     0x00020000; // draw a assist sprite
pub const EF_AWARD_DENIED: i32 =     0x00040000; // denied
pub const EF_TEAMVOTED: i32 =        0x00080000; // already cast a team vote

// entityState_t->eType
pub const ET_GENERAL: i32 = 0;
pub const ET_PLAYER: i32 = 1;
pub const ET_ITEM: i32 = 2;
pub const ET_MISSILE: i32 = 3;
pub const ET_MOVER: i32 = 4;
pub const ET_BEAM: i32 = 5;
pub const ET_PORTAL: i32 = 6;
pub const ET_SPEAKER: i32 = 7;
pub const ET_PUSH_TRIGGER: i32 = 8;
pub const ET_TELEPORT_TRIGGER: i32 = 9;
pub const ET_INVISIBLE: i32 = 10;
pub const ET_GRAPPLE: i32 = 11;
pub const ET_TEAM: i32 = 12;
pub const ET_EVENTS: i32 = 13;

pub const TR_STATIONARY: i32 = 0;
pub const TR_INTERPOLATE: i32 = 1;
pub const TR_LINEAR: i32 = 2;
pub const TR_LINEAR_STOP: i32 = 3;
pub const TR_SINE: i32 = 4;
pub const TR_GRAVITY: i32 = 5;

#[derive(Copy, Clone, Default, Pod)]
#[repr(C)]
pub struct trRefdef_t {
	pub x: int,
	pub y: int,
	pub width: int,
	pub height: int,
	pub fov_x: f32,
	pub fov_y: f32,
	pub vieworg: vec3_t,
	pub viewaxis: [vec3_t; 3],
}

#[derive(Copy, Clone, Default, Pod)]
#[repr(C)]
pub struct trajectory_t {
	pub trType: int,
	pub trTime: int,
	pub trDuration: int,
	pub trBase: vec3_t,
	pub trDelta: vec3_t,
}

pub fn evaluate_trajectory(tr: &trajectory_t, time: int) -> vec3_t {
	match tr.trType {
		TR_STATIONARY | TR_INTERPOLATE => {
			tr.trBase
		},
		TR_LINEAR => {
			let dt = (time - tr.trTime) as f32 * 0.001;
			math::add(tr.trBase, math::muls(tr.trDelta, dt))
		},
		TR_LINEAR_STOP => {
			let dt = int::clamp(time - tr.trTime, 0, tr.trDuration) as f32 * 0.001;
			math::add(tr.trBase, math::muls(tr.trDelta, dt))
		},
		TR_GRAVITY => {
			let dt = (time - tr.trTime) as f32 * 0.001;
			let mut result = math::add(tr.trBase, math::muls(tr.trDelta, dt));
			result[2] -= 0.5 * DEFAULT_GRAVITY * dt * dt;
			result
		},
		_ => [0.0; 3],
	}
}

pub const RF_MINLIGHT: i32 = 0x0001;
pub const RF_THIRD_PERSON: i32 = 0x0002;
pub const RF_FIRST_PERSON: i32 = 0x0004;
pub const RF_DEPTHHACK: i32 = 0x0008;
pub const RF_CROSSHAIR: i32 = 0x0010;
pub const RF_NOSHADOW: i32 = 0x0040;
pub const RF_LIGHTING_ORIGIN: i32 = 0x0080;
pub const RF_SHADOW_PLANE: i32 = 0x0100;
pub const RF_WRAP_FRAMES: i32 = 0x0200;

pub const RT_MODEL: i32 = 0;
pub const RT_POLY: i32 = 1;
pub const RT_SPRITE: i32 = 2;
pub const RT_BEAM: i32 = 3;
pub const RT_RAIL_CORE: i32 = 4;
pub const RT_RAIL_RINGS: i32 = 5;
pub const RT_LIGHTNING: i32 = 6;
pub const RT_PORTALSURFACE: i32 = 7;

#[derive(Copy, Clone, Debug, Default, Pod)]
#[repr(C)]
pub struct refEntity_t {
	pub reType: i32,
	pub renderfx: int,
	pub hModel: qhandle_t,
	pub lightingOrigin: vec3_t,
	pub shadowPlane: f32,
	pub axis: [vec3_t; 3],
	pub nonNormalizedAxes: qboolean,
	pub origin: [f32; 3],
	pub frame: int,
	pub oldorigin: [f32; 3],
	pub oldframe: int,
	pub backlerp: f32,
	pub skinNum: int,
	pub customSkin: qhandle_t,
	pub customShader: qhandle_t,
	pub shader: u32,
	pub shaderTexCoord: [f32; 2],
	pub shaderTime: f32,
	pub radius: f32,
	pub rotation: f32,
}


mod player_state;
pub use self::player_state::playerState_t;

mod entity_state;
pub use self::entity_state::entityState_t;

mod snapshot;
pub use self::snapshot::clSnapshot_t;
