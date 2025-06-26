use serde::{Deserialize, Serialize};
use sled::{Db, IVec};
use std::time::{SystemTime, UNIX_EPOCH};
use toml;

pub struct History {
    db: Db,
}

impl History {
    pub fn new() -> Self {
        let mut path = crate::setting::get_config_dir();

        #[cfg(debug_assertions)]
        path.push("dbg_history_db");
        #[cfg(not(debug_assertions))]
        path.push("history_db");

        Self {
            db: sled::open(&path).unwrap(),
        }
    }

    pub fn add_record(
        &mut self,
        start_time: &SystemTime,
        duration: u64,
        timer_name: &str,
        tag: &str,
    ) {
        let secs = Self::time_to_secs(start_time);
        let record = RecordTmp {
            d: duration,
            n: timer_name.to_string(),
            t: tag.to_string(),
        };
        self.db
            .insert(
                secs.to_be_bytes(),
                toml::to_string(&record).unwrap().as_bytes(),
            )
            .ok();
        self.db.flush().ok();
    }

    pub fn get_records(&self, start: &SystemTime, end: &SystemTime) -> Vec<Record> {
        let start = Self::time_to_secs(start).to_be_bytes();
        let end = Self::time_to_secs(end).to_be_bytes();
        let mut rst = Vec::new();
        for item in self.db.range(start..end) {
            if let Ok((key, value)) = item {
                if let Some(record) = Self::to_record(key, value) {
                    rst.push(record);
                }
            }
        }
        // println!("{:?}", rst);
        rst
    }

    pub fn close(&self) {
        self.db.flush().ok();
    }

    fn to_record(key: IVec, value: IVec) -> Option<Record> {
        if let Ok(value) = std::str::from_utf8(value.as_ref()) {
            if let Ok(t) = toml::from_str::<RecordTmp>(value) {
                let array: [u8; 8] = key.as_ref().try_into().unwrap();
                let start_time = u64::from_be_bytes(array);
                return Some(Record {
                    start_time,
                    duration: t.d,
                    timer_name: t.n,
                    tag: t.t,
                });
            }
        }
        None
    }

    fn time_to_secs(start_time: &SystemTime) -> u64 {
        start_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

#[derive(Debug)]
pub struct Record {
    pub start_time: u64,
    pub duration: u64,
    pub timer_name: String,
    pub tag: String,
}

#[derive(Deserialize, Serialize)]
struct RecordTmp {
    d: u64,
    n: String,
    t: String,
}
