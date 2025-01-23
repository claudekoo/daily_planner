use chrono::Timelike;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SimpleTime {
    hour: u8,
    minute: u8,
    seconds: u8,
}

impl SimpleTime {
    pub fn new(hour: u8, minute: u8, seconds: u8) -> Self {
        SimpleTime {
            hour,
            minute,
            seconds,
        }
    }

    pub fn from_now() -> Self {
        let now = chrono::Local::now();
        SimpleTime {
            hour: now.hour() as u8,
            minute: now.minute() as u8,
            seconds: now.second() as u8,
        }
    }

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn as_seconds(&self) -> u32 {
        self.hour as u32 * 3600 + self.minute as u32 * 60 + self.seconds as u32
    }

}

#[derive(Serialize, Deserialize)]
pub struct Plan {
    pub id: u32,
    pub name: String,
    pub start_time: SimpleTime,
    pub end_time: SimpleTime,
    pub is_now: bool,
}

impl Plan {
    pub fn new(
        id: u32,
        name: String,
        start_time: SimpleTime,
        end_time: SimpleTime,
    ) -> Self {
        Plan {
            id,
            name,
            start_time,
            end_time,
            is_now: false,
        }
    }

    pub fn is_now(&self, current_time: SimpleTime) -> bool {
        self.start_time.as_seconds() <= current_time.as_seconds()
            && self.end_time.as_seconds() >= current_time.as_seconds()
    }

    pub fn update_now(&mut self) {
        let current_time = SimpleTime::from_now();
        self.is_now = self.is_now(current_time);
    }
}