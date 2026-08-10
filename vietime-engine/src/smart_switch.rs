//! Smart switch key module (remembers per-app input state by bundle ID).
//! Ported from `SmartSwitchKey.cpp`.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Per-app input method memory controller.
#[derive(Clone, Debug, Default)]
pub struct SmartSwitchKey {
    data: BTreeMap<String, i8>,
    cache_key: String,
    cache_data: i8,
}

impl SmartSwitchKey {
    /// Create new SmartSwitchKey controller.
    pub fn new() -> Self {
        SmartSwitchKey {
            data: BTreeMap::new(),
            cache_key: String::new(),
            cache_data: 0,
        }
    }

    /// Clear all stored app mappings and cache.
    pub fn clear(&mut self) {
        self.data.clear();
        self.cache_key.clear();
        self.cache_data = 0;
    }

    /// Initialize state from binary config payload.
    pub fn init_from_binary(&mut self, payload: &[u8]) {
        self.clear();
        if payload.len() < 2 {
            return;
        }
        let count = u16::from_le_bytes([payload[0], payload[1]]) as usize;
        let mut cursor = 2;

        for _ in 0..count {
            if cursor >= payload.len() {
                break;
            }
            let id_len = payload[cursor] as usize;
            cursor += 1;
            if cursor + id_len > payload.len() {
                break;
            }
            let bundle_id = match core::str::from_utf8(&payload[cursor..cursor + id_len]) {
                Ok(s) => s,
                Err(_) => break,
            };
            cursor += id_len;
            if cursor >= payload.len() {
                break;
            }
            let val = payload[cursor] as i8;
            cursor += 1;

            self.data.insert(String::from(bundle_id), val);
        }
    }

    /// Get binary representation of current state for saving.
    pub fn get_binary_save_data(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let count = self.data.len() as u16;
        out.extend_from_slice(&count.to_le_bytes());

        for (key, &val) in &self.data {
            let key_bytes = key.as_bytes();
            out.push(key_bytes.len() as u8);
            out.extend_from_slice(key_bytes);
            out.push(val as u8);
        }

        out
    }

    /// Query input method for a given app bundle ID.
    ///
    /// Returns stored setting if found; otherwise registers current_input_method
    /// and returns -1.
    pub fn get_app_input_method_status(&mut self, bundle_id: &str, current_input_method: i8) -> i8 {
        if self.cache_key == bundle_id {
            return self.cache_data;
        }

        if let Some(&val) = self.data.get(bundle_id) {
            self.cache_key = String::from(bundle_id);
            self.cache_data = val;
            return self.cache_data;
        }

        self.cache_key = String::from(bundle_id);
        self.cache_data = current_input_method;
        self.data.insert(String::from(bundle_id), current_input_method);
        -1
    }

    /// Update stored input method state for an app bundle ID.
    pub fn set_app_input_method_status(&mut self, bundle_id: &str, language: i8) {
        self.data.insert(String::from(bundle_id), language);
        self.cache_key = String::from(bundle_id);
        self.cache_data = language;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_switch_workflow() {
        let mut sw = SmartSwitchKey::new();

        // 1. First time app is seen -> returns -1, registers current_input_method
        let status = sw.get_app_input_method_status("com.apple.Terminal", 1);
        assert_eq!(status, -1);

        // 2. Second query -> returns cached value 1
        let status2 = sw.get_app_input_method_status("com.apple.Terminal", 0);
        assert_eq!(status2, 1);

        // 3. Update setting to 0 (English)
        sw.set_app_input_method_status("com.apple.Terminal", 0);
        let status3 = sw.get_app_input_method_status("com.apple.Terminal", 1);
        assert_eq!(status3, 0);
    }

    #[test]
    fn test_smart_switch_binary_roundtrip() {
        let mut sw = SmartSwitchKey::new();
        sw.set_app_input_method_status("com.sublimetext.4", 1);
        sw.set_app_input_method_status("com.microsoft.VSCode", 0);

        let bytes = sw.get_binary_save_data();

        let mut sw2 = SmartSwitchKey::new();
        sw2.init_from_binary(&bytes);

        assert_eq!(sw2.get_app_input_method_status("com.sublimetext.4", 0), 1);
        assert_eq!(sw2.get_app_input_method_status("com.microsoft.VSCode", 1), 0);
    }
}
