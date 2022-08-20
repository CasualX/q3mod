use super::*;

#[derive(Default)]
pub struct Entities {
	parsed: Vec<sdk::entityState_t>,
	baseline: Vec<sdk::entityState_t>,
}

impl Entities {
	pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext, snap: &sdk::clSnapshot_t) {
		let len = i32::clamp(snap.numEntities, 0, sdk::MAX_SNAPSHOT_ENTITIES as i32) as usize;

		if self.parsed.len() != len {
			self.parsed.resize_with(len, Default::default);
		}

		let ptr = ctx.process.base.field::<[sdk::entityState_t]>(ctx.data.cl_parseEntities);

		let start = snap.parseEntitiesNum as usize & (ctx.data.max_parse_entities as usize - 1);
		let end = (snap.parseEntitiesNum as usize + len) & (ctx.data.max_parse_entities as usize - 1);

		if start <= end {
			let _ = api.vm_read_into(ptr.at(start).cast(), &mut self.parsed[..]);
		}
		else {
			let (lhs, rhs) = self.parsed.split_at_mut(ctx.data.max_parse_entities as usize - start);
			let _ = api.vm_read_into(ptr.at(start).cast(), lhs);
			let _ = api.vm_read_into(ptr, rhs);
		}

		if self.baseline.len() != sdk::MAX_GENTITIES {
			self.baseline.resize_with(sdk::MAX_GENTITIES, Default::default);
		}
		for gent in &mut self.baseline {
			gent.number = -1;
		}

		for pent in &self.parsed {
			if let Some(gent) = self.baseline.get_mut(pent.number as usize) {
				*gent = *pent;
			}
		}
	}
}

impl GameState {
	pub fn entities(&self) -> impl '_ + Iterator<Item = &sdk::entityState_t> {
		self.ents.baseline.iter().filter(|e| e.number != -1)
	}
	pub fn get_entity(&self, entnum: i32) -> Option<&sdk::entityState_t> {
		let ent = self.ents.baseline.get(entnum as usize)?;
		if ent.number != entnum {
			return None;
		}
		return Some(ent);
	}
}
