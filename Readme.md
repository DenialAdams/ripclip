# Ripclip
## Description
Turns your windows clipboard into a stack. Everything you copy is pushed onto the top. Popping is done with a configurable hotkey (default Control + Shift + C.) Check the configuration file for all options.
## Configuration
Configuration is present at C:\Users\your_user\AppData\Roaming\ripclip. A default config file is written there when clipstack is first run.
## Known Quirks
Some linux terminal emulators for windows abuse (imo) the windows clipboard by putting whatever you highlight into the clipboard, attempting to emulate common functionality in linux. That clogs up the stack, annoyingly.
## Debugging
Run with RUST_LOG=clipstack=trace as an environment variable to see debugging statements.
## Status
Pushing, popping, swapping and clearing all should work with configurable bindings. The (optional) tray icon is a WIP.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
