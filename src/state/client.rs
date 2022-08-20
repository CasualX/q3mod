use super::*;

#[derive(Default)]
pub struct Client {
	// ClientActive
	pub server_time: i32,
	pub mapname: String,
	pub viewangles: [f32; 3],
	pub serverid: i32,

	// ClientStatic
	pub state_old: i32,
	pub state: i32,
	pub framecount: i32,
	pub frametime: i32,
	pub realtime: i32,
	pub realframetime: i32,
}

impl Client {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
		let mut indices = [0, 4, 8, 12];
		if let Ok(fields) = api.vm_gatherd(ctx.process.base.field(ctx.data.cl_viewangles), 16, &mut indices) {
			self.viewangles[0] = f32::from_bits(fields[0]);
			self.viewangles[1] = f32::from_bits(fields[1]);
			self.viewangles[2] = f32::from_bits(fields[2]);
			self.serverid = fields[3] as i32;
		}

		let a = ctx.data.cls_framecount - ctx.data.cls_state;
		let mut indices = [0, a, a + 4, a + 8, a + 12];
		if let Ok(&fields) = api.vm_gatherd(ctx.process.base.field(ctx.data.cls_state), a + 16, &mut indices) {
			self.state_old = self.state;
			self.state = fields[0] as i32;
			self.framecount = fields[1] as i32;
			self.frametime = fields[2] as i32;
			self.realtime = fields[3] as i32;
			self.realframetime = fields[4] as i32;
		}
	}
}

impl GameState {
	pub fn state_changed(&self) -> Option<i32> {
		if self.client.state == self.client.state_old {
			return None;
		}
		Some(self.client.state)
	}
}
