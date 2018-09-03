#[macro_use]
extern crate bitflags;
extern crate dirs;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate winapi;

mod config;
mod win;

fn main() {
   pretty_env_logger::init();

   let config = match config::load_config() {
      Ok(config) => config,
      Err(e) => {
         eprintln!("Failed to parse config: {}", e);
         std::process::exit(-1);
      }
   };

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
   if let Some(hotkey) = config.pop_keybinding {
      win::register_hotkey(Some(window), 0, hotkey.modifiers, hotkey.key).unwrap();
   }
   if let Some(hotkey) = config.clear_keybinding {
      win::register_hotkey(Some(window), 1, hotkey.modifiers, hotkey.key).unwrap();
   }
   if let Some(hotkey) = config.swap_keybinding {
      win::register_hotkey(Some(window), 1, hotkey.modifiers, hotkey.key).unwrap();
   }

   loop {
      let message = win::get_message(
         Some(window),
         winapi::um::winuser::WM_HOTKEY,
         winapi::um::winuser::WM_CLIPBOARDUPDATE,
      ).unwrap();
      match message.message {
         winapi::um::winuser::WM_HOTKEY => match message.w_param {
            0 => {
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
            1 => {
               clipboard_stack.clear();
               win::remove_clipboard_format_listener(window).unwrap();
               {
                  let clipboard = win::open_clipboard(window).unwrap();
                  clipboard.empty().unwrap();
               }
               win::add_clipboard_format_listener(window).unwrap();
               println!("Cleared stack");
            }
            2 => {
               // SWAP TODO
               unimplemented!();
            }
            _ => {
               unreachable!();
            }
         },
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
