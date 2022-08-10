use glib::MainContext;
// SPDX-License-Identifier: WTFPL
use gtk::{prelude::*, ScrolledWindow};
use gtk::{Application, ApplicationWindow};
use update::Updater;

mod controller;
// mod cpufreq;
mod device;
mod hwmon;
mod source;
mod update;

const APP_ID: &str = "uk.linabee.hwmon-gtk";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gtk::init()?;
    let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);
    let upd = Updater::new(tx)?;
    let ctrl = controller::ControllerBuilder::new()
        .group(hwmon::modules()?)
        .build();
    let chld = ScrolledWindow::builder().child(&ctrl.view).build();
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .title("hwmon-gtk")
            .child(&chld)
            .width_request(500)
            .height_request(600)
            .build();
        win.present();
    });
    app.run();
    Ok(())
}
