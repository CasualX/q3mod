#![cfg(windows)]

use std::fmt;

use external::FromInner;
use external::system::time_s;
use external::wndclass::sleep;
use external::process::{Process, ProcessId, ProcessList, ProcessRights};
use external::mouse::Mouse;
use external::vk::VirtualKey;
use obfstr::obfstr as s;
use fmtools::fmt as f;

fn game(host: &mut Host, gamedata: &str) {
	let mut instance = q3mod::Instance::default();
	if instance.attach(host, gamedata) {
		while host.heartbeat() {
			instance.tick(host);
			host.tick(&mut instance);
		}
	}
	else {
		host.log(s!("Attach error, missing offsets?"));
	}
	host.log(s!("Exited Quake3"));
}

fn read_offsets() -> String {
	let mut gamedata = String::new();
	if let Ok(dir) = fs::read_dir(s!("offsets")) {
		for entry in dir {
			let Ok(entry) = entry else { continue };
			let path = entry.path();
			if let Ok(mut file) = fs::File::open(path) {
				if let Ok(_) = file.read_to_string(&mut gamedata) {
					gamedata.push_str("\n");
				}
			}
		}
	}
	gamedata
}

fn main() {
	let port = 30145;
	let server = ectows::Server::create(port).unwrap();

	fmtools::print!(
		"Listening on port "{port}"...\n"
		"Visit the website to connect:\n"
		"https://casualhacks.net/aurascope/#address=ws%3A%2F%2Flocalhost%3A"{port}"%2Fadmin%3Ftoken%3Dadmin\n\n"
	);

	let mut host = Host {
		server_timer: 0.0,
		server,
		signal: None,

		pid: unsafe { ProcessId::from_inner(0) },
		process: None,
		mstats: MemoryStats::default(),
	};

	loop {
		for pi in ProcessList::query().iter() {
			let pid = pi.process_id();

			if pid == host.pid {
				continue;
			}

			if pi.image_name_wide() == obfstr::wide!("quake3e.x64.exe") {
				host.pid = pid;
				host.process = Process::attach(host.pid, ProcessRights::ALL_ACCESS).ok();
				host.log(f!("Now playing Quake3 (pid="{pid}")"));

				let gamedata = read_offsets();
				if gamedata.is_empty() {
					host.log(s!("WARNING: No offsets found!"));
				}

				game(&mut host, &gamedata);

				host.process = None;
			}
		}

		// Handle any client connections
		let mut signal = host.signal.take();
		if signal.is_none() {
			host.tick(&mut cvar::Visit(|f| {
				f(&mut cvar::Action(s!("exit!"), |_, _| signal = Some(Signal::Exit)));
				f(&mut cvar::Action(s!("retry!"), |_, _| signal = Some(Signal::Retry)));
			}));
		}

		// Handle signals
		match signal {
			Some(Signal::Break) => (),
			Some(Signal::Retry) => {
				host.pid = unsafe { ProcessId::from_inner(0) };
			},
			Some(Signal::Exit) => break,
			None => (),
		}

		sleep(500);
	}
}

fn timer(next_tick: &mut f64, time: f64, interval: f64) -> bool {
	if time < *next_tick {
		return false;
	}
	*next_tick = if time < *next_tick + interval { *next_tick + interval } else { time + interval };
	return true;
}

