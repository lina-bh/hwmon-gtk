use std::{
    collections::HashMap,
    hash::Hash,
    io,
    slice::Join,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{device::Device, hwmon};

pub type DeviceMap = HashMap<String, Device>;

pub enum Update {
    Tick,
    Created,
    Removed,
}

pub struct Updater {
    pub devices: Arc<Mutex<DeviceMap>>,
    tx: glib::Sender<Update>,
}

impl Updater {
    pub fn new(tx: glib::Sender<Update>) -> Result<Updater, io::Error> {
        let mut upd = Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            tx,
        };
        for dev in hwmon::modules()? {
            upd.insert_device(dev);
        }
        upd.update();
        Ok(upd)
    }

    pub fn start(mut self) -> JoinHandle<()> {
        self.update();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(2));
            self.update();
            if self.tx.send(Update::Tick).is_err() {
                return;
            }
        })
    }

    fn insert_device(&mut self, device: Device) {
        self.devices
            .lock()
            .expect("poisoning")
            .insert(device.name.clone(), device);
        self.tx.send(Update::Created).expect("rx dropped");
    }

    fn update(&mut self) {
        self.devices
            .lock()
            .expect("poisoning")
            .values_mut()
            .flat_map(|d| d.sources.iter_mut())
            .for_each(|s| s.update());
    }
}
