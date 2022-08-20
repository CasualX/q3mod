use super::*;

#[derive(Default)]
pub struct Models {
	timer: base::Timer,
	ptrs: Vec<sdk::Ptr>,
	names: Vec<String>,
	index: usize,
}

impl Models {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
		let maxmodels = ctx.data.tr_maxmodels as usize;
		if maxmodels == 0 {
			return;
		}

		if self.ptrs.len() != maxmodels {
			self.ptrs.resize_with(maxmodels, Default::default);
		}
		if self.names.len() != maxmodels {
			self.names.resize_with(maxmodels, Default::default);
		}

		// Don't need to read the models array too often
		// Perhaps only read it once on level load?
		if self.timer.has_elapsed(ctx.time, 1.1) {
			let _ = api.vm_read_into(ctx.process.base.field(ctx.data.tr_models), &mut self.ptrs[..]);
		}

		// Load a single model_t per tick
		// Find a non-zero model_t to read
		for _ in 0..maxmodels {
			self.index += 1;
			if self.index >= maxmodels {
				self.index = 0;
			}

			if let Some(ptr) = self.ptrs.get(self.index) {
				if !ptr.is_null() {
					if let Some(entry) = self.names.get_mut(self.index) {
						let mut buf = [0u8; 64];
						if let Ok(name) = api.vm_read_cstr(ptr.cast(), &mut buf) {
							name.clone_into(entry);
						}
					}
					break;
				}
			}
		}
	}

	pub fn get_name(&self, model: i32) -> Option<&str> {
		let entry = self.names.get(model as usize)?;
		Some(entry)
	}
}

impl GameState {
	pub fn get_model_name(&self, model: i32) -> Option<&str> {
		self.models.get_name(model)
	}
}
