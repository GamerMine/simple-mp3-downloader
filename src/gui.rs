use crate::drives::drive_mod::{Drive, DriveList};
use crate::drives::get_removable_disks;
use crate::yt::{YoutubeDownloader, DEFAULT_LIB_DIR};
use relm4::gtk::glib::{GString, clone};
use relm4::gtk::prelude::{BoxExt, ButtonExt, Cast, EditableExt, GtkWindowExt, WidgetExt};
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    adw, gtk,
};
use std::path::PathBuf;
use std::time::Duration;
use libadwaita::glib;
use libadwaita::gtk::Orientation;
use libadwaita::prelude::PreferencesGroupExt;

#[derive(Debug, Clone)]
pub enum Message {
    DriveSelection(Drive),
    LinkChanged(GString),
    Save,
    SwitchToNormal,
}

#[derive(Debug, Clone)]
pub enum CommandMessage {
    PreDownloadDone,
    UpdateCheckDone,
    DownloadFinished,
}

#[derive(PartialEq)]
enum ConverterState {
    Normal,
    WrongLink,
    PreDownloading,
    CheckingUpdate,
    Downloading,
    TransitionFromDownloadSuccess,
}

pub struct ConverterWidgets {
    device_combo: gtk::DropDown,
    link_input: adw::EntryRow,
    save_button: gtk::Button,
}

pub struct Converter {
    youtube: YoutubeDownloader,
    update_checked: bool,
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
        let youtube = YoutubeDownloader::new(PathBuf::from(DEFAULT_LIB_DIR));

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
        let pref_group = adw::PreferencesGroup::new();
        pref_group.add(&link_input);
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
        vbox.append(&pref_group);
        vbox.append(&save_button);

        let model = Converter {
            youtube,
            update_checked: false,
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
                if self.converter_state != ConverterState::TransitionFromDownloadSuccess {
                    if let Some(drive) = self.selected_drive.clone() {
                        if !self.link.is_empty()
                            && (self.link.starts_with("https://www.youtube.com/watch?v=")
                            || self.link.starts_with("https://youtube.com/watch?v="))
                        {
                            self.converter_state = ConverterState::Downloading;

                            if !self.youtube.check_prerequisites() {
                                self.converter_state = ConverterState::PreDownloading;
                                self.update_checked = true;
                                sender.oneshot_command(async {
                                    YoutubeDownloader::download_prerequisites(PathBuf::from(DEFAULT_LIB_DIR)).await;
                                    CommandMessage::PreDownloadDone
                                });
                            } else if !self.update_checked {
                                self.converter_state = ConverterState::CheckingUpdate;
                                self.update_checked = true;
                                sender.spawn_oneshot_command(move || {
                                    YoutubeDownloader::check_update().wait().unwrap();
                                    CommandMessage::UpdateCheckDone
                                });
                            } else {
                                let output_dir = drive.mount_point();
                                let mut youtube = self.youtube.clone();
                                let link = self.link.clone().to_string();

                                sender.spawn_oneshot_command(move || {
                                    youtube.download(link, &output_dir).wait().unwrap();
                                    CommandMessage::DownloadFinished
                                });
                            }
                        } else {
                            self.converter_state = ConverterState::WrongLink;
                        }
                    } else {
                        println!("TODO: Handle no drive connected or selected")
                    }
                }
            }
            Message::SwitchToNormal => self.converter_state = ConverterState::Normal
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            CommandMessage::PreDownloadDone | CommandMessage::UpdateCheckDone => {
                self.update(Message::Save, sender, root);
            }
            CommandMessage::DownloadFinished => {
                self.converter_state = ConverterState::TransitionFromDownloadSuccess;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
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
                widgets.link_input.remove_css_class("error");
            }
            ConverterState::WrongLink => {
                widgets.link_input.add_css_class("error");
            }
            ConverterState::PreDownloading => {
                self.set_button_loading_text(widgets, "Téléchargement des prérequis");
            }
            ConverterState::CheckingUpdate => {
                self.set_button_loading_text(widgets, "Vérification des mises à jour");
            }
            ConverterState::Downloading => {
                self.set_button_loading_text(widgets, "Téléchargement du MP3");
            }
            ConverterState::TransitionFromDownloadSuccess => {
                widgets.save_button.add_css_class("success");
                widgets.save_button.set_child(Some(&gtk::Label::new(Some("Succès !"))));
                widgets.save_button.set_sensitive(true);
                let cloned_btn = widgets.save_button.clone();
                let cloned_sender = sender.clone();
                glib::timeout_add_local(Duration::from_secs(2), move || {
                    cloned_btn.remove_css_class("success");
                    cloned_sender.input_sender().send(Message::SwitchToNormal).unwrap();
                    glib::ControlFlow::Break
                });
            }
        }
    }
}

impl Converter {
    fn set_button_loading_text(&self, widgets: &mut ConverterWidgets, text: &str) {
        let hbox = gtk::Box::builder().orientation(Orientation::Horizontal).spacing(5).build();
        let label = gtk::Label::new(Some(text));
        let spinner = adw::Spinner::new();

        hbox.append(&label);
        hbox.append(&spinner);

        widgets.save_button.set_child(Some(&hbox));
        widgets.save_button.set_sensitive(false);
        widgets.device_combo.set_sensitive(false);
        widgets.link_input.set_sensitive(false);
    }
}
