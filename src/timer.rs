use crate::setting::TimerSetting;
use std::time::{Instant, SystemTime};

#[derive(PartialEq, Clone, Copy)]
pub enum Status {
    Stopped,
    Started,
    TimeOut,
}

pub struct Timer {
    count: u64,
    start_instant: Instant,
    start_time: SystemTime,
    status: Status,
    setting: Option<TimerSetting>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            status: Status::Stopped,
            count: 0,
            start_instant: Instant::now(),
            start_time: SystemTime::now(),
            setting: None,
        }
    }

    pub fn start(&mut self, setting: &TimerSetting) {
        self.setting = Some(setting.clone());
        self.start_instant = Instant::now();
        self.start_time = SystemTime::now();
        self.status = Status::Started;
    }

    pub fn stop(&mut self) -> u64 {
        self.status = Status::Stopped;
        if let Some(s) = self.setting.take() {
            if s.for_work {
                return self.count;
            }
        }
        0
    }

    pub fn current_name(&self) -> Option<&str> {
        if let Some(s) = self.setting.as_ref() {
            Some(&s.name)
        } else {
            None
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn get_start_time(&self) -> &SystemTime {
        &self.start_time
    }

    pub fn update(&mut self) -> (bool, String) {
        let mut is_timeout = false;
        let counter_string = if let Some(setting) = self.setting.as_ref() {
            let limit_count = setting.limit_time * 60;
            self.count = self.start_instant.elapsed().as_secs();
            if self.status != Status::TimeOut && self.count >= limit_count {
                self.status = Status::TimeOut;
                is_timeout = true;
            }

            let (sign, count) = if setting.count_up {
                ("", self.count)
            } else {
                if self.count <= limit_count {
                    ("", limit_count - self.count)
                } else {
                    ("-", self.count - limit_count)
                }
            };

            let minutes = count / 60;
            if minutes >= 60 {
                let hours = minutes / 60;
                format!("{}{:02}:{:02}:{:02}", sign, hours, minutes % 60, count % 60)
            } else {
                format!("{}{:02}:{:02}", sign, minutes, count % 60)
            }
        } else {
            "00:00".to_string()
        };
        (is_timeout, counter_string)
    }
}
