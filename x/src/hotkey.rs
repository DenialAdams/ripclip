use bitflags::bitflags;
use std::str::FromStr;

bitflags! {
   pub struct Modifiers: u8 {
      const SHIFT = 0x01;
      const LOCK = 0x02;
      const CONTROL = 0x04;
      const MOD1 = 0x08;
      const MOD2 = 0x10;
      const MOD3 = 0x20;
      const MOD4 = 0x40;
      const MOD5 = 0x80;
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
         "control" | "ctrl" => Modifiers::CONTROL,
         "caps" | "capslock" | "lock" => Modifiers::LOCK,
         "shift" => Modifiers::SHIFT,
         "mod1" => Modifiers::MOD1,
         "mod2" => Modifiers::MOD2,
         "mod3" => Modifiers::MOD3,
         "mod4" => Modifiers::MOD4,
         "mod5" => Modifiers::MOD5,
         _ => return Err(ParseModifierError::UnknownModifier(s)),
      })
   }
}
