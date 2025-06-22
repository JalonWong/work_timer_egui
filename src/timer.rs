use std::time::Instant;
use crate::setting::TimerSetting;

#[derive(PartialEq, Clone, Copy)]
pub enum Status {
    Stopped,
    Started,
    TimeOut,
}

pub struct Timer {
    count: u64,
    start_time: Instant,
    status: Status,
    setting: Option<TimerSetting>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            status: Status::Stopped,
            count: 0,
            start_time: Instant::now(),
            setting: None,
        }
    }

    pub fn start(&mut self, setting: &TimerSetting) {
        self.setting = Some(setting.clone());
        self.start_time = Instant::now();
        self.status = Status::Started;
    }

    pub fn stop(&mut self) -> u64 {
        self.status = Status::Stopped;
        self.setting.take();
        self.count
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn update(&mut self) -> String {
        if let Some(setting) = self.setting.as_ref() {
            self.count = Instant::now().duration_since(self.start_time).as_secs();
            if self.status != Status::TimeOut && self.count >= setting.limit_time * 60 {
                self.status = Status::TimeOut;
            }

            let count = self.count;
            let minutes = count / 60;
            if minutes >= 60 {
                let hours = minutes / 60;
                format!("{:02}:{:02}:{:02}", hours, minutes % 60, count % 60)
            } else {
                format!("{:02}:{:02}", minutes, count % 60)
            }
        } else {
            "00:00".to_string()
        }
    }
}
