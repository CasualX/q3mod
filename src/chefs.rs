use super::*;

mod debugger;
mod esp;
mod wallhack;
mod aimbot;

pub use self::wallhack::Wallhack;

pub struct RunContext<'a> {
	pub process: &'a GameProcess,
	pub data: &'a GameData,
	pub state: &'a GameState,
	pub screen: [i32; 2],
}

impl<'a> RunContext<'a> {
	pub fn world_to_screen(&self, origin: [f32; 3], clip: bool) -> Option<[f32; 2]> {
		self.state.world_to_screen(origin, clip)
	}

	pub fn set_view_angles(&self, api: &mut Api, mut va: [f32; 3]) {
		let ps = self.state.player_state();
		va[0] -= ps.delta_angles[0] as f32 * (360.0 / 65536.0);
		va[1] -= ps.delta_angles[1] as f32 * (360.0 / 65536.0);
		va[2] -= ps.delta_angles[2] as f32 * (360.0 / 65536.0);
		let _ = api.vm_write(self.process.base.field(self.data.cl_viewangles), &va);
	}
}

#[derive(Default)]
pub struct CheatManager {
	pub debugger: debugger::Debugger,
	pub wallhack: wallhack::Wallhack,
	pub esp: esp::ESP,
	pub aimbot: aimbot::Aimbot,
}

impl cvar::IVisit for CheatManager {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		f(&mut cvar::List(s!("dbg"), &mut self.debugger));
		f(&mut cvar::List(s!("wh"), &mut self.wallhack));
		f(&mut cvar::List(s!("esp"), &mut self.esp));
		f(&mut cvar::List(s!("aim"), &mut self.aimbot));
	}
}

impl CheatManager {
	pub fn run(&mut self, api: &mut Api, ctx: &mut RunContext) {
		self.debugger.run(api, ctx);
		self.wallhack.run(api, ctx);
		self.aimbot.run(api, ctx);

		if api.r_begin(&mut ctx.screen) {
			let rd = &ctx.state.refdef.raw;
			if rd.width != 0 && rd.height != 0 {
				ctx.screen[0] = rd.width;
				ctx.screen[1] = rd.height;
			}
			api.r_rect(0.0, 0.0, ctx.screen[0] as f32, ctx.screen[1] as f32, vgc::sRGBA::TRANSPARENT, vgc::sRGBA!(Aqua));
			self.esp.run(api, ctx);
			api.r_end();
		}
	}
}
