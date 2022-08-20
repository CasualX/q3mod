use super::*;

#[derive(Default)]
pub struct ClSnapshot {
	pub ptr: sdk::Ptr<sdk::clSnapshot_t>,
	pub old_parse_ents_num: i32,
	pub raw: sdk::clSnapshot_t,
}

impl ClSnapshot {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
		self.ptr = ctx.process.base.field(ctx.data.cl_snap);

		self.old_parse_ents_num = self.raw.parseEntitiesNum;
		let _ = api.vm_read_into(self.ptr, &mut self.raw);

		if !ctx.data.cl_snap_has_areabytes {
			dataview::bytes_mut(&mut self.raw).copy_within(6 * 4..std::mem::size_of::<sdk::clSnapshot_t>() - 4, 7 * 4);
		}
	}
	pub fn is_parse_entities_changed(&self) -> bool {
		self.old_parse_ents_num != self.raw.parseEntitiesNum
	}
	pub fn is_valid(&self) -> bool {
		self.raw.valid != 0
	}
}

impl GameState {
	pub fn player_state(&self) -> &sdk::playerState_t {
		&self.snap.raw.ps
	}
	pub fn view_angles(&self) -> [f32; 3] {
		let ref ps = self.snap.raw.ps;
		math::qnorm([
			self.client.viewangles[0] + (ps.delta_angles[0] as f32 * (360.0 / 65536.0)),
			self.client.viewangles[1] + (ps.delta_angles[1] as f32 * (360.0 / 65536.0)),
			self.client.viewangles[2] + (ps.delta_angles[2] as f32 * (360.0 / 65536.0)),
		])
	}
}
