//! "Safe" "wrapper" around a smattering of the windows API

use std::iter;
use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use std::string::FromUtf16Error;
use std::{fmt, mem, num};
use winapi;

pub struct Menu {
   inner: NonNull<winapi::shared::windef::HMENU__>,
}

impl Drop for Menu {
   fn drop(&mut self) {
      destroy_menu(self).unwrap();
   }
}

fn destroy_menu(menu: &mut Menu) -> Result<(), ErrorCode> {
   let result = unsafe { winapi::um::winuser::DestroyMenu(menu.inner.as_ptr()) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

impl Menu {
   pub fn append_line_break(&mut self, id: usize) -> Result<(), ErrorCode> {
      let result = unsafe {
         winapi::um::winuser::AppendMenuW(
            self.inner.as_ptr(),
            winapi::um::winuser::MF_SEPARATOR,
            id,
            ptr::null_mut(),
         )
      };

      if result == 0 {
         let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
         return Err(ErrorCode(code));
      }

      Ok(())
   }

   pub fn append_text(&mut self, id: usize, text: &str) -> Result<(), ErrorCode> {
      let s = to_win_utf16(text);
      let result = unsafe {
         winapi::um::winuser::AppendMenuW(
            self.inner.as_ptr(),
            winapi::um::winuser::MF_STRING,
            id,
            s.as_ptr(),
         )
      };

      if result == 0 {
         let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
         return Err(ErrorCode(code));
      }

      Ok(())
   }
}

pub fn create_popup_menu() -> Result<Menu, ErrorCode> {
   let menu = unsafe { winapi::um::winuser::CreatePopupMenu() };

   match NonNull::new(menu) {
      Some(val) => Ok(Menu { inner: val }),
      None => {
         let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
         Err(ErrorCode(code))
      }
   }
}

pub fn draw_popup_menu(menu: &Menu, x: i32, y: i32, hwnd: &WindowHandle) -> Result<(), ErrorCode> {
   let result = unsafe {
      winapi::um::winuser::TrackPopupMenuEx(
         menu.inner.as_ptr(),
         winapi::um::winuser::TPM_RIGHTALIGN
            | winapi::um::winuser::TPM_BOTTOMALIGN
            | winapi::um::winuser::TPM_LEFTBUTTON
            | winapi::um::winuser::TPM_NOANIMATION,
         x,
         y,
         hwnd.inner.as_ptr(),
         ptr::null_mut(),
      )
   };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

pub fn set_foreground_window(hwnd: &WindowHandle) -> Result<(), ()> {
   let result = unsafe { winapi::um::winuser::SetForegroundWindow(hwnd.inner.as_ptr()) };

   if result == 0 {
      return Err(());
   }

   Ok(())
}

pub struct WindowHandle<'a> {
   inner: NonNull<winapi::shared::windef::HWND__>,
   class: PhantomData<&'a ClassAtom<'a>>,
}

impl<'a> Drop for WindowHandle<'a> {
   fn drop(&mut self) {
      destroy_window(self).unwrap();
   }
}

fn destroy_window(hwnd: &mut WindowHandle) -> Result<(), ErrorCode> {
   let result = unsafe { winapi::um::winuser::DestroyWindow(hwnd.inner.as_ptr()) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

pub struct ModuleHandle(NonNull<winapi::shared::minwindef::HINSTANCE__>);

pub struct ClassAtom<'a> {
   atom: num::NonZeroU16,
   hmodule: &'a ModuleHandle,
}

impl<'a> Drop for ClassAtom<'a> {
   fn drop(&mut self) {
      unregister_class(self).unwrap();
   }
}

fn unregister_class<'a>(class_atom: &mut ClassAtom<'a>) -> Result<(), ErrorCode> {
   let result = unsafe {
      winapi::um::winuser::UnregisterClassW(
         usize::from(class_atom.atom.get()) as *const u16,
         class_atom.hmodule.0.as_ptr(),
      )
   };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

pub struct ErrorCode(u32);

impl fmt::Display for ErrorCode {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{}", self.get_description().unwrap())
   }
}

impl fmt::Debug for ErrorCode {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{}: {}", self.0, self.get_description().unwrap())
   }
}

impl ErrorCode {
   pub fn get_description(&self) -> Result<String, FromUtf16Error> {
      let mut buffer: Box<[u16]> = vec![0; 65535].into_boxed_slice();

      let size = unsafe {
         winapi::um::winbase::FormatMessageW(
            0x0000_1000,
            ptr::null(),
            self.0,
            0,
            buffer.as_mut_ptr(),
            65535,
            ptr::null_mut(),
         )
      };

      if size == 0 {
         unimplemented!()
      }

      let utf16_slice = &buffer[0..(size - 1) as usize];

      String::from_utf16(utf16_slice)
   }
}

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
         "control" | "cntrl" | "ctrl" => Modifiers::CONTROL,
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

pub fn get_module_handle_ex() -> Result<ModuleHandle, ErrorCode> {
   let mut module_handle: winapi::shared::minwindef::HMODULE = unsafe { mem::uninitialized() };

   let result = unsafe { winapi::um::libloaderapi::GetModuleHandleExW(0, ptr::null(), &mut module_handle) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe { Ok(ModuleHandle(NonNull::new_unchecked(module_handle))) }
}

fn to_win_utf16(inp: &str) -> Vec<u16> {
   inp.encode_utf16().chain(iter::once(0)).collect()
}

pub fn register_class_ex<'a>(
   module_handle: &'a ModuleHandle,
   message_fn: winapi::um::winuser::WNDPROC,
   name: &str,
) -> Result<ClassAtom<'a>, ErrorCode> {
   let options = winapi::um::winuser::WNDCLASSEXW {
      cbSize: mem::size_of::<winapi::um::winuser::WNDCLASSEXW>() as u32,
      style: 0x0000_0000,
      lpfnWndProc: message_fn,
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: module_handle.0.as_ptr(),
      hIcon: ptr::null_mut(),
      hCursor: ptr::null_mut(),
      hbrBackground: ptr::null_mut(),
      lpszMenuName: ptr::null(),
      lpszClassName: to_win_utf16(name).as_ptr(),
      hIconSm: ptr::null_mut(),
   };

   let result = unsafe { winapi::um::winuser::RegisterClassExW(&options) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe {
      Ok(ClassAtom {
         atom: num::NonZeroU16::new_unchecked(result),
         hmodule: module_handle,
      })
   }
}

pub enum WindowParent<'a> {
   _NoParent,
   _SomeParent(&'a WindowHandle<'a>),
   MessageOnly,
}

