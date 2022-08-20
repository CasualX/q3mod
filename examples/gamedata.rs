use std::env;

use pelite::pattern as pat;
use pelite::pe64::*;

fn main() {
	let args = env::args_os().collect::<Vec<_>>();

	if args.len() != 2 {
		return;
	}

	let fmap = pelite::FileMap::open(&args[1]).expect("file not found");
	let file = PeFile::from_bytes(&fmap).unwrap();
	let scanner = file.scanner();
	let mut save = [0u32; 4];

	println!("[GameData]");

	println!("TimeDateStamp={:#x}", file.file_header().TimeDateStamp);
	println!();

	// RE_RenderScene:
	if scanner.finds_code(pat!("488D15${\"R_RenderScene: NULL worldmodel\"} [1-150] 89(05|35)$'"), &mut save) {
		println!("tr.refdef={:#x}", save[1]);
	}
	else {
		eprintln!("error: cannot find tr.refdef");
	}

	// R_GetModelByHandle, RE_RegisterModel:
	// 4883F840 72% 448B1D$'
	if scanner.finds_code(pat!("4883F840 72% 448B1D$'"), &mut save) {
		println!("tr.models={:#x}", save[1]);
		println!("tr.maxmodels=1024");
	}
	else {
		eprintln!("error: cannot find tr.models");
	}
	println!();

	// CL_SetCGameTime:
	if scanner.finds_code(pat!("393D${'} 0F84$ 488D15$ \"CL_SetCGameTime:\""), &mut save) {
		println!("cl.snap={:#x}", save[1]);
	}
	else if scanner.finds_code(pat!("833D${?'}00 BB01000000 75? 488D15$ \"CL_SetCGameTime:\""), &mut save) {
		println!("cl.snap={:#x}", save[1]);
	}
	else {
		eprintln!("error: cannot find cl.snap");
	}

	// CL_ParseSnapshot:
	// cl.snap.ping = 999;
	if scanner.finds_code(pat!("8B?${'} C705????E7030000"), &mut save) {
		println!("cl.snap.ps={:#x}", save[1]);
		println!("cl.snap.ps_area_bytes=?");
	}
	else {
		eprintln!("error: cannot find cl.snap.ps");
	}

	// CL_MouseMove:
	// "ratex: %f", at the end of the function
	if scanner.finds_code(pat!("F30F1005${'} 488B????? F30F59?? F30F58"), &mut save) {
		println!("cl.viewangles={:#x}", save[1]);
	}
	else {
		eprintln!("error: cannot find cl.viewangles");
	}

	// CL_GetSnapshot:
	if scanner.finds_code(pat!("488D0D${\"CL_GetSnapshot: truncated\"}"), &mut save) {
		println!("cl.parseEntities={:#x}", save[1]);
		println!("MAX_PARSE_ENTITIES=?");
	}
	else {
		eprintln!("error: cannot find cl.parseEntities");
	}

	// CL_MapLoading:
	// "nextmap" and "localhost"
	if scanner.finds_code(pat!("488D15${\"localhost\"} [1-20] C705${????'}04000000"), &mut save) {
		println!("cls.state={:#x}", save[1]);
		println!("cls.framecount={:#x}", save[1] + 288);
	}
	else {
		eprintln!("error: cannot find cls.state");
	}
}
