use pelite::pattern as pat;
use pelite::pe64::*;

use super::*;

pub const OFFSETS: &str = concat!(
	include_str!("offsets/cnq3-x64-unknown.ini"),
	include_str!("offsets/quake3e.x64-b62dc17a.ini"),
	include_str!("offsets/quake3e.x64-ba29947b.ini"),
	include_str!("offsets/quake3e.x64-unknown.ini"),
);

fn finds_any(scanner: &scanner::Scanner<PeView>, save: &mut [u32; 4], pats: &[&[pat::Atom]]) -> bool {
	pats.iter().any(|p| scanner.finds_code(p, save))
}

pub fn offsets(api: &mut Api, image: &[u8], s: &mut String) -> bool {
	let Ok(file) = PeView::from_bytes(&image) else { return false };
	let scanner = file.scanner();
	let mut save = [0u32; 4];
	let mut nerrors = 0;
	use std::fmt::Write;

	_ = fmtools::write!(s,
		"[Offsets]\n"
		"TimeDateStamp="{file.file_header().TimeDateStamp:#x}"\n\n"
	);

	// RE_RenderScene:
	if scanner.finds_code(pat!("488D15${\"R_RenderScene: NULL worldmodel\"} [1-150] 89(05|35)$'"), &mut save) {
		_ = fmtools::write!(s, "tr.refdef="{save[1]:#x}"\n");
	}
	else {
		api.log(s!("error: cannot find tr.refdef"));
		nerrors += 1;
	}

	// R_GetModelByHandle, RE_RegisterModel:
	// "Model name exceeds MAX_QPATH"
	if scanner.finds_code(pat!("4883F840 72% 448B1D${'} [1-20] 4C8D0D${'}"), &mut save) {
		_ = fmtools::write!(s,
			"tr.models="{save[2]:#x}"\n"
			"tr.numModels="{save[1]:#x}"\n"
			"tr.maxmodels="{(save[1] - save[2]) / 8}"\n"
		);
	}
	else {
		api.log(s!("error: cannot find tr.models"));
		nerrors += 1;
	}
	s.push_str("\n");

	// CL_SetCGameTime:
	if finds_any(&scanner, &mut save, &[
		pat!("393D${'} 0F84$ 488D15$ \"CL_SetCGameTime:\""),
		pat!("833D${?'}00 BB01000000 75? 488D15$ \"CL_SetCGameTime:\""),
	]) {
		_ = fmtools::write!(s, "cl.snap="{save[1]:#x}"\n");
	}
	else {
		api.log(s!("error: cannot find cl.snap"));
		nerrors += 1;
	}

	// CL_ParseSnapshot:
	// cl.snap.ping = 999;
	if scanner.finds_code(pat!("8B?${'} C705????E7030000"), &mut save) {
		_ = fmtools::write!(s, "cl.snap.ps="{save[1]:#x}"\n");
	}
	else {
		api.log(f!("error: cannot find cl.snap.ps"));
		nerrors += 1;
	}

	// clSnapshot_t structure is slightly different between engines
	// False for cnq3, true for quake3e
	if scanner.matches(pat!("\"cnq3\""), 0..file.optional_header().SizeOfImage).next(&mut save) {
		_ = fmtools::write!(s, "cl.snap.has_area_bytes=false\n");
	}
	else if scanner.matches(pat!("\"quake3e\""), 0..file.optional_header().SizeOfImage).next(&mut save) {
		_ = fmtools::write!(s, "cl.snap.has_area_bytes=true\n");
	}
	else {
		api.log(s!("error: cannot identify the quake3 engine"));
		nerrors += 1;
	}

	// CL_MouseMove:
	// "ratex: %f", at the end of the function
	if scanner.finds_code(pat!("F30F1005${'} 488B????? F30F59?? F30F58"), &mut save) {
		_ = fmtools::write!(s, "cl.viewangles="{save[1]:#x}"\n");
	}
	else {
		api.log(s!("error: cannot find cl.viewangles"));
		nerrors += 1;
	}

	// CL_GetSnapshot:
	// "CL_GetSnapshot: truncated %i entities to %i"
	if scanner.finds_code(pat!("u20000 ?FF? ?69 [1-20] 498D?u4"), &mut save) {
		_ = fmtools::write!(s,
			"cl.parseEntities="{save[2]:#x}"\n"
			"MAX_PARSE_ENTITIES="{save[1] + 1}"\n"
		);
	}
	else {
		api.log(s!("error: cannot find cl.parseEntities"));
		nerrors += 1;
	}
	s.push_str("\n");

	// CL_MapLoading:
	// "nextmap" and "localhost"
	if scanner.finds_code(pat!("488D15${\"localhost\"} [1-20] C705${????'}04000000"), &mut save) {
		_ = fmtools::write!(s,
			"cls.state="{save[1]:#x}"\n"
			"cls.framecount="{save[1] + 288:#x}"\n"
		);
	}
	else {
		api.log(s!("error: cannot find cls.state"));
		nerrors += 1;
	}
	s.push_str("\n");

	// RE_AddRefEntityToScene:
	let mut r_numentities = 0;
	if scanner.finds_code(pat!("448B05${'} 4181F8u4 7C? 488D15$\"RE_AddRefEntityToScene: Dropping refEntity\""), &mut save) {
		r_numentities = save[1];
	}
	else {
		api.log(s!("error: cannot find r_numentities"));
		nerrors += 1;
	}

	// RE_AddRefEntityToScene:
	if scanner.finds_code(pat!("488B?${'} [2-10] 4881?u4 4869?u20000"), &mut save) {
		_ = fmtools::write!(s,
			"backEndData="{save[1]:#x}"\n"
			"r_numentities="{r_numentities:#x}"\n"
			"r_entities="{save[2]:#x}"\n"
			"sizeof refEntity_t="{save[3]:#x}"\n"
		);
	}
	else {
		api.log(s!("error: cannot find RE_AddRefEntityToScene"));
		nerrors += 1;
	}
	s.push_str("\n");

	// SV_GameClientNum:
	// E8${ 8B05${'} 0FAFC1 4898 480305${'} C3 } 488D0D$"%5i "
	if scanner.finds_code(pat!("E8${ 8B05${'} 0FAFC1 4898 480305${'} C3 } 488D0D$\"%5i \""), &mut save) {
		_ = fmtools::write!(s, "sv.gameClients="{save[1] - 8:#x}"\n");
	}
	else {
		api.log(s!("warning: cannot find sv.gameClients"));
		// nerrors += 1; (optional)
	}

	if nerrors != 0 {
		api.log(f!("error: there were "{nerrors}" errors"));
	}

	return nerrors == 0;
}
