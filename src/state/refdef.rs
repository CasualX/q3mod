use super::*;

#[repr(C)]
#[derive(Default)]
pub struct RefDef {
	pub raw: sdk::trRefdef_t,
}

impl RefDef {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
		if ctx.data.tr_refdef == 0 {
			return;
		}

		if let Ok(raw) = api.vm_read::<sdk::trRefdef_t>(ctx.process.base.field(ctx.data.tr_refdef)) {
			if raw.vieworg != [0.0; 3] {
				self.raw = raw;
			}
		}
	}
}

impl GameState {
	pub fn world_to_screen(&self, origin: [f32; 3], _clip: bool) -> Option<[f32; 2]> {
		let ref refdef = self.refdef.raw;

		let local = math::sub(origin, refdef.vieworg);
		let tx = math::dot(local, refdef.viewaxis[1]);
		let ty = math::dot(local, refdef.viewaxis[2]);
		let tz = math::dot(local, refdef.viewaxis[0]);
		if tz < 0.01 {
			return None;
		}
		let thfov_x = (refdef.fov_x.to_radians() * 0.5).tan();
		let thfov_y = (refdef.fov_y.to_radians() * 0.5).tan();
		let vx = 1.0 - (tx / tz / thfov_x);
		let vy = 1.0 - (ty / tz / thfov_y);
		// If the resulting coordinate is too far outside the screen bounds clip it manually
		// if clip && (vx < -1.0 || vx > 3.0 || vy < -1.0 || vy > 3.0) {
		// 	return None;
		// }
		let px = (refdef.width as f32 * 0.5) * vx;
		let py = (refdef.height as f32 * 0.5) * vy;
		Some([px, py])
	}
}
