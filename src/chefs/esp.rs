use super::*;

#[derive(Default)]
pub struct ESP {

}

impl cvar::IVisit for ESP {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {

	}
}

impl ESP {
	pub fn run(&mut self, api: &mut Api, ctx: &RunContext) {

		let ps = ctx.state.player_state();
		for ent in ctx.state.entities() {
			if ent.pos.trBase == [0.0; 3] {
				continue;
			}
			if let Some([x, y]) = ctx.world_to_screen(ent.pos.trBase, true) {
				api.r_rect(x - 2.0, y - 2.0, 4.0, 4.0, vgc::sRGBA::TRANSPARENT, vgc::sRGBA!(Red));
				api.r_text(0, 3, x, y, 1000.0, 100.0, vgc::sRGBA!(Red), vgc::sRGBA!(Black), &fmtools::format!({ent.modelindex}"-"{ent.modelindex2}));
			}
		}
		for r_ent in ctx.state.ref_entities() {
			if let Some([x, y]) = ctx.world_to_screen(r_ent.origin, true) {
				api.r_rect(x - 2.0, y - 2.0, 4.0, 4.0, vgc::sRGBA::TRANSPARENT, vgc::sRGBA!(Lime));

				let name = ctx.state.get_model_name(r_ent.hModel).unwrap_or("BAD");
				api.r_text(0, 3, x, y, 1000.0, 100.0, vgc::sRGBA!(Lime), vgc::sRGBA!(Black), name);
			}
		}
	}
}
