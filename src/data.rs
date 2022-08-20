#![allow(non_snake_case)]

#[derive(Copy, Clone, Default)]
pub struct GameData {
	pub TimeDateStamp: u32,

	// RE_RenderScene:
	// 488D15${"R_RenderScene: NULL worldmodel"} [1-150] 89(05|35)$'
	pub tr_refdef: u32,

	// R_GetModelByHandle, RE_RegisterModel:
	// 4883F840 72% 448B1D$'
	pub tr_models: u32,
	pub tr_maxmodels: u32,

	// cnq3's clSnapshot_t does not have field maxareabytes
	pub cl_snap_has_areabytes: bool,

	// CL_SetCGameTime:
	// cnq3:    393D${'} 0F84$ 488D15$ "CL_SetCGameTime:"
	// quake3e: 833D${?'}00 BB01000000 75? 488D15$ "CL_SetCGameTime:"
	pub cl_snap: u32,

	// CL_ParseSnapshot:
	// cl.snap.ping = 999;
	// 8B?${'} C705????E7030000
	pub cl_snap_ps: u32,

	// CL_MouseMove:
	// "ratex: %f", at the end of the function
	// F30F1005${'} 488B????? F30F59?? F30F58
	pub cl_viewangles: u32,

	// CL_GetSnapshot:
	// 488D0D${"CL_GetSnapshot: truncated"}
	pub cl_parseEntities: u32,
	pub max_parse_entities: u32,

	// CL_MapLoading:
	// "nextmap" and "localhost"
	pub cls_state: u32, // 488D15${"localhost"} [1-20] C705${????'}04000000
	pub cls_framecount: u32,

	pub backEndData: u32, // "RE_AddRefEntityToScene"
	pub r_entities: u32,
	pub r_numentities: u32,
	pub sizeof_refEntity_t: u32,
}

pub static OFFSETS: [GameData; 2] = [
	// cnq3-x64.exe
	GameData {
		TimeDateStamp: 0x5ed556e0,

		tr_refdef: 0x398b60,
		tr_models: 0x398DA0,
		tr_maxmodels: 1024,

		cl_snap: 0x2636f4,
		cl_snap_ps: 0x263730,
		cl_snap_has_areabytes: false,
		cl_viewangles: 0x268fb0,
		cl_parseEntities: 0x2A1340,
		max_parse_entities: 2048,

		cls_state: 0x193c70,
		cls_framecount: 0x193c70 + 288,

		backEndData: 0x5891C0,
		r_entities: 0x300800,
		r_numentities: 0xC1C6AC,
		sizeof_refEntity_t: 0xC0,
	},

	// quake3e.x64.exe
	GameData {
		TimeDateStamp: 0x62cacf9a,

		tr_refdef: 0xFA3468,
		tr_models: 0xFA36B8,
		tr_maxmodels: 1024,

		cl_snap: 0xD7FBC4 - 16*4,
		cl_snap_ps: 0xD7FBC4,
		cl_snap_has_areabytes: true,
		cl_viewangles: 0xD8543C,
		cl_parseEntities: 0xDBD84C,
		max_parse_entities: 8192,

		cls_state: 0xc830c0,
		cls_framecount: 0xc830c0 + 288,

		backEndData: 0xFF0B28,
		r_entities: 0x501A00,
		r_numentities: 0x8ACC3C,
		sizeof_refEntity_t: 0xD0,
	},
];
