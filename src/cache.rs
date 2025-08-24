use image::RgbaImage;
use std::io::Cursor;
use std::io::Error;
use std::result::Result;

// Time constants
const ONE_DAY: usize = 24 * 3600 * 1000; // 24 hours in milliseconds
const REBUILD_EXPIRE: usize = 5 * 60 * 1000; // 5 minutes in milliseconds
const REBUILD_TRY_LIMIT: usize = 3;

/// Represents an icon cache entry with timestamp
struct CacheEntry {
    image: Option<RgbaImage>,
    timestamp: usize,
}

/// Manages icon caching to avoid repeated extraction from executables
pub struct IconCacheManager {
    db: Option<sled::Db>, // Database handle
    rebuild_try: usize,   // Number of rebuild attempts
    rebuild_time: usize,  // Last rebuild timestamp
    expire_time: usize,   // Cache expiration time in milliseconds
}

impl IconCacheManager {
    pub fn new(expire_days: usize) -> Self {
        let db = sled::open(crate::utils::cache_path()).ok();
        Self {
            db,
            rebuild_try: 0,
            rebuild_time: crate::utils::get_unix_msec(),
            expire_time: expire_days * ONE_DAY,
        }
    }
    /// Get an icon from cache or extract it from the executable
    pub fn get(&mut self, name: &str, path: &str) -> Option<RgbaImage> {
        // If database is available, try to get from cache
        if let Some(ref db) = self.db {
            match self.get_from_db(db, name, path) {
                Ok(Some(image)) => return Some(image),
                Ok(None) => {} // Continue to extract icon
                Err(_) => {
                    // DB error, try to rebuild and extract icon directly
                    self.force_rebuild();
                }
            }
        } else {
            // No database available, try to rebuild
            self.force_rebuild();
        }

        // Extract icon directly from file
        crate::utils::extract_icon(path)
    }

    /// Try to get an icon from the database
    fn get_from_db(
        &self,
        db: &sled::Db,
        name: &str,
        path: &str,
    ) -> Result<Option<RgbaImage>, Error> {
        match db.get(name) {
            Ok(Some(data)) => {
                // Parse the cached data
                match Self::extract_data(data.to_vec()) {
                    Some(entry) => {
                        let now = crate::utils::get_unix_msec();

                        // Check if cache entry is expired
                        if entry.timestamp + self.expire_time < now {
                            // Expired, extract and update cache
                            self.update_cache(db, name, path, now);
                            Ok(crate::utils::extract_icon(path))
                        } else {
                            // Not expired, return cached image
                            Ok(entry.image)
                        }
                    }
                    None => {
                        // Parse failed, extract and update cache
                        log::error!("Data from {name} has parsing error");
                        self.update_cache(db, name, path, crate::utils::get_unix_msec());
                        Ok(crate::utils::extract_icon(path))
                    }
                }
            }
            Ok(None) => {
                // No data in cache, extract and update cache
                self.update_cache(db, name, path, crate::utils::get_unix_msec());
                Ok(crate::utils::extract_icon(path))
            }
            Err(e) => {
                log::error!("Database error: {e}");
                Err(Error::other(format!("Database error: {}", e)))
            }
        }
    }

    /// Update the cache with a new icon
    fn update_cache(&self, db: &sled::Db, name: &str, path: &str, timestamp: usize) -> Option<()> {
        let icon = crate::utils::extract_icon(path);
        let data = Self::make_data(icon.clone(), timestamp)?;

        if let Err(e) = db.insert(name, data) {
            log::error!("Failed to insert into cache: {e}");
            return None;
        }

        Some(())
    }
    /// Attempt to rebuild the database if it's corrupted
    pub fn force_rebuild(&mut self) {
        // First, try to reopen immediately without destructive actions
        match sled::open(crate::utils::cache_path()) {
            Ok(db) => {
                self.db = Some(db);
                self.rebuild_try = 0;
                self.rebuild_time = crate::utils::get_unix_msec();
                return;
            }
            Err(e) => {
                log::warn!("Failed to open cache database: {e}");
                self.db = None;
            }
        }

        let now = crate::utils::get_unix_msec();

        // If we haven't hit the try limit, just back off and try later
        if self.rebuild_try < REBUILD_TRY_LIMIT {
            self.rebuild_try += 1;
            self.rebuild_time = now;
            return;
        }

        // Rate-limit destructive purge attempts
        if now - self.rebuild_time < REBUILD_EXPIRE {
            return;
        }

        log::info!("Purging and rebuilding cache database");
        self.rebuild_time = now;

        if let Err(e) = std::fs::remove_dir_all(crate::utils::cache_path()) {
            if e.kind() != std::io::ErrorKind::NotFound {
                log::warn!("Failed to remove cache directory: {e}");
            }
        }

        match sled::open(crate::utils::cache_path()) {
            Ok(db) => {
                log::info!("Successfully rebuilt cache database");
                self.db = Some(db);
                self.rebuild_try = 0;
            }
            Err(e) => {
                log::error!("Failed to rebuild cache database: {e}");
                self.db = None;
            }
        }
    }
    /// Serialize a cache entry to bytes
    fn make_data(img: Option<RgbaImage>, timestamp: usize) -> Option<Vec<u8>> {
        // Start with timestamp bytes
        let time_bytes = timestamp.to_le_bytes();
        let mut output = Vec::from(time_bytes);

        // If we have an image, encode it as PNG and append
        if let Some(img) = img {
            let mut data = Vec::new();
            let writer = Cursor::new(&mut data);
            let encoder = image::codecs::png::PngEncoder::new(writer);

            if let Err(e) = img.write_with_encoder(encoder) {
                log::error!("Failed to encode image: {e}");
                return None;
            }

            output.append(&mut data);
        }

        Some(output)
    }

    /// Deserialize bytes to a cache entry
    fn extract_data(data: Vec<u8>) -> Option<CacheEntry> {
        let timestamp_size = std::mem::size_of::<usize>();

        // Check if data is too small to contain timestamp
        if data.len() < timestamp_size {
            log::error!("Cache data too small to contain timestamp");
            return None;
        }

        // Extract timestamp
        let (time_bytes, img_bytes) = data.split_at(timestamp_size);
        let timestamp = usize::from_le_bytes(time_bytes.try_into().ok()?);

        // If there's no image data, return entry with None image
        if img_bytes.is_empty() {
            return Some(CacheEntry {
                image: None,
                timestamp,
            });
        }

        // Try to load image data
        match image::load_from_memory(img_bytes) {
            Ok(img) => Some(CacheEntry {
                image: Some(img.to_rgba8()),
                timestamp,
            }),
            Err(e) => {
                log::error!("Failed to decode image from cache: {e}");
                Some(CacheEntry {
                    image: None,
                    timestamp,
                })
            }
        }
    }
}
