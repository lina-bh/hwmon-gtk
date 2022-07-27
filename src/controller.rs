// SPDX-License-Identifier: WTFPL
use glib::{MainContext, Sender};
use gtk::{prelude::*, CellRendererText, TreeIter, TreeStore, TreeView, TreeViewColumn};

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::group::Group;
use crate::source::Source;

const COLUMNS: &[(&str, glib::Type)] =
    &[("Name", glib::Type::STRING), ("Value", glib::Type::STRING)];

pub struct ControllerBuilder {
    groups: Vec<Group>,
}

impl ControllerBuilder {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub fn group(mut self, mut group: Vec<Group>) -> Self {
        self.groups.append(&mut group);
        self
    }

    pub fn build(self) -> Rc<Controller> {
        Controller::new(self.groups)
    }
}

struct SourceData {
    pub source: Box<dyn Source>,
    pub cur: f32,
    pub min: f32,
    pub max: f32,
    pub err: bool,
}

pub struct Controller {
    pub view: TreeView,
    model: TreeStore,
    rows: Vec<TreeIter>,
    data: Arc<Mutex<Vec<SourceData>>>,
    tx: Sender<()>,
}

impl Controller {
    fn new(groups: Vec<Group>) -> Rc<Self> {
        let store = TreeStore::new(&COLUMNS.iter().map(|c| c.1).collect::<Vec<glib::Type>>());
        let view = gtk::TreeView::builder()
            .headers_visible(true)
            .model(&store)
            .build();
        for (i, (name, _)) in COLUMNS.iter().enumerate() {
            add_column(&view, name, i as i32);
        }
        let (tx, rx) = MainContext::channel::<()>(glib::PRIORITY_DEFAULT);
        let mut ctrl = Self {
            view,
            model: store,
            rows: Vec::new(),
            tx,
            data: Arc::new(Mutex::new(Vec::new())),
        };
        for group in groups {
            ctrl.insert_group(group);
        }
        let ctrl_rc = Rc::new(ctrl);
        let rx_rc = ctrl_rc.clone();
        rx.attach(None, move |()| {
            rx_rc.update_model();
            glib::Continue(true)
        });
        ctrl_rc.start();
        ctrl_rc
    }

    fn insert_group(&mut self, group: Group) {
        let mut data = self.data.lock().unwrap();
        let group_row = self
            .model
            .insert_with_values(None, None, &[(0, &group.name.to_value())]);
        for source in group.sources.into_iter() {
            let iter = self.model.insert(Some(&group_row), -1);
            self.model.set_value(&iter, 0, &source.name().to_value());
            self.rows.push(iter);
            data.push(SourceData {
                source,
                cur: 0.,
                min: 0.,
                max: 0.,
                err: false,
            });
        }
    }

    fn start(&self) {
        self.view.expand_all();
        let data = self.data.clone();
        let sender = self.tx.clone();
        thread::spawn(move || loop {
            if let Ok(mut data) = data.lock() {
                update_data(&mut data);
            } else {
                // we panicked
                return;
            }
            if sender.send(()).is_err() {
                return;
            }
            thread::sleep(Duration::from_secs(2));
        });
    }

    fn update_model(&self) {
        let data = self.data.lock().unwrap();
        for (i, s) in data.iter().enumerate() {
            let row = self.rows.get(i).unwrap();
            let fmt = format!("{}{}", s.cur, s.source.unit());
            self.model.set_value(row, 1, &fmt.to_value());
        }
    }
}

fn update_data(data: &mut Vec<SourceData>) {
    for s in data.iter_mut() {
        if let Some(v) = s.source.read() {
            s.cur = v;
            if s.min > v {
                s.min = v;
            }
            if s.max < v {
                s.max = v;
            }
        } else {
            s.err = true;
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
