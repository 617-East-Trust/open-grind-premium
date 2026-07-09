use keyring_core::Entry;
use rand;
use serde::{Deserialize, Serialize};
use wreq::header::{HeaderName, HeaderValue};

use crate::error::AppError;

use super::version::{FALLBACK_APP_VERSION, FALLBACK_BUILD_NUMBER};

const APP_VERSION: &str = FALLBACK_APP_VERSION;
const BUILD_NUMBER: &str = FALLBACK_BUILD_NUMBER;

const MAX_ANDROID_VERSION: u8 = 16;

struct DeviceProfile {
    manufacturer: &'static str,
    device_model: &'static str,
    screen_resolution: &'static str,
    total_ram: &'static str,
    min_android: u8,
}

const DEVICE_PROFILES: &[DeviceProfile] = &[
    // Google Pixel
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 6",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 6 Pro",
        screen_resolution: "3120x1440",
        total_ram: "12017676288",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 6a",
        screen_resolution: "2400x1080",
        total_ram: "5938152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 7",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 7 Pro",
        screen_resolution: "3120x1440",
        total_ram: "12017676288",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 7a",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 8",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 8 Pro",
        screen_resolution: "2992x1344",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 8a",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 9",
        screen_resolution: "2424x1080",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 9 Pro",
        screen_resolution: "2856x1280",
        total_ram: "16065654784",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Google",
        device_model: "Pixel 9 Pro XL",
        screen_resolution: "2992x1344",
        total_ram: "16065654784",
        min_android: 14,
    },
    // Samsung
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S901B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S906B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S908B",
        screen_resolution: "3088x1440",
        total_ram: "12017676288",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S911B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S916B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S918B",
        screen_resolution: "3088x1440",
        total_ram: "12017676288",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S921B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S926B",
        screen_resolution: "2340x1080",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-S928B",
        screen_resolution: "3120x1440",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A546B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A346B",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A145F",
        screen_resolution: "2408x1080",
        total_ram: "3852152832",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-F731B",
        screen_resolution: "2640x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-F946B",
        screen_resolution: "2176x1812",
        total_ram: "12017676288",
        min_android: 13,
    },
    // Xiaomi / Redmi / POCO
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "2201123G",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "2211133G",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "23078PND5G",
        screen_resolution: "2712x1220",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "23127PN0CG",
        screen_resolution: "2670x1200",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "23021RAA2Y",
        screen_resolution: "2400x1080",
        total_ram: "3852152832",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "23117RA68G",
        screen_resolution: "2712x1220",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "23049PCD8G",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    // OnePlus
    DeviceProfile {
        manufacturer: "OnePlus",
        device_model: "NE2213",
        screen_resolution: "3216x1440",
        total_ram: "8026152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "OnePlus",
        device_model: "CPH2449",
        screen_resolution: "3216x1440",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "OnePlus",
        device_model: "CPH2581",
        screen_resolution: "3168x1440",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "OnePlus",
        device_model: "CPH2491",
        screen_resolution: "2772x1240",
        total_ram: "8026152960",
        min_android: 13,
    },
    // Motorola
    DeviceProfile {
        manufacturer: "motorola",
        device_model: "motorola edge 40",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "motorola",
        device_model: "motorola g84 5G",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    // Sony
    DeviceProfile {
        manufacturer: "Sony",
        device_model: "XQ-DQ54",
        screen_resolution: "3840x1644",
        total_ram: "12017676288",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Sony",
        device_model: "XQ-DE54",
        screen_resolution: "2520x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    // Nothing
    DeviceProfile {
        manufacturer: "Nothing",
        device_model: "A065",
        screen_resolution: "2412x1080",
        total_ram: "12017676288",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Nothing",
        device_model: "A142",
        screen_resolution: "2412x1084",
        total_ram: "8026152960",
        min_android: 14,
    },
    // Asus
    DeviceProfile {
        manufacturer: "asus",
        device_model: "ASUS_AI2205",
        screen_resolution: "2448x1080",
        total_ram: "12017676288",
        min_android: 13,
    },
    // ----- Expanded device profiles (real-world global market) -----
    // Realme
    DeviceProfile {
        manufacturer: "Realme",
        device_model: "RMX3700",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Realme",
        device_model: "RMX3851",
        screen_resolution: "2412x1080",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Realme",
        device_model: "RMX3760",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Realme",
        device_model: "RMX3615",
        screen_resolution: "1600x720",
        total_ram: "3852152832",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Realme",
        device_model: "RMX3781",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    // Oppo
    DeviceProfile {
        manufacturer: "OPPO",
        device_model: "CPH2525",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "OPPO",
        device_model: "CPH2487",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "OPPO",
        device_model: "CPH2551",
        screen_resolution: "2772x1240",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "OPPO",
        device_model: "CPH2591",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "OPPO",
        device_model: "CPH2465",
        screen_resolution: "1600x720",
        total_ram: "3852152832",
        min_android: 12,
    },
    // Vivo
    DeviceProfile {
        manufacturer: "vivo",
        device_model: "V2330",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "vivo",
        device_model: "V2309",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "vivo",
        device_model: "V2312",
        screen_resolution: "2400x1080",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "vivo",
        device_model: "V2240",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "vivo",
        device_model: "V2341",
        screen_resolution: "2800x1260",
        total_ram: "12017676288",
        min_android: 14,
    },
    // Tecno / Infinix
    DeviceProfile {
        manufacturer: "Tecno",
        device_model: "TECNO-L8Pro",
        screen_resolution: "2460x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Tecno",
        device_model: "TECNO-KJ7n",
        screen_resolution: "2460x1080",
        total_ram: "3852152832",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Tecno",
        device_model: "TECNO-CK9",
        screen_resolution: "1600x720",
        total_ram: "3852152832",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Infinix",
        device_model: "X6816",
        screen_resolution: "1612x720",
        total_ram: "3852152832",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Infinix",
        device_model: "X6827",
        screen_resolution: "2460x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Infinix",
        device_model: "X689B",
        screen_resolution: "2460x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    // Huawei
    DeviceProfile {
        manufacturer: "HUAWEI",
        device_model: "ALN-LX9",
        screen_resolution: "2700x1224",
        total_ram: "8026152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "HUAWEI",
        device_model: "MNA-LX9",
        screen_resolution: "2600x1200",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "HUAWEI",
        device_model: "JLN-LX1",
        screen_resolution: "2388x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    // Honor
    DeviceProfile {
        manufacturer: "Honor",
        device_model: "LGE-NX9",
        screen_resolution: "2664x1200",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "Honor",
        device_model: "ALI-NX3",
        screen_resolution: "2388x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Honor",
        device_model: "RBN-NX1",
        screen_resolution: "2664x1200",
        total_ram: "8026152960",
        min_android: 13,
    },
    // More budget Samsung
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A156B",
        screen_resolution: "2340x1080",
        total_ram: "5938152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A256E",
        screen_resolution: "2340x1080",
        total_ram: "5938152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-M346B",
        screen_resolution: "2408x1080",
        total_ram: "5938152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A556E",
        screen_resolution: "2340x1080",
        total_ram: "8026152960",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "samsung",
        device_model: "SM-A057F",
        screen_resolution: "1600x720",
        total_ram: "3852152832",
        min_android: 13,
    },
    // More Xiaomi/Redmi budget
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "21071115SI",
        screen_resolution: "2400x1080",
        total_ram: "5938152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Xiaomi",
        device_model: "22031116BG",
        screen_resolution: "2400x1080",
        total_ram: "5938152960",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Redmi",
        device_model: "23028RN4DG",
        screen_resolution: "2400x1080",
        total_ram: "5938152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "Redmi",
        device_model: "23076RN8DY",
        screen_resolution: "2400x1080",
        total_ram: "3852152832",
        min_android: 12,
    },
    DeviceProfile {
        manufacturer: "Redmi",
        device_model: "23122PCD1G",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    // Motorola budget
    DeviceProfile {
        manufacturer: "motorola",
        device_model: "motorola edge 40 neo",
        screen_resolution: "2400x1080",
        total_ram: "8026152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "motorola",
        device_model: "motorola g54 5G",
        screen_resolution: "2400x1080",
        total_ram: "5938152960",
        min_android: 13,
    },
    DeviceProfile {
        manufacturer: "motorola",
        device_model: "motorola g22",
        screen_resolution: "1600x720",
        total_ram: "3852152832",
        min_android: 12,
    },
    // More OnePlus
    DeviceProfile {
        manufacturer: "OnePlus",
        device_model: "CPH2611",
        screen_resolution: "2772x1240",
        total_ram: "12017676288",
        min_android: 14,
    },
    DeviceProfile {
        manufacturer: "OnePlus",
        device_model: "CPH2451",
        screen_resolution: "2772x1240",
        total_ram: "12017676288",
        min_android: 14,
    },
];

pub const SAFE_TIMEZONES: &[&str] = &[
    // Europe
    "Europe/Dublin",
    "Europe/Zurich",
    "Europe/Prague",
    "Europe/Bratislava",
    "Europe/Budapest",
    "Europe/Bucharest",
    "Europe/Sofia",
    "Europe/Zagreb",
    "Europe/Vilnius",
    "Europe/Riga",
    "Europe/Tallinn",
    "Europe/Luxembourg",
    "Europe/Malta",
    "Europe/London",
    "Europe/Paris",
    "Europe/Berlin",
    "Europe/Madrid",
    "Europe/Rome",
    "Europe/Amsterdam",
    "Europe/Brussels",
    "Europe/Vienna",
    "Europe/Stockholm",
    "Europe/Oslo",
    "Europe/Copenhagen",
    "Europe/Warsaw",
    "Europe/Helsinki",
    "Europe/Athens",
    "Europe/Lisbon",
    // Americas
    "America/Mexico_City",
    "America/Argentina/Buenos_Aires",
    "America/Santiago",
    "America/Bogota",
    "America/Lima",
    "America/Montevideo",
    "America/New_York",
    "America/Chicago",
    "America/Los_Angeles",
    "America/Denver",
    "America/Phoenix",
    "America/Anchorage",
    "America/Sao_Paulo",
    "America/Toronto",
    "America/Vancouver",
    // Asia-Pacific
    "Asia/Tokyo",
    "Asia/Taipei",
    "Asia/Seoul",
    "Asia/Bangkok",
    "Asia/Manila",
    "Asia/Singapore",
    "Asia/Shanghai",
    "Asia/Hong_Kong",
    "Asia/Dubai",
    "Asia/Kolkata",
    "Asia/Jakarta",
    "Asia/Kuala_Lumpur",
    "Australia/Sydney",
    "Australia/Melbourne",
    "Pacific/Auckland",
];

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: u8,
    pub device_id: String,
    pub os: String,
    pub screen_resolution: String,
    pub total_ram: String,
    pub advertising_id: String,
    pub device_model: String,
    pub manufacturer: String,
    pub timezone: String,
    pub locale: String,
    pub accept_language: String,
    pub last_rotated: Option<u64>,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        let profile = &DEVICE_PROFILES[rand::random::<u64>() as usize % DEVICE_PROFILES.len()];
        let timezone = SAFE_TIMEZONES[rand::random::<u64>() as usize % SAFE_TIMEZONES.len()];

        let device_id = format!("{:016x}", rand::random::<u64>());

        let range = MAX_ANDROID_VERSION.saturating_sub(profile.min_android) + 1;
        let android_version = profile.min_android + rand::random::<u8>() % range;

        Self {
            device_type: 2,
            device_id,
            os: format!("Android {android_version}"),
            screen_resolution: profile.screen_resolution.to_owned(),
            total_ram: profile.total_ram.to_owned(),
            advertising_id: uuid::Uuid::new_v4().to_string(),
            device_model: profile.device_model.to_owned(),
            manufacturer: profile.manufacturer.to_owned(),
            timezone: timezone.to_owned(),
            locale: "en_US".to_owned(),
            accept_language: "en-US".to_owned(),
            last_rotated: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ),
        }
    }
}

