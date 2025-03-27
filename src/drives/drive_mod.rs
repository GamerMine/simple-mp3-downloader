use crate::drives::drive_imp;
use relm4::gtk;
use relm4::gtk::prelude::{BoxExt, Cast, CastNone, ListItemExt, WidgetExt};
use relm4::gtk::subclass::prelude::ObjectSubclassIsExt;
use relm4::gtk::{SignalListItemFactory, gio, glib};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

glib::wrapper! {
    pub struct DriveList(ObjectSubclass<drive_imp::DriveListPriv>) @implements gio::ListModel;
}
glib::wrapper! {
    pub struct Drive(ObjectSubclass<drive_imp::DrivePriv>);
}

impl DriveList {
    pub fn new() -> DriveList {
        glib::Object::new()
    }

    pub fn from_vec(drives: &Vec<Drive>) -> DriveList {
        let dl = glib::Object::new::<DriveList>();
        let imp = dl.imp();

        let mut vec = imp.0.borrow_mut();
        for drive in drives {
            vec.push(drive.clone())
        }

        dl.clone()
    }

    pub fn create_factory() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let hbox = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(10)
                .build();

            let label = gtk::Label::new(None);

            hbox.append(&label);
            item.set_child(Some(&hbox));
        });

        factory.connect_bind(|_signal, list_item| {
            if let Some(drive) = list_item.item().and_downcast_ref::<Drive>() {
                if let Some(hbox) = list_item.child().and_downcast_ref::<gtk::Box>() {
                    let label = hbox
                        .first_child()
                        .unwrap()
                        .downcast::<gtk::Label>()
                        .unwrap();

                    label.set_label(&drive.to_string());
                }
            }
        });

        factory
    }
}

impl Default for DriveList {
    fn default() -> Self {
        Self::new()
    }
}

impl Drive {
    pub fn new(name: String, mount_point: PathBuf) -> Drive {
        glib::Object::builder()
            .property("name", name)
            .property("mount_point", mount_point)
            .build()
    }
}

impl Display for Drive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name(), self.mount_point().display())
    }
}
