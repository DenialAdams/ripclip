//! "Safe" "wrapper" around the windows clipboard API

use std::ptr::{self, NonNull};
use std::string::FromUtf16Error;
use std::{fmt, mem, num};
use winapi;

pub type WindowHandle = NonNull<winapi::shared::windef::HWND__>;

pub type ModuleHandle = NonNull<winapi::shared::minwindef::HINSTANCE__>;

#[derive(Clone, Copy)]
pub struct ClassAtom(num::NonZeroU16);

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

pub fn get_module_handle_ex() -> Result<ModuleHandle, ErrorCode> {
   let mut module_handle: winapi::shared::minwindef::HMODULE = unsafe { mem::uninitialized() };

   let result = unsafe { winapi::um::libloaderapi::GetModuleHandleExW(0, ptr::null(), &mut module_handle) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe { Ok(NonNull::new_unchecked(module_handle)) }
}

pub fn register_class_ex(
   module_handle: ModuleHandle,
   message_fn: winapi::um::winuser::WNDPROC,
   name: &str,
) -> Result<ClassAtom, ErrorCode> {
   let mut utf16_name: Vec<u16> = name.encode_utf16().collect();
   utf16_name.push(0);

   let options = winapi::um::winuser::WNDCLASSEXW {
      cbSize: mem::size_of::<winapi::um::winuser::WNDCLASSEXW>() as u32,
      style: 0x0000_0000,
      lpfnWndProc: message_fn,
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: module_handle.as_ptr(),
      hIcon: ptr::null_mut(),
      hCursor: ptr::null_mut(),
      hbrBackground: ptr::null_mut(),
      lpszMenuName: ptr::null(),
      lpszClassName: utf16_name.as_ptr(),
      hIconSm: ptr::null_mut(),
   };

   let result = unsafe { winapi::um::winuser::RegisterClassExW(&options) };

   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe { Ok(ClassAtom(num::NonZeroU16::new_unchecked(result))) }
}

#[allow(too_many_arguments)] // Roughly mirroring the windows API, can't blame me for argument count
pub fn create_window_ex(
   ex_style: u32,
   class_atom: ClassAtom,
   window_style: u32,
   x: i32,
   y: i32,
   width: i32,
   height: i32,
   parent: Option<WindowHandle>,
) -> Result<WindowHandle, ErrorCode> {
   let handle = unsafe {
      winapi::um::winuser::CreateWindowExW(
         ex_style,
         class_atom.0.get() as usize as *const u16,
         ptr::null(),
         window_style,
         x,
         y,
         width,
         height,
         parent.map_or(ptr::null_mut(), |x| x.as_ptr()),
         ptr::null_mut(),
         ptr::null_mut(),
         ptr::null_mut(),
      )
   };

   if handle.is_null() {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   unsafe { Ok(NonNull::new_unchecked(handle)) }
}

pub fn add_clipboard_format_listener(hwnd: WindowHandle) -> Result<(), ErrorCode> {
   let success = unsafe {
      let success_int = winapi::um::winuser::AddClipboardFormatListener(hwnd.as_ptr());
      success_int == 1
   };

   if !success {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(())
}

pub fn remove_clipboard_format_listener(hwnd: WindowHandle) -> Result<(), ErrorCode> {
   let success = unsafe {
      let success_int = winapi::um::winuser::RemoveClipboardFormatListener(hwnd.as_ptr());
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

#[derive(Clone)]
pub struct ClipboardHandle {
   _inner: (),
}

#[derive(Clone)]
pub struct OwnedClipboardHandle {
   _inner: (),
}

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
         println!("{}", str_len);
         let mut buffer: Vec<u8> = Vec::with_capacity(str_len);
         buffer.set_len(str_len);

         ptr::copy(handle as *const u8, buffer.as_mut_ptr(), str_len);

         println!("{:?}", buffer);

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

pub fn open_clipboard(hwnd: WindowHandle) -> Result<ClipboardHandle, ErrorCode> {
   let result = unsafe { winapi::um::winuser::OpenClipboard(hwnd.as_ptr()) };
   if result == 0 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   Ok(ClipboardHandle { _inner: () })
}

pub struct Message {
   pub hwnd: Option<WindowHandle>,
   pub message: u32,
   pub w_param: usize,
   pub l_param: isize,
}

impl From<winapi::um::winuser::MSG> for Message {
   fn from(msg: winapi::um::winuser::MSG) -> Message {
      Message {
         hwnd: WindowHandle::new(msg.hwnd),
         message: msg.message,
         w_param: msg.wParam,
         l_param: msg.lParam,
      }
   }
}

pub fn get_message(hwnd: Option<WindowHandle>, min_value: u32, max_value: u32) -> Result<Message, ErrorCode> {
   let mut message: winapi::um::winuser::MSG = unsafe { mem::uninitialized() };
   let result = unsafe {
      winapi::um::winuser::GetMessageW(
         &mut message,
         hwnd.map_or(ptr::null_mut(), |x| x.as_ptr()),
         min_value,
         max_value,
      )
   };

   if result == -1 {
      let code = unsafe { winapi::um::errhandlingapi::GetLastError() };
      return Err(ErrorCode(code));
   }

   if result == 0 {
      unimplemented!();
   }

   Ok(message.into())
}
