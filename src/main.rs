extern crate winapi;

mod win;

fn main() {
   let window = win::create_window_ex(
      0x0000_0000,
      win::register_class_ex(
         win::get_module_handle_ex().unwrap(),
         Some(on_message),
         "ClipStack_class",
      ).unwrap(),
      winapi::um::winuser::WS_MINIMIZE | winapi::um::winuser::WS_DISABLED,
      0,
      0,
      0,
      0,
      std::ptr::NonNull::new(winapi::um::winuser::HWND_MESSAGE),
   ).unwrap();

   win::add_clipboard_format_listener(window).unwrap();

   loop {
      let _message = win::get_message(
         Some(window),
         winapi::um::winuser::WM_CLIPBOARDUPDATE,
         winapi::um::winuser::WM_CLIPBOARDUPDATE,
      ).unwrap();
      println!("Clipboard updated!");
      if win::is_clipboard_format_available(win::ClipboardFormat::UnicodeText) {
         println!("Unicode text :)");
         win::remove_clipboard_format_listener(window).unwrap();
         println!("Unregistered listener");
         let clipboard = win::open_clipboard(window).unwrap();
         let mut text_buf = clipboard.get_text().unwrap();
         let owned_clipboard = clipboard.empty().unwrap();
         println!("cleared clipboard");
         println!("about to set text");
         owned_clipboard.set_text(&mut text_buf).unwrap();
         println!("set clipboard");
         win::add_clipboard_format_listener(window).unwrap();
         println!("registered listener");
      }
   }
}

unsafe extern "system" fn on_message(
   handle: winapi::shared::windef::HWND,
   umsg: u32,
   w_param: usize,
   l_param: isize,
) -> winapi::shared::minwindef::LRESULT {
   winapi::um::winuser::DefWindowProcW(handle, umsg, w_param, l_param)
}
