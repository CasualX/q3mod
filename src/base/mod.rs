
// Randomize the hash function used for string lookups
pub const SEED: u32 = obfstr::random!(u32);

#[inline]
pub const fn hash(s: &str) -> u32 {
	obfstr::murmur3(s.as_bytes(), SEED)
}
macro_rules! hash {
	($s:expr) => { ::obfstr::murmur3!($s.as_bytes(), crate::base::SEED) };
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Timer {
	last_time: f64,
}
impl Timer {
	pub fn has_elapsed(&mut self, time: f64, interval: f64) -> bool {
		let next_time = self.last_time + interval;
		if time >= next_time {
			self.last_time = if time >= next_time + interval { time } else { next_time };
			true
		}
		else {
			false
		}
	}
}

pub mod math;
pub mod solver;

#[inline(never)]
pub fn strn(bytes: &[u8]) -> Option<&str> {
	let len = bytes.iter().position(|&byte| byte == 0).unwrap_or(bytes.len());
	std::str::from_utf8(&bytes[..len]).ok()
}

pub mod pid;

pub fn parse_u32(value: &str) -> u32 {
	let (src, radix) = if value.starts_with(obfstr::obfstr!("0x")) {
		let Some(s) = value.get(2..) else { return 0 };
		(s, 16)
	}
	else {
		(value, 10)
	};
	match u32::from_str_radix(src, radix) {
		Ok(v) => v,
		Err(_) => 0,
	}
}