#[inline(never)]
fn log(host: &mut Host, args: fmt::Arguments) {
	host.server.log(args);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Signal {
	/// Exit the cheat, do not attach to the same process again.
	Break,
	/// Reinitialize the cheat and attach again.
	Retry,
	/// Exit the cheat host.
	Exit,
}

#[derive(Copy, Clone, Default)]
struct MemoryStats {
	iters: u32, // Number of accumulated iterations
	gathers: u32, // Number of gathers
	reads: u32, // Number of reads
	writes: u32, // Number of writes
	transferred: u32, // Total bytes transferred
	io_time: f64, // Total time in gather/read/write
	total_time: f64, // Total measuring time
	update_time: f64, // Last time updated
	report_time: f64, // Report 10 times per sec
}

struct Host {
	server_timer: f64,
	server: ectows::Server,
	signal: Option<Signal>,

	pid: ProcessId,
	process: Option<external::process::Process>,
	mstats: MemoryStats,
}

impl Host {
	fn log(&mut self, args: impl fmt::Display) {
		log(self, format_args!("{}", args));
	}

	fn tick(&mut self, root: &mut dyn cvar::IVisit) {
		let time = time_s();
		self.update_stats(time);

		let mut signal_data = None;
		let signal = &mut signal_data;
		let attached = self.process.is_some();

		if timer(&mut self.server_timer, time, 1.0/60.0) {
			let mut visitor = cvar::Visit(move |f| {
				if attached {
					f(&mut cvar::Action(s!("exit!"), |_, _| *signal = Some(Signal::Exit)));
					f(&mut cvar::Action(s!("break!"), |_, _| *signal = Some(Signal::Break)));
					f(&mut cvar::Action(s!("retry!"), |_, _| *signal = Some(Signal::Retry)));
				}
				root.visit(f);
			});

			self.server.tick(&mut visitor);
		}

		if signal_data.is_some() {
			self.signal = signal_data;
		}
	}

	fn update_stats(&mut self, time: f64) {
		self.mstats.iters += 1;

		self.mstats.total_time += time - self.mstats.update_time;
		self.mstats.update_time = time;

		if timer(&mut self.mstats.report_time, time, 0.1) {
			let ms = &self.mstats;
			self.server.visualize(s!("MemoryStats"), format_xml::format_args! {
				let iters = ms.iters as f64;
				let frame_time = ms.total_time / iters;
				let io_time = ms.io_time / iters;
				let gathers = (ms.gathers as f64 / iters).ceil();
				let reads = (ms.reads as f64 / iters).ceil();
				let writes = (ms.writes as f64 / iters).ceil();
				let transferred = (ms.transferred as f64 / iters).ceil();
				<pre>
					"FPS:       "{(1.0 / frame_time).round():.0}"\n"
					"FrameTime: "{frame_time * 1000.0:.1}" ms\n"
					"IoTime:    "{io_time * 1000.0:.1}" ms\n"
					"IoTime%:   "{io_time / frame_time * 100.0:.0}"%\n"
					"Gathers:   "{gathers}"\n"
					"Reads:     "{reads}"\n"
					"Writes:    "{writes}"\n"
					"IoOps:     "{gathers + reads + writes}"\n"
					"Transfer:  "{transferred / 1024.0:.2}" KiB\n"
				</pre>
			});
			self.mstats.iters = 0;
			self.mstats.gathers = 0;
			self.mstats.reads = 0;
			self.mstats.writes = 0;
			self.mstats.transferred = 0;
			self.mstats.io_time = 0.0;
			self.mstats.total_time = 0.0;
		}
	}

	fn heartbeat(&mut self) -> bool {
		if self.signal.is_some() {
			return false;
		}

		let Some(process) = &self.process else { return false };
		match process.exit_code() {
			Ok(None) => true,
			_ => false,
		}
	}
}

impl q3mod::Interface for Host {
	fn get_time(&mut self) -> f64 {
		time_s()
	}

	fn sleep(&mut self, ms: u32) {
		sleep(ms);
	}

	fn log(&mut self, args: fmt::Arguments) {
		log(self, format_args!(" {}", args));
	}

	fn visualize(&mut self, scope: &str, args: fmt::Arguments) {
		self.server.visualize(scope, args);
	}

	fn mouse_move(&mut self, dx: i32, dy: i32) {
		Mouse.mouse_move(dx, dy);
	}

	fn is_button_down(&mut self, q3btn: q3mod::Button) -> bool {
		let btn = match q3btn {
			q3mod::Button::MouseLeft => VirtualKey::LBUTTON,
			q3mod::Button::MouseRight => VirtualKey::RBUTTON,
			q3mod::Button::MouseMiddle => VirtualKey::MBUTTON,
			q3mod::Button::MouseX1 => VirtualKey::XBUTTON1,
			q3mod::Button::MouseX2 => VirtualKey::XBUTTON2,
			q3mod::Button::Space => VirtualKey::SPACE,
			q3mod::Button::Enter => VirtualKey::RETURN,
			q3mod::Button::Tab => VirtualKey::TAB,
			q3mod::Button::LShift => VirtualKey::LSHIFT,
			q3mod::Button::LCtrl => VirtualKey::LCTRL,
			q3mod::Button::LAlt => VirtualKey::LALT,
			q3mod::Button::RShift => VirtualKey::RSHIFT,
			q3mod::Button::RCtrl => VirtualKey::RCTRL,
			q3mod::Button::RAlt => VirtualKey::RALT,
		};
		btn.async_state()
	}

	fn base_address(&mut self) -> u64 {
		let Some(_process) = &self.process else { return 0 };
		let Ok(em) = external::module::EnumModules::create(self.pid) else { return 0 };
		for module in em {
			if module.name_wide().ends_with(obfstr::wide!(".exe")) {
				return module.base() as u64;
			}
		}
		return 0;
	}

	fn read_memory(&mut self, address: u64, dest: &mut [u8]) -> isize {
		let Some(process) = &self.process else { return -1 };
		let start = time_s();
		let result = process.vm_read_into(address.into(), dest);
		self.mstats.io_time += time_s() - start;
		self.mstats.reads += 1;
		if result.is_err() {
			return -1;
		}
		self.mstats.transferred += dest.len() as u32;
		return dest.len() as isize;
	}

	fn gather_memory(&mut self, address: u64, _size: u32, indices: &mut [u32]) -> isize {
		let Some(process) = &self.process else { return -1 };
		let start = time_s();
		let mut any_errors = false;
		for entry in indices.iter_mut() {
			let offset = *entry as u64;
			let mut value = [0u8; 4];
			if let Ok(_) = process.vm_read_into((address + offset).into(), &mut value) {
				*entry = u32::from_le_bytes(value);
			}
			else {
				any_errors = true;
			}
		}
		self.mstats.io_time += time_s() - start;
		self.mstats.gathers += 1;
		let nbytes = indices.len() * 4;
		self.mstats.transferred += nbytes as u32;
		return if any_errors { -1 } else { nbytes as isize };
	}

	fn write_memory(&mut self, address: u64, src: &[u8]) -> isize {
		let Some(process) = &self.process else { return -1 };
		let start = time_s();
		let result = process.vm_write(address.into(), src);
		self.mstats.io_time += time_s() - start;
		self.mstats.writes += 1;
		if result.is_err() {
			return -1;
		}
		self.mstats.transferred += src.len() as u32;
		return src.len() as isize;
	}

	fn r_begin(&mut self, screen: &mut [i32; 2]) -> bool {
		_ = screen;
		return false;
	}

	#[allow(unused_variables)]
	fn r_rect(&mut self, x: f32, y: f32, width: f32, height: f32, fill: u32, stroke: u32) {
		todo!()
	}

	#[allow(unused_variables)]
	fn r_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32, fill: u32, stroke: u32) {
		todo!()
	}

	#[allow(unused_variables)]
	fn r_text(&mut self, font: u32, flags: u32, x: f32, y: f32, width: f32, height: f32, color: u32, color2: u32, text: &str) {
		todo!()
	}

	#[allow(unused_variables)]
	fn r_line(&mut self, color: u32, x1: f32, y1: f32, x2: f32, y2: f32) {
		todo!()
	}

	#[allow(unused_variables)]
	fn r_lines(&mut self, color: u32, points: &[[f32; 2]], lines: &[[u16; 2]]) {
		todo!()
	}

	#[allow(unused_variables)]
	fn r_image(&mut self, image: u32, sx: f32, sy: f32, swidth: f32, sheight: f32, dx: f32, dy: f32, dwidth: f32, dheight: f32, opacity: f32) {
		todo!()
	}

	fn r_end(&mut self) {
	}
}
