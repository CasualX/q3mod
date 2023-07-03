use super::*;

struct Config {
	enable: bool,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enable: false,
		}
	}
}

#[derive(Default)]
pub struct Debugger {
	timer: base::Timer,
	config: Config,
}

impl cvar::IVisit for Debugger {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		let default = Config::default();
		f(&mut cvar::Property(s!("enable"), &mut self.config.enable, &default.enable));
	}
}

impl Debugger {
	pub fn run(&mut self, api: &mut Api, ctx: &RunContext) {
		if !self.config.enable {
			return;
		}

		if self.timer.has_elapsed(ctx.state.time, 1.0/20.0) {
			self.refdef(api, ctx);
			self.client(api, ctx);
			self.player(api, ctx);
			self.snapshot(api, ctx);
			self.entities(api, ctx);
			self.ref_entities(api, ctx);
		}
	}

	fn refdef(&mut self, api: &mut Api, ctx: &RunContext) {
		let ref refdef = ctx.state.refdef.raw;
		api.visualize(s!("refdef"), format_xml::xfmt! {
			<pre>
				"x: "{refdef.x}"\n"
				"y: "{refdef.y}"\n"
				"width:  "{refdef.width}"\n"
				"height: "{refdef.height}"\n"
				"fov_x:    "{refdef.fov_x}"\n"
				"fov_y:    "{refdef.fov_y}"\n"
				"vieworg:  "{refdef.vieworg:?}"\n"
				"viewaxis: "{refdef.viewaxis:?}"\n"
			</pre>
		});
	}

	fn client(&mut self, api: &mut Api, ctx: &RunContext) {
		let ref cl = ctx.state.client;
		let ps = ctx.state.player_state();
		api.visualize(s!("Client"), format_xml::xfmt! {
			<h3>"ClientActive"</h3>
			<pre>
				"server_time: "{cl.server_time}"\n"
				"mapname:     "{cl.mapname}"\n"
				"viewangles:  "{cl.viewangles:?}"\n"
				"serverid:    "{cl.serverid}"\n"
			</pre>
			<h3>"ClientState"</h3>
			<pre>
				"state: "{cl.state}"\n"
				"framecount:    "{cl.framecount}"\n"
				"frametime:     "{cl.frametime}"\n"
				"realtime:      "{cl.realtime}"\n"
				"realframetime: "{cl.realframetime}"\n"
			</pre>
			<h3>"ClientConnection"</h3>
			<pre>
				"client_num: "{ps.clientNum}"\n"
			</pre>
		});
	}

	fn player(&mut self, api: &mut Api, ctx: &RunContext) {
		let ps = ctx.state.player_state();
		api.visualize(s!("PlayerState"), format_xml::xfmt! {
			<pre>
				"origin: "{ps.origin:?}"\n"
				"velocity: "{ps.velocity:?}"\n"
				"groundent: "{ps.groundEntityNum}"\n"
				"flags: "{ps.eFlags:#x}"\n"
				"clientNum: "{ps.clientNum}"\n"
				"weapon: "{ps.weapon}"\n"
				"weaponstate: "{ps.weaponstate}"\n"
				"viewheight: "{ps.viewheight}"\n"
				"ps.viewangles: "{ps.viewangles:?}"\n"
				"cl.viewangles: "{ctx.state.view_angles():?}"\n"
			</pre>
		});
	}

	fn snapshot(&mut self, api: &mut Api, ctx: &RunContext) {
		let ref snap = ctx.state.snap.raw;
		api.visualize(s!("ClSnapshot"), format_xml::xfmt! {
			<pre>{snap:#?}</pre>
		});
	}

	fn entities(&mut self, api: &mut Api, ctx: &RunContext) {
		let state = ctx.state;
		api.visualize(s!("Entities"), format_xml::xfmt! {
			<pre>
			<table>
			(<tr>
				<th>"Number"</th>
				<th>"eType"</th>
				<th>"eFlags"</th>
				<th>"pos.trType"</th>
				<th>"pos.trTime"</th>
				<th>"pos.trDur"</th>
				<th>"pos.trBase"</th>
				<th>"pos.trDelta"</th>
				<th>"origin"</th>
				<th>"origin2"</th>
			</tr>)
			for ent in state.entities() {
				<tr>
					(<td>{ent.number}</td>)
					(<td>{ent.eType}</td>)
					(<td>{ent.eFlags:#x}</td>)
					(<td>{ent.pos.trType}</td>)
					(<td>{ent.pos.trTime}</td>)
					(<td>{ent.pos.trDuration}</td>)
					(<td>{ent.pos.trBase:?}</td>)
					(<td>{ent.pos.trDelta:?}</td>)
					(<td>{ent.origin:?}</td>)
					(<td>{ent.origin2:?}</td>)
				</tr>
			}
			</table></pre>
		});
	}

	fn ref_entities(&self, api: &mut Api, ctx: &RunContext) {
		let state = ctx.state;
		let r_ents = state.ref_entities();

		api.visualize(s!("RefEntities"), format_xml::xfmt! {
			<pre>
			"r_numentities: "{state.refents.numentslog:?}"\n"
			"\n"
			<table>
			(<tr>
				<th>"reType"</th>
				<th>"hModel"</th>
				<th>"origin"</th>
			</tr>)
			for r_ent in r_ents {
				<tr>
					<td>{r_ent.reType}</td>
					<td>{r_ent.hModel}":"{ctx.state.get_model_name(r_ent.hModel):?}</td>
					<td>{r_ent.origin:?}</td>
				</tr>
			}
			</table></pre>
		});
	}
}