impl DeviceInfo {
    /// Short 8-char hex hash of device identity for settings/debug UI.
    /// Changes whenever device id, model, or timezone rotates.
    pub fn fingerprint_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.device_id.hash(&mut hasher);
        self.device_model.hash(&mut hasher);
        self.manufacturer.hash(&mut hasher);
        self.timezone.hash(&mut hasher);
        self.advertising_id.hash(&mut hasher);
        format!("{:08x}", (hasher.finish() & 0xffff_ffff) as u32)
    }
}

pub struct DeviceStorage;

impl DeviceStorage {
    fn entry() -> Result<Entry, AppError> {
        Entry::new("open-grind", "device").map_err(|e| AppError::Auth(e.to_string()))
    }

    pub fn load() -> Result<Option<DeviceInfo>, AppError> {
        let entry = Self::entry()?;
        let bytes = match entry.get_secret() {
            Ok(b) => b,
            Err(keyring_core::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(AppError::Auth(e.to_string())),
        };
        rmp_serde::decode::from_slice(&bytes)
            .map_err(|e| AppError::Auth(e.to_string()))
            .map(Some)
    }

    pub fn save(device: &DeviceInfo) -> Result<(), AppError> {
        let bytes = rmp_serde::encode::to_vec(device).map_err(|e| AppError::Auth(e.to_string()))?;
        Self::entry()?
            .set_secret(&bytes)
            .map_err(|e| AppError::Auth(e.to_string()))
    }
}

pub fn build_user_agent(device: &DeviceInfo, subscription_tier: &str) -> String {
    format!(
        "grindr3/{APP_VERSION};{BUILD_NUMBER};{subscription_tier};{};{};{}",
        device.os, device.device_model, device.manufacturer
    )
}

pub fn build_device_info_header(device: &DeviceInfo) -> String {
    format!(
        "{};GLOBAL;{};{};{};{}",
        device.device_id,
        device.device_type,
        device.total_ram,
        device.screen_resolution,
        device.advertising_id
    )
}

/// References https://opengrind.org/grindr-api/security-headers#correct-headers-order
///   1. Authorization (optional)
///   2. L-Time-Zone
///   3. L-Grindr-Roles (only when authorized)
///   4. L-Device-Info
///   5. Accept
///   6. User-Agent
///   7. L-Locale
///   8. Accept-language (lowercase `l`)
///   9. Accept-Encoding (always `gzip`)
///
/// `Content-Type`, `Content-Length`/`Transfer-Encoding` and `Cookie` are added
/// by wreq itself. `Host` is moved to the `:authority` pseudo-header in HTTP/2.
pub struct GrindrHeaders {
    pub items: Vec<(HeaderName, HeaderValue)>,
}

impl GrindrHeaders {
    pub fn build(
        device: &DeviceInfo,
        user_agent: &str,
        authorization: Option<&str>,
        l_grindr_roles: Option<&str>,
    ) -> Result<Self, AppError> {
        let mut items: Vec<(HeaderName, HeaderValue)> = Vec::with_capacity(8);

        if let Some(auth) = authorization {
            items.push((
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(auth).map_err(invalid_header)?,
            ));
        }

        items.push((
            HeaderName::from_static("l-time-zone"),
            HeaderValue::from_str(&device.timezone).map_err(invalid_header)?,
        ));

        if let Some(roles) = l_grindr_roles {
            items.push((
                HeaderName::from_static("l-grindr-roles"),
                HeaderValue::from_str(roles).map_err(invalid_header)?,
            ));
        }

        items.push((
            HeaderName::from_static("l-device-info"),
            HeaderValue::from_str(&build_device_info_header(device)).map_err(invalid_header)?,
        ));
        items.push((
            HeaderName::from_static("accept"),
            HeaderValue::from_static("application/json"),
        ));
        items.push((
            HeaderName::from_static("user-agent"),
            HeaderValue::from_str(user_agent).map_err(invalid_header)?,
        ));
        items.push((
            HeaderName::from_static("l-locale"),
            HeaderValue::from_str(&device.locale).map_err(invalid_header)?,
        ));
        items.push((
            HeaderName::from_static("accept-language"),
            HeaderValue::from_str(&device.accept_language).map_err(invalid_header)?,
        ));
        items.push((
            HeaderName::from_static("accept-encoding"),
            HeaderValue::from_static("gzip"),
        ));

        Ok(Self { items })
    }
}

fn invalid_header<E: std::fmt::Display>(e: E) -> AppError {
    AppError::Http(format!("Invalid header value: {e}"))
}

/// Historical helper for `L-Grindr-Roles`. Intentionally unused.
///
/// Sending `PREMIUM`/`UNLIMITED` roles for free-tier accounts is a
/// high-confidence server-side detection signal. Premium is injected only via
/// response rewriting (`rewrite.rs` / `maybe_rewrite_response`).
#[allow(dead_code)]
pub fn grindr_roles_header_value() -> Option<&'static str> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_device() -> DeviceInfo {
        DeviceInfo {
            device_type: 2,
            device_id: "device123".to_owned(),
            os: "Android 14".to_owned(),
            screen_resolution: "1080x2400".to_owned(),
            total_ram: "8026152960".to_owned(),
            advertising_id: "ad-id-123".to_owned(),
            device_model: "Pixel 8".to_owned(),
            manufacturer: "Google".to_owned(),
            timezone: "Europe/Madrid".to_owned(),
            locale: "en_US".to_owned(),
            accept_language: "en-US".to_owned(),
            last_rotated: None,
        }
    }

    #[test]
    fn user_agent_format() {
        let device = test_device();
        let ua = build_user_agent(&device, "Free");
        assert_eq!(
            ua,
            format!("grindr3/{APP_VERSION};{BUILD_NUMBER};Free;Android 14;Pixel 8;Google")
        );
    }

    #[test]
    fn device_info_header_format() {
        let device = test_device();
        assert_eq!(
            build_device_info_header(&device),
            "device123;GLOBAL;2;8026152960;1080x2400;ad-id-123"
        );
    }

    #[test]
    fn headers_anonymous_order() {
        let device = test_device();
        let headers = GrindrHeaders::build(&device, "ua/1.0", None, None).unwrap();
        let names: Vec<&str> = headers.items.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "l-time-zone",
                "l-device-info",
                "accept",
                "user-agent",
                "l-locale",
                "accept-language",
                "accept-encoding",
            ]
        );
    }

    #[test]
    fn headers_authorized_order() {
        let device = test_device();
        let headers =
            GrindrHeaders::build(&device, "ua/1.0", Some("Grindr3 sid"), None).unwrap();
        let names: Vec<&str> = headers.items.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "authorization",
                "l-time-zone",
                "l-device-info",
                "accept",
                "user-agent",
                "l-locale",
                "accept-language",
                "accept-encoding",
            ]
        );
    }

    #[test]
    fn roles_header_omitted_when_none() {
        let device = test_device();
        let headers =
            GrindrHeaders::build(&device, "ua/1.0", Some("Grindr3 sid"), None).unwrap();
        let names: Vec<&str> = headers.items.iter().map(|(n, _)| n.as_str()).collect();
        assert!(!names.contains(&"l-grindr-roles"));
        assert!(grindr_roles_header_value().is_none());
    }

    #[test]
    fn fingerprint_hash_is_stable_for_same_device() {
        let device = test_device();
        let a = device.fingerprint_hash();
        let b = device.fingerprint_hash();
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);
    }

    #[test]
    fn fingerprint_hash_changes_with_identity() {
        let mut device = test_device();
        let original = device.fingerprint_hash();
        device.device_id = "other-device".to_owned();
        assert_ne!(original, device.fingerprint_hash());
    }

    #[test]
    fn safe_timezones_have_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for tz in SAFE_TIMEZONES {
            assert!(seen.insert(*tz), "duplicate timezone: {tz}");
        }
    }

    #[test]
    fn device_profiles_non_empty() {
        assert!(DEVICE_PROFILES.len() >= 50, "expected expanded device pool");
    }
}
