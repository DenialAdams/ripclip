use crate::api::{ErrorCode, WindowHandle};
use bitflags::bitflags;
use std::str::FromStr;
use std::{mem, ptr};

bitflags! {
   pub struct Modifiers: u32 {
      const ALT = 0x0001;
      const CONTROL = 0x0002;
      const NO_REPEAT = 0x4000;
      const SHIFT = 0x0004;
      const WIN = 0x0008;
   }
}

pub enum ParseModifierError {
   UnknownModifier(String),
}

impl FromStr for Modifiers {
   type Err = ParseModifierError;

   fn from_str(s: &str) -> Result<Modifiers, ParseModifierError> {
      let s = s.to_ascii_lowercase();
      Ok(match s.as_ref() {
         "alt" => Modifiers::ALT,
         "control" | "ctrl" => Modifiers::CONTROL,
         "shift" => Modifiers::SHIFT,
         "win" | "windows" | "super" => Modifiers::WIN,
         _ => return Err(ParseModifierError::UnknownModifier(s)),
      })
   }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VirtualKey {
   LeftClick = 0x01,
   RightClick,
   Cancel,
   MiddleClick,
   Backspace = 0x08,
   Tab,
   Enter = 0x0D,
   Shift = 0x10,
   Control,
   Alt,
   Pause,
   CapsLock,
   Escape = 0x1B,
   Space = 0x20,
   PageUp,
   PageDown,
   End,
   Home,
   Left,
   Up,
   Right,
   Down,
   Select,
   Print = 0x2A,
   Execute,
   PrintScreen,
   Insert,
   Delete,
   Help,
   Zero,
   One,
   Two,
   Three,
   Four,
   Five,
   Six,
   Seven,
   Eight,
   Nine,
   A = 0x41,
   B,
   C,
   D,
   E,
   F,
   G,
   H,
   I,
   J,
   K,
   L,
   M,
   N,
   O,
   P,
   Q,
   R,
   S,
   T,
   U,
   V,
   W,
   X,
   Y,
   Z,
   LeftWindows,
   RightWindows,
   Applications,
   Sleep = 0x5F,
   NumpadZero,
   NumpadOne,
   NumpadTwo,
   NumpadThree,
   NumpadFour,
   NumpadFive,
   NumpadSix,
   NumpadSeven,
   NumpadEight,
   NumpadNine,
   F1 = 0x70,
   F2,
   F3,
   F4,
   F5,
   F6,
   F7,
   F8,
   F9,
   F10,
   F11,
   F12,
   F13,
   F14,
   F15,
   F16,
   F17,
   F18,
   F19,
   F20,
   F21,
   F22,
   F23,
   F24, // I want to meet the person with 24 function keys
   Numlock = 0x90,
   LeftShift = 0xA0,
   RightShift,
   LeftControl,
   RightControl,
   LeftAlt,
   RightAlt,
   Play = 0xFA,
   Zoom,
}

pub enum ParseVirtualKeyError {
   UnknownKey(String),
}

impl FromStr for VirtualKey {
   type Err = ParseVirtualKeyError;

