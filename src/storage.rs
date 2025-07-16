use crate::resp::RESP;
use crate::set::{parse_set_arguments, KeyExpiry, SetArgs};
use crate::storage_result::{StorageError, StorageResult};
use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Debug, PartialEq)]
pub enum StorageValue {
    String(String),
}

#[derive(Debug)]
pub struct StorageData {
    pub value: StorageValue,
    pub created_at: SystemTime,
    pub expiry: Option<Duration>,
}

impl StorageData {
    pub fn add_expiry(&mut self, expiry: Duration) {
        self.expiry = Some(expiry);
    }
}

impl From<String> for StorageData {
    fn from(s: String) -> StorageData {
        StorageData {
            value: StorageValue::String(s),
            created_at: SystemTime::now(),
            expiry: None,
        }
    }
}

impl PartialEq for StorageData {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.expiry == other.expiry
    }
}

pub struct Storage {
    store: HashMap<String, StorageData>,
    expiry: HashMap<String, SystemTime>,
    active_expiry: bool,
}

impl Storage {
    pub fn new() -> Self {
        let store: HashMap<String, StorageData> = HashMap::new();
        let expiry: HashMap<String, SystemTime> = HashMap::new();
        let active_expiry: bool = true;
        Self {
            store,
            expiry,
            active_expiry,
        }
    }

    pub fn process_command(&mut self, command: &Vec<String>) -> StorageResult<RESP> {
        match command[0].to_lowercase().as_str() {
            "ping" => self.command_ping(&command),
            "echo" => self.command_echo(&command),
            "get" => self.command_get(&command),
            "set" => self.command_set(&command),
            _ => Err(StorageError::CommandNotAvailable(command[0].clone())),
        }
    }

