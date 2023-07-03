// Running Windows Quake3 client on Linux under Wine, eg:
// wine quake3e.x64.exe

#![cfg(unix)]

use std::{fmt, fs, mem, slice, time, thread};
use std::io::prelude::*;
use std::path::Path;

use obfstr::obfstr as s;
use fmtools::fmt as f;

fn game(host: &mut Host, instance: &mut q3mod::Instance, offsets: &str) {
	if instance.attach(host, offsets) {
		while host.heartbeat() {
			instance.tick(host);
			host.tick(instance);
		}
	}
	else {
		host.log(s!("Attach error, missing offsets?"));
	}
	host.log(s!("Exited Quake3"));
}

fn check(host: &mut Host, pid: u32, path: &Path) -> bool {
	let Ok(cmdline) = fs::read_to_string(path.join(s!("cmdline"))) else { return false };
	let cmdline = cmdline.trim_end_matches("\0");

	s! {
		let quake3e_x64 = "quake3e.x64.exe";
		let cnq3_x64 = "cnq3-x64.exe";
	}
	if ![quake3e_x64, cnq3_x64].iter().any(|&exe| cmdline.ends_with(exe)) {
		return false;
	}

	// On Linux under wine it's not guaranteed to have the main executable loaded
	// However the cheat assumes this is true, so wait until it's found
	host.load_maps(pid);
	if q3mod::Interface::base_address(host) == 0 {
		return false;
	}

	host.pid = pid;
	host.load_input();
	return true;
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
		time_base: time::Instant::now(),
		server_timer: 0.0,
		server,
		signal: None,
		debug: false,
		attached: false,
		pid: 0,
		maps: Vec::new(),
		mstats: MemoryStats::default(),
		mouse_fd: None,
		kbd_fd: None,
		key_map_refresh: true,
		key_map: [0; KEY_LEN],
	};

	let mut instance = q3mod::Instance::default();

	host.server.settingsui(q3mod::SETTINGSUI);

	loop {
		let Ok(proc) = fs::read_dir(s!("/proc")) else {
			host.log(s!("Failed to read /proc"));
			break;
		};

		for entry in proc {
			let Ok(entry) = entry else { continue };
			let path = entry.path();

			// Look for process directories
			let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else { continue };
			let Ok(pid) = file_name.parse::<u32>() else { continue };

			// Avoid reattaching to the same process
			if pid == 0 || pid == host.pid {
				continue;
			}

			if check(&mut host, pid, &path) {
				host.log(f!(move "Now playing Quake3 (pid="{pid}")"));
				let offsets = "";

				host.attached = true;
				game(&mut host, &mut instance, &offsets);
				host.attached = false;

				host.maps.clear();
				host.mouse_fd = None;
				host.kbd_fd = None;
			}
		}

		// Handle any client connections
		if host.signal.is_none() {
			host.tick(&mut instance);
		}

		// Handle signals
		match host.signal.take() {
			Some(Signal::Break) => (),
			Some(Signal::Retry) => {
				host.pid = 0;
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

fn sleep(ms: u32) {
	thread::sleep(time::Duration::from_millis(ms as u64));
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

struct MemoryMap {
	address: u64,
	path: String,
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
	time_base: time::Instant,

	server_timer: f64,
	server: ectows::Server,
	signal: Option<Signal>,

	debug: bool,
	attached: bool,
	pid: u32,
	maps: Vec<MemoryMap>,
	mstats: MemoryStats,

	mouse_fd: Option<fs::File>,
	kbd_fd: Option<fs::File>,

	key_map_refresh: bool,
	key_map: [u8; KEY_LEN],
}

impl Host {
	fn log(&mut self, args: impl fmt::Display) {
		log(self, format_args!("{}", args));
	}

	fn get_time(&self) -> f64 {
		time::Instant::now().duration_since(self.time_base).as_secs_f64()
	}

	fn tick(&mut self, root: &mut dyn cvar::IVisit) {
		let time = self.get_time();
		self.update_stats(time);

		if timer(&mut self.server_timer, time, 1.0 / 60.0) {
			let mut visitor = cvar::Visit(|f| {
				f(&mut cvar::Action(s!("exit!"), |_, _| self.signal = Some(Signal::Exit)));
				f(&mut cvar::Action(s!("break!"), |_, _| self.signal = Some(Signal::Break)));
				f(&mut cvar::Action(s!("retry!"), |_, _| self.signal = Some(Signal::Retry)));
				f(&mut cvar::Property(s!("debug.enable"), &mut self.debug, &false));
				root.visit(f);
			});
			self.server.tick(&mut visitor);
		}
	}

	fn update_stats(&mut self, time: f64) {
		self.mstats.iters += 1;

		self.mstats.total_time += time - self.mstats.update_time;
		self.mstats.update_time = time;

		if timer(&mut self.mstats.report_time, time, 0.1) {
			if self.debug {
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
			}
			self.mstats.iters = 0;
			self.mstats.gathers = 0;
			self.mstats.reads = 0;
			self.mstats.writes = 0;
			self.mstats.transferred = 0;
			self.mstats.io_time = 0.0;
			self.mstats.total_time = 0.0;
		}
	}

	#[inline(never)]
	fn load_maps(&mut self, pid: u32) {
		self.maps.clear();
		if pid == 0 {
			return;
		}

		let path = fmtools::format!("/proc/"{pid}"/maps");
		let Ok(maps) = fs::read_to_string(path) else { return };

		let re = regex_lite::Regex::new(s!(r"(\w+)-\w+ .... \w+ \w+:\w+ \d+\s+(.*)")).unwrap();
		for line in maps.lines() {
			let Some(caps) = re.captures(line) else { continue };
			let Some(address) = caps.get(1) else { continue };
			let Some(path) = caps.get(2) else { continue };
			let path = path.as_str();
			let address = address.as_str();
			let Ok(address) = u64::from_str_radix(address, 16) else { continue };

			self.maps.push(MemoryMap { address, path: path.to_owned() });
		}
	}

	#[inline(never)]
	fn load_input(&mut self) {
		// Gather mouse and keyboard devices
		for entry in fs::read_dir("/dev/input/by-id").unwrap() {
			let Ok(entry) = entry else { continue };
			let path = entry.path();
			let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else { continue };
			if self.mouse_fd.is_none() && file_name.ends_with(s!("-event-mouse")) {
				self.log(f!("Using device: "{path.display()}));
				match fs::OpenOptions::new().read(true).write(true).open(&path) {
					Ok(fd) => self.mouse_fd = Some(fd),
					Err(err) => self.log(f!("Failed to open mouse device: "{err})),
				}
			}
			if self.kbd_fd.is_none() && file_name.ends_with(s!("-event-kbd")) {
				self.log(f!("Using device: "{path.display()}));
				match fs::OpenOptions::new().read(true).write(true).open(&path) {
					Ok(fd) => self.kbd_fd = Some(fd),
					Err(err) => self.log(f!("Failed to open keyboard device: "{err})),
				}
			}
		};
		if self.mouse_fd.is_none() {
			self.log(s!("Failed to find mouse device"));
		}
		if self.kbd_fd.is_none() {
			self.log(s!("Failed to find keyboard device"));
		}
	}

	fn heartbeat(&mut self) -> bool {
		// During heartbeat reset these resources
		// They are reloaded on first use
		self.key_map_refresh = true;

		// Break out if a pending signal is present
		if self.signal.is_some() {
			return false;
		}

		unsafe {
			let pid = self.pid as libc::pid_t;
			libc::kill(pid, 0) == 0
		}
	}
}

const GATHER_MEMORY_MAX_LEN: usize = 64;

impl q3mod::Interface for Host {
	fn get_time(&mut self) -> f64 {
		Host::get_time(self)
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
		let Some(mouse) = self.mouse_fd.as_mut() else { return };
		let ev = [
			libc::input_event {
				time: libc::timeval { tv_sec: 0, tv_usec: 0 },
				type_: 0x02, // EV_REL
				code: 0x00, // REL_X
				value: dx,
			},
			libc::input_event {
				time: libc::timeval { tv_sec: 0, tv_usec: 0 },
				type_: 0x02, // EV_REL
				code: 0x01, // REL_Y
				value: dy,
			},
			libc::input_event {
				time: libc::timeval { tv_sec: 0, tv_usec: 0 },
				type_: 0x00, // EV_SYN
				code: 0x00, // SYN_REPORT
				value: 0,
			},
		];

		let buf = unsafe {
			slice::from_raw_parts(ev.as_ptr() as *const u8, mem::size_of_val(&ev))
		};
		if let Err(err) = mouse.write(buf) {
			self.log(f!("mouse_move: "{err}))
		}
	}

	fn is_button_down(&mut self, q3btn: q3mod::Button) -> bool {
		// Refresh the key map once per tick
		if self.key_map_refresh {
			self.key_map_refresh = false;
			self.key_map.fill(0);
			// Update our copy of the key map with the current state of the keyboard and mouse buttons
			for file in [self.kbd_fd.as_mut(), self.mouse_fd.as_mut()] {
				let Some(file) = file else { continue };
				let fileno = std::os::fd::AsRawFd::as_raw_fd(file);
				let mut key_map = [0u8; KEY_LEN];
				unsafe {
					libc::ioctl(fileno, EVIOCGKEY(KEY_LEN), &mut key_map);
				}
				for i in 0..KEY_LEN {
					self.key_map[i] |= key_map[i];
				}
			}
		}

		let btn: u16 = match q3btn {
			q3mod::Button::MouseLeft => 0x110, // BTN_LEFT
			q3mod::Button::MouseRight => 0x111, // BTN_RIGHT
			q3mod::Button::MouseMiddle => 0x112, // BTN_MIDDLE
			q3mod::Button::MouseX1 => 0x113, // BTN_SIDE
			q3mod::Button::MouseX2 => 0x114, // BTN_EXTRA
			q3mod::Button::Space => 0x39, // KEY_SPACE
			q3mod::Button::Enter => 0x1c, // KEY_ENTER
			q3mod::Button::Tab => 0x0f, // KEY_TAB
			q3mod::Button::LShift => 0x2a, // KEY_LEFTSHIFT
			q3mod::Button::LCtrl => 0x1d, // KEY_LEFTCTRL
			q3mod::Button::LAlt => 0x38, // KEY_LEFTALT
			q3mod::Button::RShift => 0x36, // KEY_RIGHTSHIFT
			q3mod::Button::RCtrl => 0x1d, // KEY_RIGHTCTRL
			q3mod::Button::RAlt => 0x38, // KEY_RIGHTALT
		};
		let btn = btn as usize;
		let byte = self.key_map[btn / 8];
		let mask = 1 << (btn % 8);
		return byte & mask != 0;
	}

	fn base_address(&mut self) -> u64 {
		for map in &self.maps {
			if map.path.ends_with(s!(".exe")) {
				return map.address;
			}
		}
		return 0;
	}

	fn read_memory(&mut self, address: u64, dest: &mut [u8]) -> isize {
		if !self.attached {
			return -1;
		}

		let start = self.get_time();
		let nbytes = unsafe {
			let local_iov = libc::iovec {
				iov_base: dest.as_mut_ptr() as *mut libc::c_void,
				iov_len: dest.len(),
			};
			let remote_iov = libc::iovec {
				iov_base: address as *mut libc::c_void,
				iov_len: dest.len(),
			};
			let pid = self.pid as libc::pid_t;
			libc::process_vm_readv(pid, &local_iov, 1, &remote_iov, 1, 0)
		};

		self.mstats.io_time += self.get_time() - start;
		self.mstats.reads += 1;

		if nbytes < 0 || nbytes as usize != dest.len() {
			return -1;
		}

		self.mstats.transferred += nbytes as u32;
		return nbytes;
	}

	fn gather_memory(&mut self, address: u64, _size: u32, indices: &mut [u32]) -> isize {
		if !self.attached {
			return -1;
		}

		let start = self.get_time();
		let nbytes = unsafe {
			let mut local_iovs: [libc::iovec; GATHER_MEMORY_MAX_LEN] = mem::zeroed();
			let mut remote_iovs: [libc::iovec; GATHER_MEMORY_MAX_LEN] = mem::zeroed();

			let mut cnt = 0;
			let base = indices.as_mut_ptr();

			for i in 0..indices.len() {
				if cnt >= GATHER_MEMORY_MAX_LEN {
					return -1;
				}
				if i > 0 && cnt > 0 && indices[i - 1] + 4 == indices[i] {
					local_iovs[cnt - 1].iov_len += 4;
					remote_iovs[cnt - 1].iov_len += 4;
				}
				else {
					local_iovs[cnt].iov_base = base.offset(i as isize) as *mut libc::c_void;
					local_iovs[cnt].iov_len = 4;
					remote_iovs[cnt].iov_base = (address + indices[i] as u64) as *mut libc::c_void;
					remote_iovs[cnt].iov_len = 4;
					cnt += 1;
				}
			}

			let pid = self.pid as libc::pid_t;
			libc::process_vm_readv(pid, local_iovs.as_ptr(), cnt as u64, remote_iovs.as_ptr(), cnt as u64, 0)
		};

		self.mstats.io_time += self.get_time() - start;
		self.mstats.gathers += 1;

		if nbytes < 0 || nbytes as usize != indices.len() * 4 {
			return -1;
		}

		self.mstats.transferred += nbytes as u32;
		return nbytes;
	}

	fn write_memory(&mut self, address: u64, src: &[u8]) -> isize {
		if !self.attached {
			return -1;
		}

		let start = self.get_time();
		let nbytes = unsafe {
			let local_iov = libc::iovec {
				iov_base: src.as_ptr() as *mut libc::c_void,
				iov_len: src.len(),
			};
			let remote_iov = libc::iovec {
				iov_base: address as *mut libc::c_void,
				iov_len: src.len(),
			};
			let pid = self.pid as libc::pid_t;
			libc::process_vm_writev(pid, &local_iov, 1, &remote_iov, 1, 0)
		};

		self.mstats.io_time += self.get_time() - start;
		self.mstats.writes += 1;

		if nbytes < 0 || nbytes as usize != src.len() {
			return -1;
		}

		self.mstats.transferred += nbytes as u32;
		return nbytes;
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

const KEY_LEN: usize = 0x300 / 8;

#[allow(non_snake_case)]
fn EVIOCGKEY(len: usize) -> u64 {
	_IOC(2, 'E', 0x18, len)
}
#[allow(non_snake_case)]
fn _IOC(dir: u64, ty: char, nr: usize, size: usize) -> u64 {
	const _IOC_NRBITS: u64 = 8;
	const _IOC_TYPEBITS: u64 = 8;
	const _IOC_SIZEBITS: u64 = 14;
	const _IOC_DIRBITS: u64 = 2;
	const _IOC_NRSHIFT: u64 = 0;
	const _IOC_TYPESHIFT: u64 = _IOC_NRSHIFT + _IOC_NRBITS;
	const _IOC_SIZESHIFT: u64 = _IOC_TYPESHIFT + _IOC_TYPEBITS;
	const _IOC_DIRSHIFT: u64 = _IOC_SIZESHIFT + _IOC_SIZEBITS;

	(dir << _IOC_DIRSHIFT) | ((ty as u64) << _IOC_TYPESHIFT) | ((nr as u64) << _IOC_NRSHIFT) | ((size as u64) << _IOC_SIZESHIFT)
}
