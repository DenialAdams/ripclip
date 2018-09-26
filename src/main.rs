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

const POP_MENU_ID: usize = 100;
const SWAP_MENU_ID: usize = 101;
const CLEAR_MENU_ID: usize = 102;
const EXIT_MENU_ID: usize = 103;
const RELOAD_MENU_ID: usize = 104;

const POP_HOTKEY_ID: u16 = 1;
const SWAP_HOTKEY_ID: u16 = 2;
const CLEAR_HOTKEY_ID: u16 = 3;

fn main() {
   pretty_env_logger::init();

   let mut config = match config::load_config() {
      Ok(config) => config,
      Err(e) => {
         eprintln!("Failed to parse config: {}", e);
         std::process::exit(-1);
      }
   };

   let module = win::get_module_handle_ex().unwrap();
   let class = win::register_class_ex(&module, Some(on_message), "ripclip_class").unwrap();

   let window = win::create_window_ex(
      0x0000_0000,
      &class,
      winapi::um::winuser::WS_MINIMIZE | winapi::um::winuser::WS_DISABLED,
      0,
      0,
      0,
      0,
      &win::WindowParent::MessageOnly,
   ).unwrap();

   let mut menu = win::create_popup_menu().unwrap();
   menu.append_text(POP_MENU_ID, "Pop").unwrap();
   menu.append_text(SWAP_MENU_ID, "Swap").unwrap();
   menu.append_text(CLEAR_MENU_ID, "Clear").unwrap();
   menu.append_line_break(1).unwrap();
   menu.append_text(RELOAD_MENU_ID, "Reload Configuration").unwrap();
   menu.append_text(EXIT_MENU_ID, "Exit").unwrap();

   let mut clipboard_stack: Vec<win::ClipboardText> = if let Some(max_stack_size) = config.max_stack_size {
      Vec::with_capacity(max_stack_size)
   } else {
      Vec::new()
   };
   let mut managing_clipboard = false;

   let mut _trayicon = if config.show_tray_icon {
      Some(win::add_tray_icon(&window, 0, 100, "ripclip", &module).unwrap())
   } else {
      None
   };

   win::add_clipboard_format_listener(&window).unwrap();

   set_keybindings(&config, &window);

   loop {
      let message = win::get_message(Some(&window), 0, 0).unwrap();
      match message.message {
         winapi::um::winuser::WM_HOTKEY => match message.w_param {
            0 => {
               pop(&window, &mut clipboard_stack, &mut managing_clipboard);
            }
            1 => {
               clear(&window, &mut clipboard_stack, &mut managing_clipboard);
            }
            2 => {
               swap(&window, &mut clipboard_stack, managing_clipboard);
            }
            x => {
               warn!("Unknown hotkey {}", x);
            }
         },
         winapi::um::winuser::WM_CLIPBOARDUPDATE => {
            trace!("Clipboard updated!");
            if win::is_clipboard_format_available(win::ClipboardFormat::UnicodeText) {
               trace!("Unicode text available");
               win::remove_clipboard_format_listener(&window).unwrap();
               let clipboard_text = {
                  let clipboard = win::open_clipboard(&window).unwrap();
                  let text_buf = clipboard.get_text().unwrap();
                  let owned_clipboard = clipboard.empty().unwrap();
                  owned_clipboard.set_text(text_buf.clone()).unwrap();
                  text_buf
               };
               win::add_clipboard_format_listener(&window).unwrap();
               if config.prevent_duplicate_push && Some(&clipboard_text) == clipboard_stack.last() {
                  trace!("Ignoring push because it was a duplicate");
               } else {
                  if Some(clipboard_stack.len()) == config.max_stack_size {
                     clipboard_stack.remove(0);
                  }
                  clipboard_stack.push(clipboard_text);
                  trace!("Pushed clipboard contents onto stack");
               }
               managing_clipboard = true;
            } else {
               managing_clipboard = false;
            }
         }
         winapi::um::winuser::WM_CONTEXTMENU => {
            let x = winapi::shared::windowsx::GET_X_LPARAM(message.w_param as isize);
            let y = winapi::shared::windowsx::GET_Y_LPARAM(message.w_param as isize);
            win::set_foreground_window(&window).unwrap();
            win::draw_popup_menu(&menu, x, y, &window).unwrap();
         }
         winapi::um::winuser::WM_QUIT => {
            break;
         }
         winapi::um::winuser::WM_COMMAND => {
            if message.w_param & 0xFFFF_FFFF_0000_0000 == 0 {
               // Menu event
               match message.w_param & 0x0000_0000_FFFF_FFFF {
                  POP_MENU_ID => {
                     pop(&window, &mut clipboard_stack, &mut managing_clipboard);
                  }
                  SWAP_MENU_ID => {
                     swap(&window, &mut clipboard_stack, managing_clipboard);
                  }
                  CLEAR_MENU_ID => {
                     clear(&window, &mut clipboard_stack, &mut managing_clipboard);
                  }
                  EXIT_MENU_ID => {
                     break;
                  }
                  RELOAD_MENU_ID => {
                     match config::load_config() {
                        Ok(new_config) => {
                           if config.pop_keybinding.is_some() {
                              win::unregister_hotkey(Some(&window), POP_HOTKEY_ID).unwrap();
                           }
                           if config.swap_keybinding.is_some() {
                              win::unregister_hotkey(Some(&window), SWAP_HOTKEY_ID).unwrap();
                           }
                           if config.clear_keybinding.is_some() {
                              win::unregister_hotkey(Some(&window), CLEAR_HOTKEY_ID).unwrap();
                           }
                           config = new_config;
                           set_keybindings(&config, &window);
                           if config.show_tray_icon {
                              // note: intentionally two lines because the destructor matters!
                              _trayicon = None;
                              _trayicon = Some(win::add_tray_icon(&window, 0, 100, "ripclip", &module).unwrap());
                           } else {
                              _trayicon = None;
                           };
                           trace!("Successfully reloaded configuration");
                        }
                        Err(e) => {
                           eprintln!("Failed to parse config: {}", e);
                        }
                     };
                  }
                  _ => {
                     warn!("Unknown menu command");
                     continue;
                  }
               }
            }
         }
         _ => {
            continue;
         }
      }
   }
}