pub fn create_window_ex<'a>(
   ex_style: u32,
   class_atom: &'a ClassAtom,
   window_style: u32,
   x: i32,
   y: i32,
   width: i32,
   height: i32,
   parent: &WindowParent,
) -> Result<WindowHandle<'a>, ErrorCode> {
   let parent_ptr = match parent {
      WindowParent::_NoParent => ptr::null_mut(),
      WindowParent::_SomeParent(handle) => handle.inner.as_ptr(),
      WindowParent::MessageOnly => winapi::um::winuser::HWND_MESSAGE,
   };

   let handle = unsafe {
      winapi::um::winuser::CreateWindowExW(
         ex_style,
         usize::from(class_atom.atom.get()) as *const u16,
         ptr::null(),
         window_style,
         x,
         y,
         width,
         height,
         parent_ptr,
         ptr::null_mut(),
         ptr::null_mut(),
         ptr::null_mut(),
      )
   };

   if handle.is_null() {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe {
      Ok(WindowHandle {
         inner: NonNull::new_unchecked(handle),
         class: PhantomData,
      })
   }
}

pub fn add_clipboard_format_listener(hwnd: &WindowHandle) -> Result<(), ErrorCode> {
   let result = unsafe { winapi::um::winuser::AddClipboardFormatListener(hwnd.inner.as_ptr()) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

pub fn remove_clipboard_format_listener(hwnd: &WindowHandle) -> Result<(), ErrorCode> {
   let success = unsafe {
      let success_int = winapi::um::winuser::RemoveClipboardFormatListener(hwnd.inner.as_ptr());
      success_int == 1
   };

   if !success {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum ClipboardFormat {
   UnicodeText = 13,
}

pub fn is_clipboard_format_available(format: ClipboardFormat) -> bool {
   unsafe { winapi::um::winuser::IsClipboardFormatAvailable(format as u32) != 0 }
}

pub struct ClipboardHandle {
   _inner: (),
}

pub struct OwnedClipboardHandle {
   _inner: (),
}

#[derive(Clone, PartialEq)]
pub struct ClipboardText(Vec<u8>);

impl ClipboardHandle {
   pub fn get_text(&self) -> Result<ClipboardText, ErrorCode> {
      let handle = unsafe { winapi::um::winuser::GetClipboardData(ClipboardFormat::UnicodeText as u32) };

      if handle.is_null() {
         let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
         return Err(ErrorCode(code));
      }

      unsafe {
         let str_len = {
            let mut index = 1;
            while *((handle as *const u8).offset(index - 1)) != 0 || *((handle as *const u8).offset(index)) != 0 {
               index += 2;
            }
            index as usize + 1
         };
         let mut buffer: Vec<u8> = Vec::with_capacity(str_len);
         buffer.set_len(str_len);

         ptr::copy(handle as *const u8, buffer.as_mut_ptr(), str_len);

         Ok(ClipboardText(buffer))
      }
   }

   // Set clipboard content
   pub fn empty(self) -> Result<OwnedClipboardHandle, ErrorCode> {
      let result = unsafe { winapi::um::winuser::EmptyClipboard() };

      if result == 0 {
         let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
         return Err(ErrorCode(code));
      }

      mem::forget(self);

      Ok(OwnedClipboardHandle { _inner: () })
   }
}

fn close_clipboard() -> Result<(), ErrorCode> {
   let result = unsafe { winapi::um::winuser::CloseClipboard() };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

impl Drop for ClipboardHandle {
   fn drop(&mut self) {
      close_clipboard().unwrap();
   }
}

impl OwnedClipboardHandle {
   pub fn set_text(&self, mut clipboard_text: ClipboardText) -> Result<(), ErrorCode> {
      let result = unsafe {
         winapi::um::winuser::SetClipboardData(
            ClipboardFormat::UnicodeText as u32,
            clipboard_text.0.as_mut_ptr() as *mut winapi::ctypes::c_void,
         )
      };

      if result.is_null() {
         let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
         return Err(ErrorCode(code));
      }

      // I'm not sure how (if) this buffer ever gets freed...
      mem::forget(clipboard_text);

      Ok(())
   }
}

impl Drop for OwnedClipboardHandle {
   fn drop(&mut self) {
      close_clipboard().unwrap();
   }
}

pub fn open_clipboard(hwnd: &WindowHandle) -> Result<ClipboardHandle, ErrorCode> {
   let result = unsafe { winapi::um::winuser::OpenClipboard(hwnd.inner.as_ptr()) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(ClipboardHandle { _inner: () })
}

pub struct Message {
   pub hwnd: Option<NonNull<winapi::shared::windef::HWND__>>,
   pub message: u32,
   pub w_param: usize,
   pub l_param: isize,
}

impl From<winapi::um::winuser::MSG> for Message {
   fn from(msg: winapi::um::winuser::MSG) -> Message {
      let window_handle = if let Some(handle) = NonNull::new(msg.hwnd) {
         Some(handle)
      } else {
         None
      };
      Message {
         hwnd: window_handle,
         message: msg.message,
         w_param: msg.wParam,
         l_param: msg.lParam,
      }
   }
}

pub fn get_message(hwnd: Option<&WindowHandle>, min_value: u32, max_value: u32) -> Result<Message, ErrorCode> {
   let mut message: winapi::um::winuser::MSG = unsafe { mem::uninitialized() };
   let result = unsafe {
      winapi::um::winuser::GetMessageW(
         &mut message,
         hwnd.map_or(ptr::null_mut(), |x| x.inner.as_ptr()),
         min_value,
         max_value,
      )
   };

   if result == -1 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(message.into())
}

pub struct TrayIcon<'a> {
   id: u32,
   hwnd: &'a WindowHandle<'a>,
}

impl<'a> Drop for TrayIcon<'a> {
   fn drop(&mut self) {
      remove_tray_icon(self).unwrap();
   }
}

fn remove_tray_icon<'a>(tray_icon: &mut TrayIcon<'a>) -> Result<(), ErrorCode> {
   let mut remove_tray_icon_options = winapi::um::shellapi::NOTIFYICONDATAW {
      cbSize: mem::size_of::<winapi::um::shellapi::NOTIFYICONDATAW>() as u32,
      hWnd: tray_icon.hwnd.inner.as_ptr(),
      uID: tray_icon.id,
      uFlags: 0,
      uCallbackMessage: 0,
      hIcon: ptr::null_mut(),
      szTip: [0; 128],
      dwState: 0,
      dwStateMask: 0,
      szInfo: [0; 256],
      u: unsafe { mem::zeroed() },
      szInfoTitle: [0; 64],
      dwInfoFlags: 0,
      guidItem: winapi::shared::guiddef::GUID {
         Data1: 0,
         Data2: 0,
         Data3: 0,
         Data4: [0; 8],
      },
      hBalloonIcon: ptr::null_mut(),
   };
   let result = unsafe {
      winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_DELETE, &mut remove_tray_icon_options)
   };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

/// Truncates any tooltip ecxeeding 254 bytes (when encoded as UTF-16)
/// (that can result in corrupted unicode... don't pass long strings!)
pub fn add_tray_icon<'a>(
   hwnd: &'a WindowHandle,
   id: u32,
   callback_id: u32,
   tooltip: &str,
   module_hande: &ModuleHandle,
) -> Result<TrayIcon<'a>, ErrorCode> {
   let icon = unsafe { winapi::um::winuser::LoadIconW(module_hande.0.as_ptr(), 1usize as *const u16) };

   if icon.is_null() {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   let mut tooltip_text: [u16; 128] = [0; 128];

   for (i, utf16_char) in tooltip.encode_utf16().enumerate().take(127) {
      tooltip_text[i] = utf16_char;
   }

   let mut nofify_icon_data_union: winapi::um::shellapi::NOTIFYICONDATAW_u = unsafe { mem::zeroed() };
   unsafe {
      *nofify_icon_data_union.uVersion_mut() = winapi::um::shellapi::NOTIFYICON_VERSION_4;
   }

   let mut add_tray_icon_options = winapi::um::shellapi::NOTIFYICONDATAW {
      cbSize: mem::size_of::<winapi::um::shellapi::NOTIFYICONDATAW>() as u32,
      hWnd: hwnd.inner.as_ptr(),
      uID: id,
      uFlags: winapi::um::shellapi::NIF_ICON
         | winapi::um::shellapi::NIF_MESSAGE
         | winapi::um::shellapi::NIF_TIP
         | winapi::um::shellapi::NIF_SHOWTIP,
      uCallbackMessage: callback_id,
      hIcon: icon,
      szTip: tooltip_text,
      dwState: 0,
      dwStateMask: 0,
      szInfo: [0; 256],
      u: nofify_icon_data_union,
      szInfoTitle: [0; 64],
      dwInfoFlags: 0,
      guidItem: winapi::shared::guiddef::GUID {
         Data1: 0,
         Data2: 0,
         Data3: 0,
         Data4: [0; 8],
      },
      hBalloonIcon: ptr::null_mut(),
   };

   let result =
      unsafe { winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_ADD, &mut add_tray_icon_options) };
   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   let result = unsafe {
      winapi::um::shellapi::Shell_NotifyIconW(winapi::um::shellapi::NIM_SETVERSION, &mut add_tray_icon_options)
   };
   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(TrayIcon { id, hwnd })
}
