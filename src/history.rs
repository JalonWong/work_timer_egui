use serde::{Deserialize, Serialize};
use sled::{Db, IVec};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use toml;

pub struct History {
    db: Db,
}

impl History {
    pub fn new() -> Self {
        let mut path = crate::setting::get_config_dir();
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
        let key = Self::to_key(start_time);
        let record = RecordTmp {
            d: duration,
            n: timer_name.to_string(),
            t: tag.to_string(),
        };
        self.db
            .insert(key, toml::to_string(&record).unwrap().as_bytes())
            .ok();
        self.db.flush().ok();
    }

    pub fn get_records(&self, start: &SystemTime, end: &SystemTime, reverse: bool) -> Vec<Record> {
        let start = Self::to_key(start);
        let end = Self::to_key(end);
        let mut rst = Vec::new();
        if reverse {
            for item in self.db.range(start..end).rev() {
                if let Ok((key, value)) = item {
                    if let Some(record) = Self::to_record(key, value) {
                        rst.push(record);
                    }
                }
            }
        } else {
            for item in self.db.range(start..end) {
                if let Ok((key, value)) = item {
                    if let Some(record) = Self::to_record(key, value) {
                        rst.push(record);
                    }
                }
            }
        }

        // println!("{:?}", rst);
        rst
    }

    pub fn remove(&mut self, key: &SystemTime) {
        self.db.remove(Self::to_key(key)).ok();
        self.db.flush().ok();
    }

    fn to_record(key: IVec, value: IVec) -> Option<Record> {
        if let Ok(value) = std::str::from_utf8(value.as_ref()) {
            if let Ok(t) = toml::from_str::<RecordTmp>(value) {
                let array: [u8; 8] = key.as_ref().try_into().unwrap();
                let start_time_u64 = u64::from_be_bytes(array);
                let start_time = UNIX_EPOCH + Duration::from_secs(start_time_u64);
                return Some(Record {
                    start_time_u64,
                    start_time,
                    duration: t.d,
                    timer_name: t.n,
                    tag: t.t,
                });
            }
        }
        None
    }

    fn to_key(start_time: &SystemTime) -> [u8; 8] {
        start_time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_be_bytes()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Record {
    pub start_time_u64: u64,
    pub start_time: SystemTime,
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
