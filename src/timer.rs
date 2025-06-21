use std::time::Instant;

#[derive(PartialEq, Clone, Copy)]
pub enum Status {
    Stopped,
    Started,
    TimeOut,
}

pub struct Timer {
    count: u64,
    limit_time: u64,
    start_time: Instant,
    status: Status,
    // TODO count down
}

impl Timer {
    pub fn new() -> Self {
        Self {
            limit_time: 25,
            status: Status::Stopped,
            count: 0,
            start_time: Instant::now(),
        }
    }

    pub fn start(&mut self, limit_time: u64) {
        self.limit_time = limit_time;
        self.start_time = Instant::now();
        self.status = Status::Started;
    }

    pub fn stop(&mut self) -> u64 {
        self.status = Status::Stopped;
        self.count
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn update(&mut self) -> String {
        if self.status != Status::Stopped {
            self.count = Instant::now().duration_since(self.start_time).as_secs();
            if self.status != Status::TimeOut && self.count >= self.limit_time * 60 {
                self.status = Status::TimeOut;
            }
        }

        let count = self.count;
        let minutes = count / 60;
        if minutes >= 60 {
            let hours = minutes / 60;
            format!("{:02}:{:02}:{:02}", hours, minutes % 60, count % 60)
        } else {
            format!("{:02}:{:02}", minutes, count % 60)
        }
    }
}
