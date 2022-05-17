const DATETIME_FORMAT: &str ="%Y/%m/%d %H:%M:%S";

pub mod datetime_serde_format {
    use chrono::{offset::TimeZone, DateTime, Local, NaiveDateTime};
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::DATETIME_FORMAT;

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(&DATETIME_FORMAT));
        return serializer.serialize_str(&s);
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut s = String::deserialize(deserializer)?;
        let mut colon_count = s.matches(":").count();

        // Add Trailing Zeros for Missing DateTime data
        while colon_count < 2 {
            s.push_str(":00");
            colon_count += 1;
        }

        match NaiveDateTime::parse_from_str(&s, &DATETIME_FORMAT).map_err(serde::de::Error::custom) {
            Ok(val) => {
                return Ok(Local.from_local_datetime(&val).unwrap());
            },
            Err(e) => return Err(e),
        }
    }
}
