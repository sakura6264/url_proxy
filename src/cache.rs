use sled;

const ONE_DAY: usize = 24 * 3600 * 1000;
const REBUILD_EXPIRE: usize = 5 * 60 * 1000;
const REBUILD_TRY_LIMIT: usize = 3;

pub struct IconCacheManager {
    db: Option<sled::Db>,
    rebuild_try: usize,
    rebuild_time: usize,
    expire_time: usize,
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
    pub fn get(&mut self, name: &str, path: &str) -> Option<image::RgbaImage> {
        match self.db {
            Some(ref db) => {
                match db.get(name) {
                    Ok(Some(data)) => {
                        // the data is not raw picture data.
                        // must parse first.
                        match Self::extract_data(data.to_vec()) {
                            Some((img, time)) => {
                                let now = crate::utils::get_unix_msec();
                                if time + self.expire_time < now {
                                    // expired
                                    // load from file and cache it.
                                    let icon = crate::utils::extract_icon(path);
                                    let to_insert = match Self::make_data(icon.clone(), now) {
                                        Some(data) => data,
                                        None => Self::make_data(None, now).unwrap(),
                                    };
                                    if let Err(e) = db.insert(name, to_insert) {
                                        log::error!("Insert into DB failed:{e}");
                                    }
                                    return icon;
                                }
                                return img;
                            }
                            None => {
                                // parse failed
                                // load from file and cache it.
                                let now = crate::utils::get_unix_msec();
                                log::error!("Data from {name} has error");
                                let icon = crate::utils::extract_icon(path);
                                let to_insert = match Self::make_data(icon.clone(), now) {
                                    Some(data) => data,
                                    None => Self::make_data(None, now).unwrap(),
                                };
                                if let Err(e) = db.insert(name, to_insert) {
                                    log::error!("Insert into DB failed:{e}");
                                }
                                return icon;
                            }
                        }
                    }
                    Ok(None) => {
                        // no data
                        // load from file and cache it.
                        let now = crate::utils::get_unix_msec();
                        let icon = crate::utils::extract_icon(path);
                        let to_insert = match Self::make_data(icon.clone(), now) {
                            Some(data) => data,
                            None => Self::make_data(None, now).unwrap(),
                        };
                        if let Err(e) = db.insert(name, to_insert) {
                            log::error!("Insert into DB failed:{e}");
                        }
                        return icon;
                    }
                    Err(e) => {
                        // error
                        // try to force rebuild
                        // load from file for this time
                        log::error!("DB is down: {e}");
                        self.force_rebuild();
                        let icon = crate::utils::extract_icon(path);
                        return icon;
                    }
                }
            }
            None => {
                // no db
                // the db is down
                // try rebuild and
                // direct load from file
                self.force_rebuild();
                let icon = crate::utils::extract_icon(path);
                return icon;
            }
        }
    }
    pub fn force_rebuild(&mut self) {
        if self.rebuild_try < REBUILD_TRY_LIMIT {
            self.db = None;
            return;
        }
        let now = crate::utils::get_unix_msec();
        if now - self.rebuild_time < REBUILD_EXPIRE {
            self.db = None;
            return;
        }
        self.rebuild_try = self.rebuild_try + 1;
        self.rebuild_time = now;
        self.db = None;
        // force clear the db path
        std::fs::remove_dir_all(crate::utils::cache_path()).ok();
        self.db = sled::open(crate::utils::cache_path()).ok();
    }
    fn make_data(img: Option<image::RgbaImage>, timestamp: usize) -> Option<Vec<u8>> {
        let time_bytes = timestamp.to_le_bytes();
        let mut output = Vec::from(time_bytes);
        if let Some(img) = img {
            let mut data = Vec::new();
            let writer = std::io::Cursor::new(&mut data);
            let encoder = image::codecs::png::PngEncoder::new(writer);
            img.write_with_encoder(encoder).ok()?;
            output.append(&mut data);
        }
        Some(output)
    }
    fn extract_data(data: Vec<u8>) -> Option<(Option<image::RgbaImage>, usize)> {
        // timestamp is usize
        if data.len() < std::mem::size_of::<usize>() {
            return None;
        } else if data.len() == std::mem::size_of::<usize>() {
            let time_bytes = data.as_slice();
            let timestamp = usize::from_le_bytes(time_bytes.try_into().ok()?);
            return Some((None, timestamp));
        } else {
            let (time_bytes, img_bytes) = data.split_at(std::mem::size_of::<usize>());
            let timestamp = usize::from_le_bytes(time_bytes.try_into().ok()?);
            let img = image::load_from_memory(&img_bytes).ok()?;
            Some((Some(img.to_rgba8()), timestamp))
        }
    }
}
