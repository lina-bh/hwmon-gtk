// SPDX-License-Identifier: WTFPL
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

mod controller;
mod cpufreq;
mod fuse;
mod group;
mod hwmon;
mod source;

const APP_ID: &str = "uk.linabee.hwmon-gtk";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gtk::init()?;
    let ctrl = controller::ControllerBuilder::new()
        .group(hwmon::modules()?)
        .build();
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .title("hwmon-gtk")
            .child(&ctrl.view)
            .width_request(500)
            .height_request(600)
            .build();
        win.present();
    });
    app.run();
    Ok(())
}
