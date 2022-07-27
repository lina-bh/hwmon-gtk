// SPDX-License-Identifier: WTFPL
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{group::Group, source::Source};

pub struct State {
    source: Box<dyn Source>,
    cur: f32,
    min: f32,
    max: f32,
    err: bool,
}

pub struct UpdateThread {
    thread: Cell<Option<JoinHandle<()>>>,
    data: Arc<Mutex<HashMap<(String, String), State>>>,
    sender: glib::Sender<()>,
}

impl UpdateThread {
    fn new(groups: Vec<Group>, sender: glib::Sender<bool>) -> UpdateThread {
        let mut map = HashMap::new();
        for group in groups.into_iter() {
            for source in group.sources.into_iter() {
                map.insert(
                    (group.name.clone(), source.name().to_owned()),
                    State {
                        source,
                        cur: 0.0,
                        min: 0.0,
                        max: 0.0,
                        err: false,
                    },
                );
            }
        }
        UpdateThread {
            thread: Cell::new(None),
            data: Arc::new(Mutex::new(map)),
        }
    }

    fn start(&self) {
        let inner_arc = self.data.clone();
        let handle = thread::spawn(move || {
            for s in inner_arc.lock().unwrap().values_mut() {
                s.err = false;
                if let Some(v) = s.source.read() {
                    if v < s.min {
                        s.min = v;
                    }
                    if s.max < v {
                        s.max = v;
                    }
                    s.cur = v;
                } else {
                    s.err = true;
                }
            }
            thread::sleep(Duration::from_secs(2));
        });
        self.thread.set(Some(handle));
    }
}
