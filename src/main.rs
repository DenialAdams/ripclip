#![feature(nll)]

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

   let mut clipboard_stack: Vec<win::ClipboardText> = if let Some(max_stack_size) = config.max_stack_size {
      Vec::with_capacity(max_stack_size)
   } else {
      Vec::new()
   };
   let mut managing_clipboard = false;

   if config.show_tray_icon {
      win::add_tray_icon(window, 0).unwrap();
   }

   win::add_clipboard_format_listener(window).unwrap();
   if let Some(hotkey) = config.pop_keybinding {
      win::register_hotkey(Some(window), 0, hotkey.modifiers, hotkey.key).unwrap();
   }
   if let Some(hotkey) = config.clear_keybinding {
      win::register_hotkey(Some(window), 1, hotkey.modifiers, hotkey.key).unwrap();
   }
   if let Some(hotkey) = config.swap_keybinding {
      win::register_hotkey(Some(window), 2, hotkey.modifiers, hotkey.key).unwrap();
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
               if managing_clipboard {
                  clipboard_stack.pop();
                  trace!("Popped element off clipboard stack")
               }
               managing_clipboard = true;

               win::remove_clipboard_format_listener(window).unwrap();
               {
                  let clipboard = win::open_clipboard(window).unwrap();
                  let owned_clipboard = clipboard.empty().unwrap();
                  if let Some(text) = clipboard_stack.last() {
                     owned_clipboard.set_text(text.clone()).unwrap();
                     trace!("Placed top of stack in clipboard");
                  } else {
                     trace!("Nothing on stack to place in clipboard");
                  }
               }
               win::add_clipboard_format_listener(window).unwrap();
            }
            1 => {
               clipboard_stack.clear();
               win::remove_clipboard_format_listener(window).unwrap();
               {
                  let clipboard = win::open_clipboard(window).unwrap();
                  clipboard.empty().unwrap();
               }
               win::add_clipboard_format_listener(window).unwrap();
               managing_clipboard = true;
               trace!("Cleared stack");
            }
            2 => {
               if !managing_clipboard {
                  trace!(
                     "Can't swap when the clipboard is not being managed by clipstack (clipboard contains non-text)"
                  );
                  continue;
               }
               if clipboard_stack.len() >= 2 {
                  let second_from_top = clipboard_stack.swap_remove(clipboard_stack.len() - 2);
                  clipboard_stack.push(second_from_top);
                  win::remove_clipboard_format_listener(window).unwrap();
                  {
                     let clipboard = win::open_clipboard(window).unwrap();
                     let owned_clipboard = clipboard.empty().unwrap();
                     owned_clipboard
                        .set_text(clipboard_stack.last().unwrap().clone())
                        .unwrap();
                  }
                  win::add_clipboard_format_listener(window).unwrap();
                  trace!("Swapped top 2 elements of stack")
               } else {
                  trace!("Stack too small to swap")
               }
            }
            _ => {
               unreachable!();
            }
         },
         winapi::um::winuser::WM_CLIPBOARDUPDATE => {
            trace!("Clipboard updated!");
            if win::is_clipboard_format_available(win::ClipboardFormat::UnicodeText) {
               trace!("Unicode text available");
               win::remove_clipboard_format_listener(window).unwrap();
               let clipboard_text = {
                  let clipboard = win::open_clipboard(window).unwrap();
                  let text_buf = clipboard.get_text().unwrap();
                  let owned_clipboard = clipboard.empty().unwrap();
                  owned_clipboard.set_text(text_buf.clone()).unwrap();
                  text_buf
               };
               win::add_clipboard_format_listener(window).unwrap();
               if !config.prevent_duplicate_push || Some(&clipboard_text) != clipboard_stack.last() {
                  if Some(clipboard_stack.len()) == config.max_stack_size {
                     clipboard_stack.remove(0);
                  }
                  clipboard_stack.push(clipboard_text);
                  trace!("Pushed clipboard contents onto stack");
               } else {
                  trace!("Ignoring push because it was a duplicate");
               }
               managing_clipboard = true;
            } else {
               managing_clipboard = false;
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
