use super::*;

pub struct WeaponInfo {
	/// Weapon index.
	pub weapon: i32,

	/// Cooldown aimbot after firing.
	pub cooldown: f32,

	/// This weapon can use triggerbot.
	///
	/// Shotgun, Railgun.
	pub trigger: bool,

	pub projectile_speed: f32,
	pub projectile_gravity: bool,
}

impl base::solver::ProjectileWeapon for WeaponInfo {
	fn projectile_speed(&self) -> f32 {
		self.projectile_speed
	}
	fn projectile_gravity(&self) -> f32 {
		if self.projectile_gravity { sdk::DEFAULT_GRAVITY } else { 0.0 }
	}
}

pub static WEAPON_INFO: [WeaponInfo; 14] = [
	// WP_NONE
	WeaponInfo {
		weapon: 0,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_GAUNTLET
	WeaponInfo {
		weapon: 1,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_MACHINEGUN
	WeaponInfo {
		weapon: 2,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_SHOTGUN
	WeaponInfo {
		weapon: 3,
		cooldown: 1.0,
		trigger: true,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_GRENADE_LAUNCHER
	WeaponInfo {
		weapon: 4,
		cooldown: 0.8,
		trigger: false,
		projectile_speed: 700.0,
		projectile_gravity: true,
	},
	// WP_ROCKET_LAUNCHER
	WeaponInfo {
		weapon: 5,
		cooldown: 0.8,
		trigger: false,
		projectile_speed: 900.0,
		projectile_gravity: false,
	},
	// WP_LIGHTNING
	WeaponInfo {
		weapon: 6,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_RAILGUN
	WeaponInfo {
		weapon: 7,
		cooldown: 1.5,
		trigger: true,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_PLASMAGUN
	WeaponInfo {
		weapon: 8,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 2000.0,
		projectile_gravity: false,
	},
	// WP_BFG
	WeaponInfo {
		weapon: 9,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 2000.0,
		projectile_gravity: false,
	},
	// WP_GRAPPLING_HOOK
	WeaponInfo {
		weapon: 10,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
	// WP_NAILGUN
	WeaponInfo {
		weapon: 11,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 555.0,
		projectile_gravity: false,
	},
	// WP_PROX_LAUNCHER
	WeaponInfo {
		weapon: 12,
		cooldown: 0.8,
		trigger: false,
		projectile_speed: 700.0,
		projectile_gravity: true,
	},
	// WP_CHAINGUN
	WeaponInfo {
		weapon: 13,
		cooldown: 0.0,
		trigger: false,
		projectile_speed: 0.0,
		projectile_gravity: false,
	},
];
