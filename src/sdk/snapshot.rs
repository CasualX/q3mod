use super::*;

#[derive(Debug, Default, Pod)]
#[repr(C)]
pub struct clSnapshot_t {
	pub valid: qboolean,
	pub snapFlags: int,

	pub serverTime: int,

	pub messageNum: int,
	pub deltaNum: int,
	pub ping: int,
	pub areabytes: int,
	pub areamask: [u32; MAX_MAP_AREA_BYTES / 4],

	pub cmdNum: int,
	pub ps: playerState_t,

	pub numEntities: int,
	pub parseEntitiesNum: int,

	pub serverCommandNum: int,
}
