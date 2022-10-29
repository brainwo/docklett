use crate::{item::Item, mime::get_mimetypes};
use std::ffi::OsStr;
use std::fs;

mod item;
mod mime;

use gtk::{
    cairo,
    gdk::{self, Event, EventMask, EventType},
    gio,
    prelude::*,
    ApplicationWindow, DestDefaults, SelectionData, TargetFlags, WindowType,
};
use item::ICON_SIZE;

const APPLICATION_ID: &str = "io.github.brainwo.docklett";
const TITLE: &str = "Docklett";
const DEFAULT_WIDTH: i32 = 0; // Follows the HBox width
const DEFAULT_HEIGHT: i32 = ICON_SIZE + 20;

fn main() {
    let application = gtk::Application::new(Some(APPLICATION_ID), Default::default());

    application.connect_activate(build_ui);
    application.run();
}

// TODO: A better Result managemenet
fn load_data() -> String {
    let home_dir = home::home_dir().unwrap();
    let config_dir = &format!("{}/.config/docklett.config", home_dir.to_str().unwrap());
    let root_path = match fs::read_to_string(config_dir) {
        Ok(string) => string.trim().to_string(),
        Err(_) => {
            fs::write(config_dir, home_dir.to_str().unwrap()).unwrap();
            home_dir.to_str().unwrap().to_string()
        }
    };
    root_path
}

fn fetch_dir(root_path: String) -> Vec<Item> {
    fs::read_dir(root_path)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.file_name().unwrap().to_string_lossy().starts_with('.') {
                return None;
            }
            if path.is_dir() {
                Some(Item::folder(format!("file://{}", path.to_string_lossy())))
            } else {
                match path.extension().and_then(OsStr::to_str) {
                    Some("jpg") | Some("png") => Some(Item::file_thumbnail(format!(
                        "file://{}",
                        path.to_string_lossy()
                    ))),
                    Some(extension) => Some(Item::file(
                        format!("file://{}", path.to_string_lossy()),
                        get_mimetypes(extension).to_string(),
                    )),
                    None => Some(Item::file(
                        format!("file://{}", path.to_string_lossy()),
                        String::from("text-x-generic"),
                    )),
                }
            }
        })
        .collect()
}

fn build_ui(application: &gtk::Application) {
    let root_path = load_data();

    // Horizontal Box to contain items
    let hbox = gtk::BoxBuilder::new()
        .orientation(gtk::Orientation::Horizontal)
        .margin(8)
        .build();

    let window = gtk::ApplicationWindowBuilder::new()
        .application(application)
        .type_(WindowType::Toplevel)
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .type_hint(gdk::WindowTypeHint::Dock)
        .app_paintable(true)
        .title(TITLE)
        .focus_visible(true)
        .child(&hbox)
        .build();

    // List of shown item
    let item_list: Vec<Item> = fetch_dir(root_path);

    // Load all items to Hbox
    item_list.iter().for_each(|item| hbox.add(&item.get_view()));

    // Use screen width, screen height
    if let Some(screen) = window.screen() {
        window.move_((screen.width() / 2) - 100, 1080 - DEFAULT_HEIGHT);
    }
    window.connect_draw(draw);
    set_visual(&window, None);

    let targets = vec![gtk::TargetEntry::new(
        "text/uri-list",
        TargetFlags::OTHER_APP,
        0,
    )];

    // TODO: Hide and show
    window.add_events(EventMask::POINTER_MOTION_MASK);
    window.add_events(EventMask::LEAVE_NOTIFY_MASK);
    window.add_events(EventMask::PROXIMITY_IN_MASK);
    window.add_events(EventMask::PROXIMITY_OUT_MASK);

    window.set_opacity(0.);

    window.connect_visible_notify(|i| {
        if let (Some(screen), Some(window)) = (i.screen(), i.window()) {
            i.move_(
                screen.width() / 2 - window.width() / 2,
                screen.height() - DEFAULT_HEIGHT,
            );
        }
    });

    window.connect_event(|i, e: &Event| -> Inhibit {
        if e.event_type() == EventType::LeaveNotify {
            if let (Some(mouse), Some(screen), Some(window)) = (e.device(), i.screen(), i.window())
            {
                if mouse.position().2 < screen.height() - DEFAULT_HEIGHT
                    || mouse.position().1 < screen.width() / 2 - window.width() / 2
                    || mouse.position().1 > screen.width() / 2 + window.width() / 2
                {
                    i.set_opacity(0.)
                }
            }
        }
        if e.event_type() == EventType::MotionNotify || e.event_type() == EventType::EnterNotify {
            i.set_opacity(1.);
        }

        Inhibit(false)
    });
    // TODO: Dropping a new file doesn't work
    window.drag_dest_set(DestDefaults::DROP, &targets, gdk::DragAction::COPY);

    window.connect_drag_data_received(|_, _, _, _, d: &SelectionData, _, _| {
        let mut buffer = String::new();
        println!("{:?}", d.uris());

        for file in d.uris() {
            let file = gio::File::for_uri(&file);
            let display_name = if file.is_native() {
                file.path().unwrap().display().to_string()
            } else {
                file.uri().into()
            };
            let bulleted_file_path = format!(" • {}\n", &display_name);
            buffer.push_str(bulleted_file_path.trim());
            println!("{}", bulleted_file_path);
        }
    });

    window.show_all();
}

// Set visual as transparent
fn set_visual(window: &ApplicationWindow, _screen: Option<&gdk::Screen>) {
    if let Some(screen) = window.screen() {
        if let Some(ref visual) = screen.rgba_visual() {
            window.set_visual(Some(visual));
        }
    }
}

// Set surface with some color
fn draw(_window: &ApplicationWindow, ctx: &cairo::Context) -> Inhibit {
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.2);
    ctx.set_operator(cairo::Operator::Screen);
    ctx.paint().expect("Invalid cairo surface state");
    Inhibit(false)
}
