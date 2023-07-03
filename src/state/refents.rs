use super::*;

#[allow(non_snake_case)]
#[derive(Default)]
pub struct RefEntities {
	r_numentities: u32,
	r_goodentities: u32,
	backEndData: sdk::Ptr,

	entbuf: Vec<sdk::refEntity_t>,
	entities: Vec<sdk::refEntity_t>,

	numentsbuf: Vec<u32>,
	pub numentslog: Vec<u32>,
}

impl RefEntities {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext, models: &Models, connected: bool) {
		if ctx.data.r_numentities == 0 || ctx.data.backEndData == 0 {
			return;
		}

		if !connected {
			self.entities.clear();
			return;
		}

		// Read the r_numentities as the game renders entities
		let r_numentities = self.r_numentities;
		let _ = api.vm_read_into(ctx.process.base.field(ctx.data.r_numentities), &mut self.r_numentities);

		// Check if passed InitNextFrame()
		if self.r_numentities < r_numentities {
			// backEndData ptr never changes, except when it does
			// Keep refreshing it at the start of every frame
			let _ = api.vm_read_into(ctx.process.base.field(ctx.data.backEndData), &mut self.backEndData);

			// Save a copy of the refEntities
			std::mem::swap(&mut self.entbuf, &mut self.entities);
			std::mem::swap(&mut self.numentsbuf, &mut self.numentslog);
			self.entbuf.clear();
			self.r_goodentities = 0;
			self.numentsbuf.clear();
		}

		if self.backEndData.is_null() {
			return;
		}

		if r_numentities != self.r_numentities {
			self.numentsbuf.push(self.r_numentities);
		}

		// Make room for newly added refEntities
		self.entbuf.resize_with(self.r_numentities as usize, Default::default);

		// Copy over refEntities
		for (r_index, r_ent) in self.entbuf.iter_mut().enumerate() {
			let r_index = r_index as u32;

			// I don't know why this fails sometimes
			// Keep trying until origin is non-zero
			// Or we've caught up with good entities
			if r_index >= self.r_goodentities && r_ent.origin == [0.0; 3] {
				let ptr = self.backEndData.field(ctx.data.r_entities + r_index * ctx.data.sizeof_refEntity_t);
				let _ = api.vm_read_into(ptr, r_ent);

				// refEntity_t read successfully
				if r_ent.origin != [0.0; 3] {
					self.r_goodentities = r_index + 1;

					// Apply generic wallhack effect
					ctx.wallhack.poke(api, models, ptr, r_ent);
				}
			}
		}
	}
}

#[allow(dead_code)]
impl GameState {
	pub fn ref_entities(&self) -> &[sdk::refEntity_t] {
		&self.refents.entities
	}

	pub fn find_nearest(&self, pos: &[f32; 3], part: &str) -> Option<&sdk::refEntity_t> {
		let mut current = f32::INFINITY;
		let mut nearest = None;

		for r_ent in &self.refents.entities {
			if let Some(model_name) = self.get_model_name(r_ent.hModel) {
				if model_name.ends_with(part) {
					let d = math::dist(*pos, r_ent.origin);
					if d < current {
						current = d;
						nearest = Some(r_ent);
					}
				}
			}
		}

		return nearest;
	}
}
