// build.rs

#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
   let mut res = winres::WindowsResource::new();
   res.set_icon("ripclip.ico");
   res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
