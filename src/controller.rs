// SPDX-License-Identifier: WTFPL
use crate::source::Input;
use crate::update::{self, DeviceMap};
use crate::{device::Device, update::Update};
use glib::{MainContext, Sender};
use gtk::{prelude::*, CellRendererText, TreeIter, TreeStore, TreeView, TreeViewColumn};
use std::{
    collections::HashMap,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const COLUMNS: &[(&str, glib::Type)] =
    &[("Name", glib::Type::STRING), ("Value", glib::Type::STRING)];

pub struct Controller {
    pub view: TreeView,
    model: TreeStore,
    devices_rows: HashMap<String, (TreeIter, HashMap<String, TreeIter>)>,
    data: Arc<Mutex<DeviceMap>>,
}

impl Controller {
    fn new(rx: glib::Receiver<Update>, data: Arc<Mutex<DeviceMap>>) -> Rc<Self> {
        let model = TreeStore::new(&COLUMNS.iter().map(|c| c.1).collect::<Vec<glib::Type>>());
        let view = gtk::TreeView::builder()
            .headers_visible(true)
            .model(&model)
            .build();
        for (i, (name, _)) in COLUMNS.iter().enumerate() {
            add_column(&view, name, i as i32);
        }
        let ctrl = Self {
            view,
            model,
            devices_rows: HashMap::new(),
            data,
        };

        let ctrl_rc = Rc::new(ctrl);
        let rx_rc = ctrl_rc.clone();
        rx.attach(None, move |msg| {
            match msg {
                Created | Tick => {}
            };
            glib::Continue(true)
        });
        ctrl_rc.start();
        ctrl_rc
    }

    fn insert_group(&mut self, group: &Device) {
        let mut data = self.data.lock().unwrap();
        let group_row = self
            .model
            .insert_with_values(None, None, &[(0, &group.name.to_value())]);
        let rows_map = HashMap::new();
        for source in group.sources.into_iter() {
            let iter = self.model.insert(Some(&group_row), -1);
            rows_map.insert(source.input.name().to_string(), iter);
        }
        self.devices_rows
            .insert(group.name.clone(), (group_row, rows_map));
    }

    fn update_model(&self, data: &DeviceMap) {
        for dev in data.values() {
            let (device_row, source_rows) = match self.devices_rows.get(&dev.name) {
                None => {
                    let iter =
                        self.model
                            .insert_with_values(None, None, &[(0, &dev.name.to_value())]);
                    let val = (iter, HashMap::new());
                    self.devices_rows.insert(dev.name.clone(), val);
                    &val
                }
                Some(t) => t,
            };
            for source in dev.sources.iter() {
                let source_row = source_rows.get(&source.input.name());
            }
        }
    }
}

fn add_column(widget: &gtk::TreeView, name: &str, idx: i32) {
    let renderer = CellRendererText::new();
    let col = TreeViewColumn::new();
    col.pack_start(&renderer, true);
    col.add_attribute(&renderer, "text", idx);
    col.set_title(name);
    widget.append_column(&col);
}
