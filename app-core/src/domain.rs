pub mod invoice;
pub mod supplier;
pub mod user;

use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct YMD {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

impl ToString for YMD {
    fn to_string(&self) -> String {
        if self.is_empty() {
            return "".to_string();
        }
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for YMD {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(YMD {
                year: 0,
                month: 0,
                day: 0,
            });
        }

        if s.len() != 10 {
            return Err("illegal length".to_string());
        }

        let tmp = s.split("-").collect::<Vec<&str>>();

        Ok(YMD {
            year: tmp[0].parse().unwrap(),
            month: tmp[1].parse().unwrap(),
            day: tmp[2].parse().unwrap(),
        })
    }
}

impl YMD {
    fn is_empty(&self) -> bool {
        self.year == 0 || self.month == 0 || self.day == 0
    }

    pub fn to_datetime(&self) -> Option<chrono::NaiveDateTime> {
        if self.is_empty() {
            return None;
        }

        let result = chrono::NaiveDateTime::parse_from_str(
            format!("{} 09:00:00", self.to_string()).as_str(),
            "%Y-%m-%d %H:%M:%S",
        );

        match result {
            Ok(datetime) => Some(datetime),
            Err(_) => None,
        }
    }
}
