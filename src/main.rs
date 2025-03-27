#![windows_subsystem = "windows"]
use crate::gui::Converter;
use relm4::RelmApp;

mod drives;
mod gui;
pub mod yt;

pub fn main() {
    let app = RelmApp::new("fr.gamermine.convertisseur");
    app.run::<Converter>(());
}
