use crate::drives::drive_mod::{Drive, DriveList};
use libadwaita::glib::{Object, Type};
use relm4::gtk::prelude::{Cast, ObjectExt, StaticType};
use relm4::gtk::subclass::prelude::{
    DerivedObjectProperties, ListModelImpl, ObjectImpl, ObjectSubclass,
};
use relm4::gtk::{gio, glib};
use std::cell::RefCell;
use std::path::PathBuf;

#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = Drive)]
pub struct DrivePriv {
    #[property(get, set)]
    name: RefCell<String>,
    #[property(get, set)]
    mount_point: RefCell<PathBuf>,
}

#[glib::object_subclass]
impl ObjectSubclass for DrivePriv {
    const NAME: &'static str = "Drive";
    type Type = Drive;
}

#[glib::derived_properties]
impl ObjectImpl for DrivePriv {}

#[derive(Debug, Default)]
pub struct DriveListPriv(pub(super) RefCell<Vec<Drive>>);

#[glib::object_subclass]
impl ObjectSubclass for DriveListPriv {
    const NAME: &'static str = "DriveListModel";
    type Type = DriveList;
    type Interfaces = (gio::ListModel,);
}

impl ObjectImpl for DriveListPriv {}

impl ListModelImpl for DriveListPriv {
    fn item_type(&self) -> Type {
        Drive::static_type()
    }

    fn n_items(&self) -> u32 {
        self.0.borrow().len() as u32
    }

    fn item(&self, position: u32) -> Option<Object> {
        self.0
            .borrow()
            .get(position as usize)
            .map(|o| o.clone().upcast::<Object>())
    }
}