   fn from_str(s: &str) -> Result<VirtualKey, ParseVirtualKeyError> {
      let s = s.to_ascii_lowercase();
      Ok(match s.as_ref() {
         "left click" | "left button" => VirtualKey::LeftClick,
         "right click" | "right button" => VirtualKey::RightClick,
         "cancel" => VirtualKey::Cancel,
         "middle click" | "middle button" => VirtualKey::MiddleClick,
         "backspace" | "back" => VirtualKey::Backspace,
         "tab" => VirtualKey::Tab,
         "enter" => VirtualKey::Enter,
         "shift" => VirtualKey::Shift,
         "control" => VirtualKey::Control,
         "alt" => VirtualKey::Alt,
         "pause" => VirtualKey::Pause,
         "caps lock" => VirtualKey::CapsLock,
         "escape" => VirtualKey::Escape,
         "space" => VirtualKey::Space,
         "page up" => VirtualKey::PageUp,
         "page down" => VirtualKey::PageDown,
         "end" => VirtualKey::End,
         "home" => VirtualKey::Home,
         "left" => VirtualKey::Left,
         "up" => VirtualKey::Up,
         "right" => VirtualKey::Right,
         "down" => VirtualKey::Down,
         "select" => VirtualKey::Select,
         "print" => VirtualKey::Print,
         "execute" => VirtualKey::Execute,
         "print screen" => VirtualKey::PrintScreen,
         "insert" => VirtualKey::Insert,
         "delete" => VirtualKey::Delete,
         "help" => VirtualKey::Help,
         "zero" | "0" => VirtualKey::Zero,
         "one" | "1" => VirtualKey::One,
         "two" | "2" => VirtualKey::Two,
         "three" | "3" => VirtualKey::Three,
         "four" | "4" => VirtualKey::Four,
         "five" | "5" => VirtualKey::Five,
         "six" | "6" => VirtualKey::Six,
         "seven" | "7" => VirtualKey::Seven,
         "eight" | "8" => VirtualKey::Eight,
         "nine" | "9" => VirtualKey::Nine,
         "a" => VirtualKey::A,
         "b" => VirtualKey::B,
         "c" => VirtualKey::C,
         "d" => VirtualKey::D,
         "e" => VirtualKey::E,
         "f" => VirtualKey::F,
         "g" => VirtualKey::G,
         "h" => VirtualKey::H,
         "i" => VirtualKey::I,
         "j" => VirtualKey::J,
         "k" => VirtualKey::K,
         "l" => VirtualKey::L,
         "m" => VirtualKey::M,
         "n" => VirtualKey::N,
         "o" => VirtualKey::O,
         "p" => VirtualKey::P,
         "q" => VirtualKey::Q,
         "r" => VirtualKey::R,
         "s" => VirtualKey::S,
         "t" => VirtualKey::T,
         "u" => VirtualKey::U,
         "v" => VirtualKey::V,
         "w" => VirtualKey::W,
         "x" => VirtualKey::X,
         "y" => VirtualKey::Y,
         "z" => VirtualKey::Z,
         "left windows" | "left win" | "left super" => VirtualKey::LeftWindows,
         "right windows" | "right win" | "right super" => VirtualKey::RightWindows,
         "applications" | "apps" => VirtualKey::Applications,
         "sleep" => VirtualKey::Sleep,
         "numpad zero" | "numpad 0" => VirtualKey::NumpadZero,
         "numpad one" | "numpad 1" => VirtualKey::NumpadOne,
         "numpad two" | "numpad 2" => VirtualKey::NumpadTwo,
         "numpad three" | "numpad 3" => VirtualKey::NumpadThree,
         "numpad four" | "numpad 4" => VirtualKey::NumpadFour,
         "numpad five" | "numpad 5" => VirtualKey::NumpadFive,
         "numpad six" | "numpad 6" => VirtualKey::NumpadSix,
         "numpad seven" | "numpad 7" => VirtualKey::NumpadSeven,
         "numpad eight" | "numpad 8" => VirtualKey::NumpadEight,
         "numpad nine" | "numpad 9" => VirtualKey::NumpadNine,
         "f1" => VirtualKey::F1,
         "f2" => VirtualKey::F2,
         "f3" => VirtualKey::F3,
         "f4" => VirtualKey::F4,
         "f5" => VirtualKey::F5,
         "f6" => VirtualKey::F6,
         "f7" => VirtualKey::F7,
         "f8" => VirtualKey::F8,
         "f9" => VirtualKey::F9,
         "f10" => VirtualKey::F10,
         "f11" => VirtualKey::F11,
         "f12" => VirtualKey::F12,
         "f13" => VirtualKey::F13,
         "f14" => VirtualKey::F14,
         "f15" => VirtualKey::F15,
         "f16" => VirtualKey::F16,
         "f17" => VirtualKey::F17,
         "f18" => VirtualKey::F18,
         "f19" => VirtualKey::F19,
         "f20" => VirtualKey::F20,
         "f21" => VirtualKey::F21,
         "f22" => VirtualKey::F22,
         "f23" => VirtualKey::F23,
         "f24" => VirtualKey::F24,
         "numlock" | "num lock" => VirtualKey::Numlock,
         "left shift" => VirtualKey::LeftShift,
         "right shift" => VirtualKey::RightShift,
         "left control" => VirtualKey::LeftControl,
         "right control" => VirtualKey::RightControl,
         "left alt" => VirtualKey::LeftAlt,
         "right alt" => VirtualKey::RightAlt,
         "play" => VirtualKey::Play,
         "zoom" => VirtualKey::Zoom,
         _ => return Err(ParseVirtualKeyError::UnknownKey(s)),
      })
   }
}

impl VirtualKey {
   pub fn is_modifier(self) -> bool {
      self == VirtualKey::Alt
         || self == VirtualKey::Control
         || self == VirtualKey::Shift
         || self == VirtualKey::LeftWindows
         || self == VirtualKey::RightWindows
         || self == VirtualKey::LeftAlt
         || self == VirtualKey::RightAlt
         || self == VirtualKey::LeftControl
         || self == VirtualKey::RightControl
         || self == VirtualKey::LeftShift
         || self == VirtualKey::RightShift
   }
}

pub fn register_hotkey(
   hwnd: Option<&WindowHandle>,
   id: u16,
   modifiers: Modifiers,
   key: VirtualKey,
) -> Result<(), ErrorCode> {
   let result = unsafe {
      winapi::um::winuser::RegisterHotKey(
         hwnd.map_or(ptr::null_mut(), |x| x.inner.as_ptr()),
         mem::transmute::<u32, i32>(u32::from(id)),
         modifiers.bits(),
         key as u32,
      )
   };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

pub fn unregister_hotkey(hwnd: Option<&WindowHandle>, id: u16) -> Result<(), ErrorCode> {
   let result = unsafe {
      winapi::um::winuser::UnregisterHotKey(
         hwnd.map_or(ptr::null_mut(), |x| x.inner.as_ptr()),
         mem::transmute::<u32, i32>(u32::from(id)),
      )
   };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}
