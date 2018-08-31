use dirs;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::usize;

const DEFAULT_CONFIG: &'static [u8] = b"\
max_stack_size = 100
show_tray_icon = false
pop_keybinding = Ctrl + Shift + C
clear_keybinding = None
swap_keybinding = None
";

pub struct Config {
   max_stack_size: usize,
   show_tray_icon: bool,
   pop_keybinding: Option<String>,
   clear_keybinding: Option<String>,
   swap_keybinding: Option<String>,
}

impl Default for Config {
   fn default() -> Config {
      Config {
         max_stack_size: 100,
         show_tray_icon: false,
         pop_keybinding: None,
         clear_keybinding: None,
         swap_keybinding: None,
      }
   }
}

pub enum LineError {
   Malformed,
   UnknownOption(String),
   UnknownModifier,
   UnknownKey,
   ExpectedBool(String),
   ExpectedInt(ParseIntError),
}

impl fmt::Display for LineError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
         LineError::Malformed => write!(
            f,
            "Line must be an option, followed by an equals sign, followed by a value."
         ),
         LineError::UnknownOption(option) => write!(f, "Unknown option `{}`", option),
         LineError::UnknownModifier => write!(f, "Unknown modifier ``"),
         LineError::UnknownKey => write!(f, "Unknown key ``"),
         LineError::ExpectedBool(got) => write!(f, "Expected value to be one of `true` or `false`, got {}", got),
         LineError::ExpectedInt(err) => write!(
            f,
            "Expected value to be a positive integer less than or equal to {}, but failed to parse: {}",
            usize::MAX,
            err
         ),
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
               "max_stack_size" => match pieces[1].trim().parse::<usize>() {
                  Ok(value) => config.max_stack_size = value,
                  Err(e) => return Err(ParseError::Line(LineError::ExpectedInt(e), i)),
               },
               "show_tray_icon" => match pieces[1].trim() {
                  "true" => {
                     config.show_tray_icon = true;
                  }
                  "false" => {
                     config.show_tray_icon = false;
                  }
                  x => return Err(ParseError::Line(LineError::ExpectedBool(x.to_owned()), i)),
               },
               "pop_keybinding" => {
                  // TODO
               }
               "clear_keybinding" => {
                  // TODO
               }
               "swap_keybinding" => {
                  // TODO
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
