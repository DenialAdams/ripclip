#[macro_use]
extern crate bitflags;
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
      Some(win::MESSAGE_PARENT),
   ).unwrap();

   let mut clipboard_stack: Vec<win::ClipboardText> = Vec::new();

   win::add_clipboard_format_listener(window).unwrap();
   win::register_hotkey(Some(window), 0, win::Modifiers::SHIFT, 0x43).unwrap();

   loop {
      let message = win::get_message(
         Some(window),
         winapi::um::winuser::WM_HOTKEY,
         winapi::um::winuser::WM_CLIPBOARDUPDATE,
      ).unwrap();
      match message.message {
         winapi::um::winuser::WM_HOTKEY => {
            clipboard_stack.pop();
            win::remove_clipboard_format_listener(window).unwrap();
            {
               let clipboard = win::open_clipboard(window).unwrap();
               let owned_clipboard = clipboard.empty().unwrap();
               if let Some(text) = clipboard_stack.last() {
                  owned_clipboard.set_text(text.clone()).unwrap();
               }
            }
            win::add_clipboard_format_listener(window).unwrap();
            println!("Removed top of stack");
         }
         winapi::um::winuser::WM_CLIPBOARDUPDATE => {
            println!("Clipboard updated!");
            if win::is_clipboard_format_available(win::ClipboardFormat::UnicodeText) {
               println!("Unicode text :)");
               win::remove_clipboard_format_listener(window).unwrap();
               let clipboard_text = {
                  let clipboard = win::open_clipboard(window).unwrap();
                  let text_buf = clipboard.get_text().unwrap();
                  let owned_clipboard = clipboard.empty().unwrap();
                  owned_clipboard.set_text(text_buf.clone()).unwrap();
                  text_buf
               };
               win::add_clipboard_format_listener(window).unwrap();
               clipboard_stack.push(clipboard_text);
            }
         }
         _ => {
            continue;
         }
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
