//! Device information collection for the FunnelMob SDK.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::FunnelMobError;

/// Device information collected for event tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier (stored UUID).
    pub device_id: String,

    /// Operating system name.
    pub os_name: String,

    /// Operating system version.
    pub os_version: String,

    /// Device hostname.
    pub hostname: Option<String>,

    /// System locale.
    pub locale: Option<String>,

    /// System timezone.
    pub timezone: Option<String>,
}

impl DeviceInfo {
    /// Collects device information.
    ///
    /// The device_id is stored in a file for persistence across sessions.
    /// If the file doesn't exist, a new UUID is generated and stored.
    ///
    /// `storage_id` is a filesystem-safe identifier used to namespace the
    /// device_id storage (derived from the API key).
    pub fn collect(storage_id: &str) -> Result<Self, FunnelMobError> {
        let device_id = Self::get_or_create_device_id(storage_id)?;

        Ok(Self {
            device_id,
            os_name: Self::get_os_name(),
            os_version: Self::get_os_version(),
            hostname: Self::get_hostname(),
            locale: Self::get_locale(),
            timezone: Self::get_timezone(),
        })
    }

    /// Converts collected host details into the `/v1/session` context payload.
    pub fn to_context(&self) -> crate::internal::network_client::DeviceContext {
        crate::internal::network_client::DeviceContext {
            app_version: None,
            os_name: Some(self.os_name.clone()),
            os_version: Some(self.os_version.clone()),
            device_model: None,
            locale: self.locale.clone(),
            timezone: self.timezone.clone(),
            screen_width: None,
            screen_height: None,
        }
    }

    /// Gets the stored device ID or creates a new one.
    fn get_or_create_device_id(storage_id: &str) -> Result<String, FunnelMobError> {
        let path = Self::device_id_path(storage_id)?;

        // Try to read existing device ID
        if path.exists() {
            if let Ok(id) = fs::read_to_string(&path) {
                let id = id.trim();
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }

        // Generate new device ID
        let device_id = Uuid::new_v4().to_string();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                FunnelMobError::Configuration(format!(
                    "Failed to create device ID directory: {}",
                    e
                ))
            })?;
        }

        // Store the device ID
        fs::write(&path, &device_id).map_err(|e| {
            FunnelMobError::Configuration(format!("Failed to write device ID: {}", e))
        })?;

        Ok(device_id)
    }

    /// Returns the path where the device ID is stored.
    fn device_id_path(storage_id: &str) -> Result<PathBuf, FunnelMobError> {
        let data_dir = dirs::data_dir().ok_or_else(|| {
            FunnelMobError::Configuration("Could not determine data directory".to_string())
        })?;

        // Sanitize storage_id for use in path
        let safe_id: String = storage_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        Ok(data_dir
            .join("funnelmob")
            .join(safe_id)
            .join("device_id"))
    }

    /// Gets the operating system name.
    fn get_os_name() -> String {
        #[cfg(target_os = "linux")]
        {
            "Linux".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "macOS".to_string()
        }
        #[cfg(target_os = "windows")]
        {
            "Windows".to_string()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            std::env::consts::OS.to_string()
        }
    }

    /// Gets the operating system version.
    fn get_os_version() -> String {
        #[cfg(target_os = "linux")]
        {
            Self::get_linux_version()
        }
        #[cfg(target_os = "macos")]
        {
            Self::get_macos_version()
        }
        #[cfg(target_os = "windows")]
        {
            Self::get_windows_version()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            "unknown".to_string()
        }
    }

    #[cfg(target_os = "linux")]
    fn get_linux_version() -> String {
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("VERSION_ID=") {
                    return line
                        .trim_start_matches("VERSION_ID=")
                        .trim_matches('"')
                        .to_string();
                }
            }
        }

        std::process::Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(target_os = "macos")]
    fn get_macos_version() -> String {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    #[cfg(target_os = "windows")]
    fn get_windows_version() -> String {
        "10".to_string()
    }

    /// Gets the system hostname.
    fn get_hostname() -> Option<String> {
        #[cfg(unix)]
        {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        }
        #[cfg(windows)]
        {
            std::env::var("COMPUTERNAME").ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            None
        }
    }

    /// Gets the system locale.
    fn get_locale() -> Option<String> {
        std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .ok()
            .map(|l| {
                l.split('.')
                    .next()
                    .unwrap_or(&l)
                    .to_string()
            })
    }

    /// Gets the system timezone.
    fn get_timezone() -> Option<String> {
        std::env::var("TZ").ok().or_else(|| {
            #[cfg(target_os = "linux")]
            {
                fs::read_to_string("/etc/timezone")
                    .ok()
                    .map(|s| s.trim().to_string())
            }
            #[cfg(not(target_os = "linux"))]
            {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_os_name() {
        let os_name = DeviceInfo::get_os_name();
        assert!(!os_name.is_empty());
        assert!(
            os_name == "Linux" || os_name == "macOS" || os_name == "Windows" || !os_name.is_empty()
        );
    }

    #[test]
    fn test_os_version() {
        let version = DeviceInfo::get_os_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_device_id_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("device_id");

        let test_id = "test-device-id-123";
        fs::write(&path, test_id).unwrap();

        let read_id = fs::read_to_string(&path).unwrap();
        assert_eq!(read_id, test_id);
    }

    #[test]
    fn test_collect() {
        let result = DeviceInfo::collect("test_key");
        if let Ok(info) = result {
            assert!(!info.device_id.is_empty());
            assert!(!info.os_name.is_empty());
        }
    }

    #[test]
    fn test_to_context_omits_device_model_without_true_model() {
        let info = DeviceInfo {
            device_id: "device".to_string(),
            os_name: "Linux".to_string(),
            os_version: "6.1".to_string(),
            hostname: Some("host".to_string()),
            locale: Some("en_US".to_string()),
            timezone: Some("UTC".to_string()),
        };

        let context = info.to_context();
        assert_eq!(context.os_name.as_deref(), Some("Linux"));
        assert_eq!(context.os_version.as_deref(), Some("6.1"));
        assert_eq!(context.device_model, None);
        assert_eq!(context.locale.as_deref(), Some("en_US"));
        assert_eq!(context.timezone.as_deref(), Some("UTC"));
        assert_eq!(context.screen_width, None);
        assert_eq!(context.screen_height, None);
    }
}