    fn command_ping(&self, _command: &Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::SimpleString("PONG".to_string()))
    }

    fn command_echo(&self, command: &Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::BulkString(command[1].clone()))
    }

    fn set(&mut self, key: String, value: String, args: SetArgs) -> StorageResult<String> {
        let mut data = StorageData::from(value);

        if let Some(value) = args.expiry {
            let expiry = match value {
                KeyExpiry::EX(v) => Duration::from_secs(v),
                KeyExpiry::PX(v) => Duration::from_millis(v),
            };
            data.add_expiry(expiry);
            self.expiry
                .insert(key.clone(), SystemTime::now().add(expiry));
        }
        self.store.insert(key.clone(), data);
        Ok(String::from("OK"))
    }

    fn get(&mut self, key: String) -> StorageResult<Option<String>> {
        if let Some(&expiry) = self.expiry.get(&key) {
            if SystemTime::now() >= expiry {
                self.expiry.remove(&key);
                self.store.remove(&key);
                return Ok(None); // Key has expired
            }
        }
        match self.store.get(&key) {
            Some(StorageData {
                value: StorageValue::String(v),
                created_at: _,
                expiry: _,
            }) => return Ok(Some(v.clone())),
            None => return Ok(None),
        }
    }

    fn command_set(&mut self, command: &Vec<String>) -> StorageResult<RESP> {
        if command.len() < 3 {
            return Err(StorageError::CommandSyntaxError(command.join(" ")));
        }

        let key = command[1].clone();
        let value = command[2].clone();
        let args = parse_set_arguments(&command[3..].to_vec())?;
        let _ = self.set(key, value, args);
        Ok(RESP::SimpleString(String::from("OK")))
    }

    fn command_get(&mut self, command: &Vec<String>) -> StorageResult<RESP> {
        if command.len() != 2 {
            return Err(StorageError::CommandSyntaxError(command.join(" ")));
        }
        let key = command[1].clone();
        let output = self.get(key);
        match output {
            Ok(Some(value)) => Ok(RESP::BulkString(value)),
            Ok(None) => Ok(RESP::Null),
            Err(_) => Err(StorageError::CommandInternalError(command.join(" "))),
        }
    }

    pub fn set_active_expiry(&mut self, active: bool) {
        self.active_expiry = active;
    }

    pub fn expire_keys(&mut self) {
        if !self.active_expiry {
            return;
        }
        let now = SystemTime::now();
        self.expiry.retain(|key, &mut expiry_time| {
            if expiry_time <= now {
                self.store.remove(key);
                false
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new() {
        let storage: Storage = Storage::new();
        assert_eq!(storage.store.len(), 0);
        assert_eq!(storage.expiry.len(), 0);
        assert_eq!(storage.expiry, HashMap::<String, SystemTime>::new());
        assert!(storage.active_expiry);
    }

    #[test]
    fn test_command_ping() {
        let command = vec![String::from("ping")];
        let storage: Storage = Storage::new();
        let output = storage.command_ping(&command).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_command_ping_uppercase() {
        let command = vec![String::from("PING")];
        let storage: Storage = Storage::new();
        let output = storage.command_ping(&command).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_command_echo() {
        let command = vec![String::from("echo"), String::from("Hello, World!")];
        let storage: Storage = Storage::new();
        let output = storage.command_echo(&command).unwrap();
        assert_eq!(output, RESP::BulkString(String::from("Hello, World!")));
    }

    #[test]
    fn test_set_value() {
        let mut storage = Storage::new();
        let some_value = StorageData::from(String::from("some_value"));
        let output = storage
            .set(
                String::from("some_key"),
                String::from("some_value"),
                SetArgs::new(),
            )
            .unwrap();
        assert_eq!(output, String::from("OK"));
        assert_eq!(storage.store.len(), 1);
        match storage.store.get(&String::from("some_key")) {
            Some(value) => assert_eq!(value, &some_value),
            None => panic!("Value not found in storage"),
        }
    }

    #[test]
    fn test_get_value() {
        let mut storage = Storage::new();
        storage.store.insert(
            String::from("some_key"),
            StorageData::from(String::from("some_value")),
        );
        let result = storage.get(String::from("some_key")).unwrap();
        assert_eq!(storage.store.len(), 1);
        assert_eq!(result, Some(String::from("some_value")));
    }

    #[test]
    fn test_get_value_key_does_not_exist() {
        let mut storage = Storage::new();
        let result = storage.get(String::from("null_key")).unwrap();
        assert_eq!(storage.store.len(), 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_process_command_set() {
        let mut storage: Storage = Storage::new();
        let command = vec![
            String::from("set"),
            String::from("key"),
            String::from("value"),
        ];
        let output = storage.process_command(&command).unwrap();
        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(storage.store.len(), 1);
    }
    #[test]
    fn test_process_command_get() {
        let mut storage: Storage = Storage::new();
        storage.store.insert(
            String::from("akey"),
            StorageData::from(String::from("avalue")),
        );
        let command = vec![String::from("get"), String::from("akey")];
        let output = storage.process_command(&command).unwrap();
        assert_eq!(output, RESP::BulkString(String::from("avalue")));
        assert_eq!(storage.store.len(), 1);
    }

    #[test]
    fn test_expire_keys() {
        let mut storage: Storage = Storage::new();
        storage
            .set(
                String::from("some_key"),
                String::from("some_value"),
                SetArgs::new(),
            )
            .unwrap();
        storage.expiry.insert(
            String::from("some_key"),
            SystemTime::now() - Duration::from_secs(5),
        );
        storage.expire_keys();
        assert_eq!(storage.store.len(), 0);
    }

    #[test]
    fn test_expire_keys_deactivated() {
        let mut storage = Storage::new();
        storage.set_active_expiry(false);
        storage
            .set(
                String::from("some_key"),
                String::from("some_value"),
                SetArgs::new(),
            )
            .unwrap();
        storage.expiry.insert(
            String::from("some_key"),
            SystemTime::now() - Duration::from_secs(5),
        );
        storage.expire_keys();
        assert_eq!(storage.store.len(), 1);
    }

    #[test]
    fn test_set_value_with_px() {
        let mut storage = Storage::new();
        let mut some_value = StorageData::from(String::from("some_value"));
        some_value.add_expiry(Duration::from_millis(100));

        let output = storage.set(
            String::from("some_key"),
            String::from("some_value"),
            SetArgs {
                expiry: Some(KeyExpiry::PX(100)),
                existence: None,
                get: false,
            },
        )
        .unwrap();

        assert_eq!(output, String::from("OK"));
        assert_eq!(storage.store.len(), 1);
        match storage.store.get(&String::from("some_key")) {
            Some(value) => {
                assert_eq!(value, &some_value);
            }
            None => panic!("Value not found in storage"),
        }
        storage.expiry.get(&String::from("some_key")).unwrap();
    }
}
