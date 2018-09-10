use dirs;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::usize;
use win;

const DEFAULT_CONFIG: &[u8] = b"\
max_stack_size = 100
show_tray_icon = true
pop_keybinding = Control + Shift + C
clear_keybinding = None
swap_keybinding = None
prevent_duplicate_push = false
";

pub struct Config {
   pub max_stack_size: Option<usize>,
   pub show_tray_icon: bool,
   pub pop_keybinding: Option<Hotkey>,
   pub clear_keybinding: Option<Hotkey>,
   pub swap_keybinding: Option<Hotkey>,
   pub prevent_duplicate_push: bool,
}

impl Default for Config {
   fn default() -> Config {
      Config {
         max_stack_size: Some(100),
         show_tray_icon: false,
         pop_keybinding: None,
         clear_keybinding: None,
         swap_keybinding: None,
         prevent_duplicate_push: false,
      }
   }
}

pub enum LineError {
   Malformed,
   UnknownOption(String),
   UnknownModifier(String),
   UnknownKey(String),
   ExpectedBool(String),
   ExpectedInt(ParseIntError),
   ModifierWithNoKey,
}

impl fmt::Display for LineError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
         LineError::Malformed => write!(
            f,
            "Line must be an option, followed by an equals sign, followed by a value."
         ),
         LineError::UnknownOption(got) => write!(f, "Unknown option `{}`", got),
         LineError::UnknownModifier(got) => write!(f, "Unknown modifier `{}`", got),
         LineError::UnknownKey(got) => write!(f, "Unknown key `{}`", got),
         LineError::ExpectedBool(got) => write!(f, "Expected value to be one of `true` or `false`, got {}", got),
         LineError::ExpectedInt(err) => write!(
            f,
            "Expected value to be a positive integer less than or equal to {}, but failed to parse: {}",
            usize::MAX,
            err
         ),
         LineError::ModifierWithNoKey => write!(
            f,
            "It doesn't make sense to have an empty key (None) with any modifiers, or other tokens"
         ),
      }
   }
}

impl From<win::ParseVirtualKeyError> for LineError {
   fn from(e: win::ParseVirtualKeyError) -> LineError {
      match e {
         win::ParseVirtualKeyError::UnknownKey(got) => LineError::UnknownKey(got),
      }
   }
}

impl From<win::ParseModifierError> for LineError {
   fn from(e: win::ParseModifierError) -> LineError {
      match e {
         win::ParseModifierError::UnknownModifier(got) => LineError::UnknownModifier(got),
      }
   }
}

pub enum ParseError {
   Io(io::Error),
   Line(LineError, usize),
}

impl From<io::Error> for ParseError {
   fn from(e: io::Error) -> ParseError {
      ParseError::Io(e)
   }
}

impl fmt::Display for ParseError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
         ParseError::Io(e) => write!(f, "I/O Error: {}", e),
         ParseError::Line(e, index) => write!(f, "Error at line {}: {}", index + 1, e),
      }
   }
}

pub struct Hotkey {
   pub key: win::VirtualKey,
   pub modifiers: win::Modifiers,
}

fn parse_hotkey(hotkey: &str) -> Result<Option<Hotkey>, LineError> {
   let mut tokens_iter = hotkey.split('+').rev();
   let raw_key = tokens_iter.next().unwrap().trim();
   if raw_key == "None" {
      if tokens_iter.next() != None {
         return Err(LineError::ModifierWithNoKey);
      }
      return Ok(None);
   }
   let key: win::VirtualKey = raw_key.parse()?;
   if key.is_modifier() {
      warn!(
         "Encountered a modifier key `{}` in key position while parsing hotkey. Is this intended?",
         raw_key
      );
   }
   let mut modifiers = win::Modifiers::empty();
   for modifier in tokens_iter {
      let modifier: win::Modifiers = modifier.trim().parse()?;
      modifiers |= modifier;
   }
   Ok(Some(Hotkey { key, modifiers }))
}

pub fn load_config() -> Result<Config, ParseError> {
   let path_opt = dirs::config_dir();
   if let Some(mut path) = path_opt {
      path.push(PathBuf::from("Clipstack"));
      // Maybe it already exists, maybe not.
      // We ignore errors because it will be handled when we try to
      // write/read the configuration
      let _ = fs::create_dir(&path);
      path.push(PathBuf::from("clipstack.conf"));
      if let Ok(file) = File::open(&path) {
         let mut config = Config::default();
         for (i, line) in BufReader::new(file).lines().enumerate() {
            let line = line?;
            let pieces: Vec<_> = line.split('=').collect();
            if pieces.len() != 2 {
               return Err(ParseError::Line(LineError::Malformed, i));
            }
            match pieces[0].trim() {
               "max_stack_size" => {
                  let opt_value = pieces[1].trim();
                  config.max_stack_size = if opt_value == "None" {
                     None
                  } else {
                     match opt_value.parse::<usize>() {
                        Ok(value) => Some(value),
                        Err(e) => return Err(ParseError::Line(LineError::ExpectedInt(e), i)),
                     }
                  }
               }
               "show_tray_icon" => match pieces[1].trim() {
                  "true" => {
                     config.show_tray_icon = true;
                  }
                  "false" => {
                     config.show_tray_icon = false;
                  }
                  x => return Err(ParseError::Line(LineError::ExpectedBool(x.to_owned()), i)),
               },
               "prevent_duplicate_push" => match pieces[1].trim() {
                  "true" => {
                     config.prevent_duplicate_push = true;
                  }
                  "false" => {
                     config.prevent_duplicate_push = false;
                  }
                  x => return Err(ParseError::Line(LineError::ExpectedBool(x.to_owned()), i)),
               },
               "pop_keybinding" => {
                  config.pop_keybinding = match parse_hotkey(pieces[1].trim()) {
                     Ok(binding) => binding,
                     Err(e) => return Err(ParseError::Line(e, i)),
                  }
               }
               "clear_keybinding" => {
                  config.clear_keybinding = match parse_hotkey(pieces[1].trim()) {
                     Ok(binding) => binding,
                     Err(e) => return Err(ParseError::Line(e, i)),
                  }
               }
               "swap_keybinding" => {
                  config.swap_keybinding = match parse_hotkey(pieces[1].trim()) {
                     Ok(binding) => binding,
                     Err(e) => return Err(ParseError::Line(e, i)),
                  }
               }
               x => return Err(ParseError::Line(LineError::UnknownOption(x.to_owned()), i)),
            }
         }
         info!("Read configuration from {:#?}", path);
         Ok(config)
      } else {
         match File::create(&path) {
            Ok(mut file) => {
               if let Err(e) = file.write_all(DEFAULT_CONFIG) {
                  warn!("Unable to write default configuration to {:#?}.\n Error: {}", path, e);
               } else {
                  info!("Wrote default configuration to {:#?}", path);
               }
            }
            Err(e) => {
               warn!("Unable to write default configuration to {:#?}.\n Error: {}", path, e);
            }
         }
         Ok(Config::default())
      }
   } else {
      warn!("Unable to determine configuration directory; Falling back to default");
      Ok(Config::default())
   }
}
