use super::*;

#[derive(Default)]
pub struct GameProcess {
	pub base: sdk::Ptr,
	pub time_date_stamp: u32,
	pub checksum: u32,
	pub size_of_image: u32,
}

impl GameProcess {
	pub fn attach(&mut self, api: &mut Api) -> bool {
		self.time_date_stamp = 0;
		self.checksum = 0;
		self.size_of_image = 0;

		let base = sdk::Ptr::from_raw(api.base_address());
		api.log(f!({base:#x}" base address"));

		self.base = base;
		if base.is_null() {
			return false;
		}

		{
			use pelite::pe64::*;

			let Ok(dos_header) = api.vm_read::<image::IMAGE_DOS_HEADER>(base.cast()) else { return false };
			let Ok(nt_headers) = api.vm_read::<image::IMAGE_NT_HEADERS>(base.field(dos_header.e_lfanew)) else { return false };

			self.time_date_stamp = nt_headers.FileHeader.TimeDateStamp;
			self.checksum = nt_headers.OptionalHeader.CheckSum;
			self.size_of_image = nt_headers.OptionalHeader.SizeOfImage;

			api.log(f!("TimeDateStamp="{self.time_date_stamp:#x}));
			api.log(f!("CheckSum="{self.checksum:#x}));
			api.log(f!("SizeOfImage="{self.size_of_image:#x}));
		}

		return true;
	}

	pub fn is_valid(&self, _api: &mut Api) -> bool {
		if self.base.is_null() {
			return false;
		}
		if self.time_date_stamp == 0 {
			return false;
		}
		return true;
	}
}
