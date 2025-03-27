use crate::drives::drive_mod::{Drive, DriveList};
use crate::drives::get_removable_disks;
use crate::yt::YoutubeDownloader;
use relm4::gtk::glib::{GString, clone};
use relm4::gtk::prelude::{BoxExt, ButtonExt, Cast, EditableExt, GtkWindowExt, WidgetExt};
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    adw, gtk,
};
use std::path::PathBuf;
use relm4::gtk::gdk;

#[derive(Debug, Clone)]
pub enum Message {
    DriveSelection(Drive),
    LinkChanged(GString),
    Save,
}

#[derive(Debug, Clone)]
pub enum CommandMessage {
    DownloadDone,
}

enum ConverterState {
    Normal,
    WrongLink,
    Downloading,
}

pub struct ConverterWidgets {
    device_combo: gtk::DropDown,
    link_input: adw::EntryRow,
    save_button: gtk::Button,
}

pub struct Converter {
    youtube: YoutubeDownloader,
    selected_drive: Option<Drive>,
    link: GString,
    converter_state: ConverterState
}

impl Component for Converter {
    type CommandOutput = CommandMessage;
    type Input = Message;
    type Output = ();
    type Init = ();
    type Root = gtk::Window;
    type Widgets = ConverterWidgets;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("Convertisseur MP3")
            .default_width(480)
            .default_height(160)
            .resizable(false)
            .build()
    }

    fn init(
        _model: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let drives = get_removable_disks();
        let selected_drive = {
            if drives.is_empty() {
                None
            } else {
                Some(drives[0].clone())
            }
        };
        let youtube = YoutubeDownloader::new(PathBuf::from("libs"));

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();
        vbox.set_margin_all(10);

        let drive_list = DriveList::from_vec(&drives);
        let device_combo = gtk::DropDown::builder()
            .factory(&DriveList::create_factory())
            .model(&drive_list)
            .list_factory(&DriveList::create_factory())
            .build();

        let link_input = adw::EntryRow::builder().title("Lien Youtube").build();
        let save_button_content = adw::ButtonContent::builder()
            .label("Enregistrer")
            .icon_name("document-save")
            .build();
        let save_button = gtk::Button::builder()
            .child(&save_button_content)
            .hexpand(false)
            .halign(gtk::Align::Center)
            .build();

        device_combo.connect_selected_item_notify(clone!(
            #[strong]
            sender,
            move |e| {
                sender.input(Message::DriveSelection(
                    e.selected_item().unwrap().downcast::<Drive>().unwrap(),
                ))
            }
        ));

        link_input.connect_changed(clone!(
            #[strong]
            sender,
            move |e| sender.input(Message::LinkChanged(e.text()))
        ));

        save_button.connect_clicked(clone!(
            #[strong]
            sender,
            move |_| {
                sender.input(Message::Save);
            }
        ));

        window.set_child(Some(&vbox));
        vbox.append(&device_combo);
        vbox.append(&link_input);
        vbox.append(&save_button);

        let model = Converter {
            youtube,
            selected_drive,
            link: GString::new(),
            converter_state: ConverterState::Normal
        };

        let widgets = ConverterWidgets {
            device_combo,
            link_input,
            save_button,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Message::DriveSelection(drive) => {
                self.selected_drive = Some(drive);
            }
            Message::LinkChanged(link) => {
                self.link = link;
                self.converter_state = ConverterState::Normal;
            }
            Message::Save => {
                if let Some(drive) = self.selected_drive.clone() {
                    if !self.link.is_empty()
                        && (self.link.starts_with("https://www.youtube.com/watch?v=")
                            || self.link.starts_with("https://youtube.com/watch?v="))
                    {
                        self.converter_state = ConverterState::Downloading;

                        let output_dir = drive.mount_point();
                        let mut youtube = self.youtube.clone();
                        let link = self.link.clone().to_string();

                        sender.spawn_oneshot_command(move || {
                            youtube.download(link, &output_dir).wait().unwrap();
                            CommandMessage::DownloadDone
                        });
                    } else {
                        self.converter_state = ConverterState::WrongLink;
                    }
                } else {
                    println!("TODO: Handle no drive connected or selected")
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandMessage::DownloadDone => {
                self.converter_state = ConverterState::Normal;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        match self.converter_state {
            ConverterState::Normal => {
                let save_button_content = adw::ButtonContent::builder()
                    .label("Enregistrer")
                    .icon_name("document-save")
                    .build();
                widgets.save_button.set_child(Some(&save_button_content));
                widgets.save_button.set_sensitive(true);
                widgets.device_combo.set_sensitive(true);
                widgets.link_input.set_sensitive(true);
                widgets.link_input.remove_css_class("custom_entry");
            }
            ConverterState::WrongLink => {
                widgets.link_input.add_css_class("custom_entry");
                let provider = gtk::CssProvider::new();
                provider.load_from_data(".custom_entry { background-color: #c01c28; }");
                if let Some(display) = gdk::Display::default() {
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }
            }
            ConverterState::Downloading => {
                let spinner = adw::Spinner::new();
                widgets.save_button.set_child(Some(&spinner));
                widgets.save_button.set_sensitive(false);
                widgets.device_combo.set_sensitive(false);
                widgets.link_input.set_sensitive(false);
            }
        }
    }
}
