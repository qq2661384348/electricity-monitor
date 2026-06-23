//! roomid 序列化工具
//!
//! 新外部接口的 RoomID 可能超过 JavaScript 安全整数范围。后端内部和数据库使用
//! `i64`，HTTP JSON 对外统一按字符串传输；反序列化保留数字兼容，便于旧调用方过渡。

use serde::{
    de::{Error, Visitor},
    Deserializer, Serializer,
};
use std::fmt;

pub fn serialize<S>(roomid: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&roomid.to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(RoomIdVisitor)
}

pub fn to_string(roomid: i64) -> String {
    roomid.to_string()
}

struct RoomIdVisitor;

impl<'de> Visitor<'de> for RoomIdVisitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("roomid as a string or integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(value)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        i64::try_from(value).map_err(|_| E::custom("roomid is larger than i64"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        value
            .parse::<i64>()
            .map_err(|_| E::custom("roomid string must be an i64 integer"))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(&value)
    }
}
