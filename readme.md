Quake3 external cheat
=====================

An external cheat for [Quake III Arena](https://en.wikipedia.org/wiki/Quake_III_Arena) for [Quake3e](https://github.com/ec-/Quake3e/) and [CNQ3](https://www.playmorepromode.com/) engines.

Features
--------

* Aimbot
* Wallhack
* Trainer
* Debugger

Offsets
-------

The cheat runs externally alongside the game as a separate process and does not require modifications or a custom build of the game itself. This requires offsets for the game's internal structures and are specific to the engine. Offsets for mainstream engines (Quake3e and CNQ3) are included by default and if absent the cheat will try to find them automatically. However this is very likely to fail and you will need to manually provide the offsets.

Control panel
-------------

The cheat is configured via web interface, which is available [here](https://casualhacks.net/aurascope/#address=ws%3A%2F%2Flocalhost%3A30145%2Fadmin%3Ftoken%3Dadmin) by default. Run the cheat then visit the link.

Building from source
--------------------

### Windows

```
cls && cargo run --release --example=q3mod-win32
```

### Linux

Under Linux the cheat needs specific permissions, just run as root.

```
clear && cargo build --release --example=q3mod-linux && sudo ./target/release/examples/q3mod-linux
```

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