fn set_keybindings(config: &config::Config, window: &win::WindowHandle) {
   if let Some(hotkey) = &config.pop_keybinding {
      win::register_hotkey(Some(&window), POP_HOTKEY_ID, hotkey.modifiers, hotkey.key).unwrap();
   }
   if let Some(hotkey) = &config.swap_keybinding {
      win::register_hotkey(Some(&window), SWAP_HOTKEY_ID, hotkey.modifiers, hotkey.key).unwrap();
   }
   if let Some(hotkey) = &config.clear_keybinding {
      win::register_hotkey(Some(&window), CLEAR_HOTKEY_ID, hotkey.modifiers, hotkey.key).unwrap();
   }
}

fn pop(window: &win::WindowHandle, clipboard_stack: &mut Vec<win::ClipboardText>, managing_clipboard: &mut bool) {
   if *managing_clipboard {
      clipboard_stack.pop();
      trace!("Popped element off clipboard stack")
   }
   *managing_clipboard = true;

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

fn clear(window: &win::WindowHandle, clipboard_stack: &mut Vec<win::ClipboardText>, managing_clipboard: &mut bool) {
   clipboard_stack.clear();
   win::remove_clipboard_format_listener(window).unwrap();
   {
      let clipboard = win::open_clipboard(window).unwrap();
      clipboard.empty().unwrap();
   }
   win::add_clipboard_format_listener(window).unwrap();
   *managing_clipboard = true;
   trace!("Cleared stack");
}

fn swap(window: &win::WindowHandle, clipboard_stack: &mut Vec<win::ClipboardText>, managing_clipboard: bool) {
   if !managing_clipboard {
      trace!("Can't swap when the clipboard is not being managed by ripclip (clipboard contains non-text)");
      return;
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
      trace!("Swapped top 2 elements of stack");
   } else {
      trace!("Stack too small to swap");
   }
}

unsafe extern "system" fn on_message(
   handle: winapi::shared::windef::HWND,
   umsg: u32,
   w_param: usize,
   l_param: isize,
) -> winapi::shared::minwindef::LRESULT {
   let l_param_u: usize = std::mem::transmute::<_, _>(l_param);
   if (l_param_u & 0x0000_0000_0000_ffff) as u32 == winapi::um::winuser::WM_CONTEXTMENU {
      let result = winapi::um::winuser::PostMessageW(handle, winapi::um::winuser::WM_CONTEXTMENU, w_param, l_param);
      if result == 0 {
         warn!("Failed to post context menu event");
      }
   }
   winapi::um::winuser::DefWindowProcW(handle, umsg, w_param, l_param)
}
