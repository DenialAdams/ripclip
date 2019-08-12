# ripclip
[![ripclip crate](https://img.shields.io/crates/v/ripclip.svg)](https://crates.io/crates/ripclip)
## Description
Turns your windows clipboard into a stack! Everything you copy is pushed onto the top. Popping is done with a configurable hotkey (default Control + Shift + C.)
## Features
### Stack De-Duplication
If the same item is copied repeatedly, ripclip can automatically identify this and ignore the duplicate copies.
### Tray Icon
A small, unobtrusive tray icon that lets you know ripclip is still running. May be disabled in the configuration.
### Configurable
All keybindings are configurable, and extra features (above) may be disabled.
### Robust
ripclip uses an exponential backoff policy to try accessing the clipboard in case opening it fails (because it is open by another process).
### Multi-Platform
Coming soon! If you're waiting for this, file an issue and it will motivate me :)
## Configuration
Configuration is present at `C:\Users\{your_user}\AppData\Roaming\ripclip.` A default config file is written there when ripclip is first run.

The configuration options are as follows:
```
max_stack_size = 100
```
The maximum size of the stack. Any further items copied will push out the oldest (bottom) element of the stack. This can be `None` for a cliboard stack limited only by available memory.
```
show_tray_icon = true
```
Whether or not to enable the tray icon.
```
pop_keybinding = Control + Shift + C
```
They keybinding to remove the top item of the stack.
```
swap_keybinding = None
```
The keybinding to swap the top two items of the stack.
```
clear_keybinding = None
```
The keybinding to remove all items from the stack.
```
prevent_duplicate_push = false
```
Whether or not to prevent duplicate items from being pushed onto the stack.
## Known Quirks
Some terminal emulators for windows abuse (IMO) the windows clipboard by putting whatever you highlight into the clipboard, attempting to emulate common functionality in linux. That clogs up the stack, annoyingly.
## Debugging
Run with RUST_LOG=ripclip=trace as an environment variable to see debugging statements.
## Status
Pushing, popping, swapping and clearing all should work with configurable bindings. A tray icon is available. Multi-platform is a work in progress (see above.)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
