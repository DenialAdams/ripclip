//! "Safe" "wrapper" around a smattering of the windows API

use std::iter;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::{self, NonNull};
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
         winapi::um::winuser::AppendMenuW(self.inner.as_ptr(), winapi::um::winuser::MF_STRING, id, s.as_ptr())
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
   pub(crate) inner: NonNull<winapi::shared::windef::HWND__>,
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

pub struct ErrorCode(pub(crate) u32);

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

pub fn get_module_handle_ex() -> Result<ModuleHandle, ErrorCode> {
   let mut module_handle: MaybeUninit<winapi::shared::minwindef::HMODULE> = MaybeUninit::uninit();

   let result = unsafe { winapi::um::libloaderapi::GetModuleHandleExW(0, ptr::null(), module_handle.as_mut_ptr()) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe { Ok(ModuleHandle(NonNull::new_unchecked(module_handle.assume_init()))) }
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

#[allow(clippy::too_many_arguments)] // We're roughly trying to mirror the Windows API, so the number of arguments is what it is
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
   let mut message: MaybeUninit<winapi::um::winuser::MSG> = MaybeUninit::uninit();
   let result = unsafe {
      winapi::um::winuser::GetMessageW(
         message.as_mut_ptr(),
         hwnd.map_or(ptr::null_mut(), |x| x.inner.as_ptr()),
         min_value,
         max_value,
      )
   };

   if result == -1 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe { Ok(message.assume_init().into()) }
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
