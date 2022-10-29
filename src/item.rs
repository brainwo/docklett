use crate::fetch_dir;
use gtk::{
    cairo,
    gdk::{self, EventType},
    gdk_pixbuf,
    prelude::{ContainerExt, IconThemeExt, ImageExt, WidgetExt, WidgetExtManual},
    Button, Inhibit, TargetFlags,
};

type Uri = &'static str;
type MimeType = &'static str;
pub const ICON_SIZE: i32 = 32;

#[allow(dead_code)]
pub enum ItemKind {
    Bin, // Trash bin
    App, // Click to open an app
    File(Uri, MimeType),
    FileThumbnail(Uri),
    Folder(Uri),
}

pub struct Item {
    kind: ItemKind,
}

// This required by drag data
// Not sure if this is unsafe or not
// https://stackoverflow.com/questions/23975391/how-to-convert-a-string-into-a-static-str
fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl Item {
    pub fn file(path: String, mimetype: String) -> Self {
        Item {
            kind: ItemKind::File(string_to_static_str(path), string_to_static_str(mimetype)),
        }
    }

    pub fn file_thumbnail(path: String) -> Self {
        Item {
            kind: ItemKind::FileThumbnail(string_to_static_str(path)),
        }
    }

    pub fn folder(path: String) -> Self {
        Item {
            kind: ItemKind::Folder(string_to_static_str(path)),
        }
    }

    pub fn get_view(&self) -> gtk::Button {
        // Target for draging
        let targets = vec![gtk::TargetEntry::new(
            "text/uri-list",
            TargetFlags::OTHER_APP,
            0,
        )];

        //
        let uri = match &self.kind {
            ItemKind::Folder(u) => u,
            ItemKind::File(u, _) => u,
            ItemKind::FileThumbnail(u) => u,
            _ => panic!(""),
        };

        // Image for the button
        let image = gtk::Image::new();
        // Set image
        match &self.kind {
            ItemKind::FileThumbnail(_) => {
                let pixbuf =
                    gdk_pixbuf::Pixbuf::from_file_at_scale(&uri[7..], ICON_SIZE, ICON_SIZE, true)
                        .unwrap();
                image.set_from_pixbuf(Some(&pixbuf));
            }
            // Matching item type with GTK icons
            // For more info use `gtk3-icon-browser`
            itemkind => {
                let icon = match itemkind {
                    ItemKind::File(_, mime) => *mime,
                    ItemKind::Folder(_) => "folder",
                    ItemKind::Bin => "user-trash",
                    _ => "text-x-generic",
                };

                let icon_theme = gtk::IconTheme::default().unwrap();
                let result = icon_theme.load_icon(icon, ICON_SIZE, gtk::IconLookupFlags::empty());
                image.set_from_pixbuf(Some(&result.unwrap().unwrap()))
            }
        }

        let button = gtk::ButtonBuilder::new()
            .image(&image)
            .tooltip_text(&uri[7..])
            .build();

        // Apparently you can only set dragging to a `Button`
        button.drag_source_set(
            gdk::ModifierType::MODIFIER_MASK,
            &targets,
            gdk::DragAction::COPY,
        );

        button.connect_drag_data_get(|_, _, s, _, _| {
            s.set_uris(&[uri]);
        });

        // Right click popup menu
        if let ItemKind::Folder(u) = self.kind {
            button.connect_button_press_event(|_, event| {
                if event.event_type() == EventType::ButtonPress && event.button() == 3 {
                    let hbox = gtk::BoxBuilder::new()
                        .orientation(gtk::Orientation::Horizontal)
                        .build();

                    // List of shown item
                    let item_list: Vec<Item> = fetch_dir(u[7..].to_owned());
                    // Load all items to Grid

                    let dialog = gtk::WindowBuilder::new()
                        .type_(gtk::WindowType::Popup)
                        .type_hint(gdk::WindowTypeHint::Dialog)
                        .window_position(gtk::WindowPosition::Mouse)
                        .build();

                    item_list.iter().for_each(|item| hbox.add(&item.get_view()));

                    dialog.add(&hbox);
                    dialog.show_all();

                    dialog.connect_leave_notify_event(|i, e| {
                        if let (Some(device), Some(window)) = (e.device(), i.window()) {
                            if device.position().1 < window.position().0
                                || device.position().1 > window.position().0 + window.width()
                                || device.position().2 < window.position().1
                                || device.position().2 > window.position().1 + window.height()
                            {
                                unsafe {
                                    i.destroy();
                                }
                            }
                        }
                        Inhibit(false)
                    });
                }

                Inhibit(false)
            });
        }

        // Make it transparent somehow
        button.set_app_paintable(true);
        button.connect_draw(draw);
        set_visual(&button, None);

        button
    }
}

// Set visual as transparent
fn set_visual(window: &Button, _screen: Option<&gdk::Screen>) {
    if let Some(screen) = window.screen() {
        if let Some(ref visual) = screen.rgba_visual() {
            window.set_visual(Some(visual));
        }
    }
}

// Set surface with some color
fn draw(_window: &Button, ctx: &cairo::Context) -> Inhibit {
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.2);
    ctx.set_operator(cairo::Operator::Screen);
    ctx.paint().expect("Invalid cairo surface state");
    Inhibit(false)
}
