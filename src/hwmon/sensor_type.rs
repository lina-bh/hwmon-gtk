// SPDX-License-Identifier: WTFPL
#[derive(Debug)]
pub enum SensorType {
    Current,
    Fan,
    Temperature,
    Voltage,
    Other(String),
}

impl SensorType {
    pub fn from_str(s: &str) -> SensorType {
        match s {
            "cur" => Self::Current,
            "fan" => Self::Fan,
            "temp" => Self::Temperature,
            "in" => Self::Voltage,
            s => Self::Other(s.to_owned()),
        }
    }

    pub const fn unit(&self) -> &str {
        match self {
            Self::Current => " A",
            Self::Fan => " RPM",
            Self::Temperature => "°C",
            Self::Voltage => "v",
            _ => "",
        }
    }
}
