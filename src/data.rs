#![allow(non_snake_case)]

use crate::Api;
use crate::base::{hash, parse_u32};

#[derive(Copy, Clone, Debug, Default)]
pub struct GameData {
	pub TimeDateStamp: u32,

	// RE_RenderScene:
	// 488D15${"R_RenderScene: NULL worldmodel"} [1-150] 89(05|35)$'
	pub tr_refdef: u32,

	// R_GetModelByHandle, RE_RegisterModel:
	// 4883F840 72% 448B1D$'
	pub tr_models: u32,
	pub tr_numModels: u32,
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

	// SV_GameClientNum
	// E8${ 8B05${'} 0FAFC1 4898 480305${'} C3 } 488D0D$"%5i "
	pub sv_gameClients: u32,
}

impl GameData {
	pub fn load(&mut self, api: &mut Api, parser: &mut ini_core::Parser) {
		use std::collections::HashMap;
		use ini_core::*;
		trait Parse {
			fn parse(&mut self, api: &mut Api, value: &str);
		}
		impl Parse for u32 {
			fn parse(&mut self, _api: &mut Api, value: &str) {
				*self = parse_u32(value);
			}
		}
		impl Parse for bool {
			fn parse(&mut self, api: &mut Api, value: &str) {
				if hash(value) == hash!("true") {
					*self = true;
				}
				else if hash(value) == hash!("false") {
					*self = false;
				}
				else {
					api.log(fmtools::fmt!("GameData::load: Invalid bool value: "{value}));
				}
			}
		}
		let mut map = HashMap::new();
		macro_rules! build_map {
			($($lit:literal => $field:ident,)*) => {
				$(
					let $field = hash!($lit);
					map.insert($field, &mut self.$field as &mut dyn Parse);
				)*
			};
		}
		build_map! {
			"TimeDateStamp" => TimeDateStamp,
			"tr.refdef" => tr_refdef,
			"tr.models" => tr_models,
			"tr.numModels" => tr_numModels,
			"tr.maxmodels" => tr_maxmodels,
			"cl.snap" => cl_snap,
			"cl.snap.ps" => cl_snap_ps,
			"cl.snap.has_area_bytes" => cl_snap_has_areabytes,
			"cl.viewangles" => cl_viewangles,
			"cl.parseEntities" => cl_parseEntities,
			"MAX_PARSE_ENTITIES" => max_parse_entities,
			"cls.state" => cls_state,
			"cls.framecount" => cls_framecount,
			"backEndData" => backEndData,
			"r_entities" => r_entities,
			"r_numentities" => r_numentities,
			"sizeof refEntity_t" => sizeof_refEntity_t,
			"sv.gameClients" => sv_gameClients,
		}

		for line in parser {
			match line {
				Item::SectionEnd => {
					return;
				}
				Item::Property(key, Some(value)) => {
					let k = hash(key);
					if let Some(field) = map.get_mut(&k) {
						field.parse(api, value);
					}
					else {
						api.log(fmtools::fmt!("GameData::load: Unknown key: "{key}));
					}
				}
				_ => {}
			}
		}
	}
}
