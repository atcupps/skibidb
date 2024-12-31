use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{DBError, DBResult};

use super::error::StorageError;

/// A `Page` is a portion of a database file which can be read from a database
/// file, modified in main memory, and written back to the database file.
/// It consists of a vector of bytes, which has a maximum length of `page_size`
/// specified in `FileManager`.
struct Page {
    data: Vec<u8>,
    dirty: bool,
    pin_count: u16,
}

/// A `FileManager` manages reads and writes to a database file through a
/// `buffer_pool` of pages.
pub struct FileManager {
    file_path: String,
    file: File,
    buffer_pool: HashMap<u64, Page>,
    page_size: usize,
    max_pages_in_pool: usize,
    num_pages: u64,
}

impl FileManager {
    /// Creates a new `FileManager` given a `path` to the database File that
    /// is processed via this `FileManager`. `page_size` specifies the maximum
    /// number of bytes a `Page` can contain, and `max_pages_in_pool` specifies
    /// the maximum number of pages that can be held in the buffer pool.
    /// `max_pages_in_pool` must be at least 1.
    pub fn new(path: &str, page_size: usize, max_pages_in_pool: usize) -> DBResult<Self> {
        // ------------------- FIRST: CHECKING ALL ARGS ------------------- //
        if path.is_empty() {
            return Err(DBError::from(StorageError::InvalidArgument(
                "invalid FileManager file path: path string is empty.".to_string(),
            )));
        }

        if page_size < 1 {
            return Err(DBError::from(StorageError::InvalidArgument(
                "invalid FileManager page size: must be at least 1 byte.".to_string(),
            )));
        }

        if max_pages_in_pool < 1 {
            return Err(DBError::from(StorageError::InvalidArgument(
                "invalid FileManager `max_pages_in_pool`: must be at least 1.".to_string(),
            )));
        }

        // --------------- NOW: ACTUALLY CREATING THE THING --------------- //
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let num_pages = file.metadata()?.len() / (page_size as u64);

        Ok(FileManager {
            file_path: path.to_string(),
            file,
            buffer_pool: HashMap::new(),
            page_size,
            max_pages_in_pool,
            num_pages,
        })
    }

    /// Reads and returns a page of bytes from the file given its `page_id`.
    /// If the page is not currently in the buffer pool, it will be loaded into
    /// the buffer pool and another page will be evicted. If a page cannot be
    /// evicted, then this function will return an error.
    pub fn read_page(&mut self, page_id: u64) -> DBResult<&[u8]> {
        // Add the page to the buffer pool if it is not already present
        if !self.buffer_pool.contains_key(&page_id) {
            // If buffer pool is full, evict a page
            while self.buffer_pool.len() >= self.max_pages_in_pool {
                self.evict_page()?;
            }

            // Read page from disk
            let mut page_data = vec![0; self.page_size];
            self.file
                .seek(SeekFrom::Start(page_id * (self.page_size as u64)))?;
            self.file.read_exact(&mut page_data)?;

            // Add page to buffer pool
            self.buffer_pool.insert(
                page_id,
                Page {
                    data: page_data,
                    dirty: false,
                    pin_count: 0,
                },
            );
        }

        Ok(&self.buffer_pool.get(&page_id).unwrap().data)

        // Look! a wonderful field of flowers!
        // ❃✿❀❃✿❀❃✿
        // ❀❃✿❀❃✿❀❃
        // ✿❀❃✿❀❃✿❀
    }

    /// Given bytes `data`, write to the page with the given `page_id` in the
    /// `FileManager`'s buffer pool. All data in the page will be overwritten,
    /// and the page will be marked dirty. `data` must have a length exactly
    /// equal to the `page_size` specified when creating this `FileManager`.
    ///
    /// If the page with the given `page_id` is not in the buffer pool, it is
    /// added to the buffer pool; if the pool is full and no page can be
    /// evicted, this function will return an error.
    ///
    /// This function guarantees that if an error is returned, the data has
    /// **not** been written to the pool.
    ///
    /// **NOTE:** This does **not** write the data to disk. In order to do
    /// that, call `flush_page` with the given `page_id`.
    fn write_page_to_pool(&mut self, page_id: u64, data: &[u8]) -> DBResult<()> {
        // Reject incorrect size data
        if data.len() != self.page_size {
            return Err(DBError::from(StorageError::InvalidArgument(format!(
                "length of data to write to page: {} did not match expected page size of {} bytes.",
                data.len(),
                self.page_size
            ))));
        }

        // Add page to buffer pool if it is not present
        if !self.buffer_pool.contains_key(&page_id) {
            // If the buffer pool is full, evict items until it has space
            while self.buffer_pool.len() >= self.max_pages_in_pool {
                self.evict_page()?;
            }

            // Add page to buffer pool
            self.buffer_pool.insert(
                page_id,
                Page {
                    data: vec![0; self.page_size],
                    dirty: false,
                    pin_count: 0,
                },
            );
        }

        // Unwrapping is safe because the item was just added to the pool
        let page = self.buffer_pool.get_mut(&page_id).unwrap();

        page.data.copy_from_slice(data);
        page.dirty = true;

        Ok(())
    }

    /// Allocate a new page in the buffer pool.
    ///
    /// If the buffer pool is full and no pages can be evicted, then an error
    /// will be returned.
    ///
    /// If the page is allocated in the buffer pool but cannot be written to
    /// disk, then the page will be removed from the buffer pool as if this
    /// function was never called.
    pub fn allocate_page(&mut self) -> DBResult<u64> {
        while self.num_pages >= (self.max_pages_in_pool as u64) {
            self.evict_page()?;
        }

        let page_id = self.num_pages + 1;

        let zeros = vec![0; self.page_size];
        self.write_page_to_pool(page_id, &zeros)?;

        // If everything was successful, increase `num_pages`
        self.num_pages += 1;
        Ok(self.num_pages)
    }

    /// Pins a page in memory; a page can only be removed from the buffer
    /// pool if no threads have pinned it.
    pub fn pin_page(&mut self, page_id: u64) -> DBResult<()> {
        if let Some(page) = self.buffer_pool.get_mut(&page_id) {
            page.pin_count += 1;
        } else {
            // Load the page into memory
            self.read_page(page_id)?;
            if let Some(page) = self.buffer_pool.get_mut(&page_id) {
                page.pin_count += 1;
            } else {
                return Err(DBError::from(StorageError::UnknownPage(page_id)));
            }
        }

        Ok(())
    }

    /// Unpins a page in the buffer pool and returns the number of pins the
    /// page has after unpinning. If the page is not present in the buffer
    /// pool, the function does nothing and returns `None`.
    pub fn unpin_page(&mut self, page_id: u64) -> Option<u16> {
        if let Some(page) = self.buffer_pool.get_mut(&page_id) {
            if page.pin_count > 0 {
                page.pin_count -= 1;
            }
            Some(page.pin_count)
        } else {
            None
        }
    }

    /// Flushes a specific page to disk if it is dirty.
    pub fn flush_page(&mut self, page_id: u64) -> DBResult<()> {
        if let Some(page) = self.buffer_pool.get_mut(&page_id) {
            if page.dirty {
                self.file
                    .seek(SeekFrom::Start(page_id * self.page_size as u64))?;
                self.file.write_all(&page.data)?;
                page.dirty = false;
            }
        }
        Ok(())
    }

    /// Flushes all pages in the buffer pool to disk if they are dirty.
    /// This should be used with caution, especially when writing concurrently,
    /// because it may disrupt ACID guarantees.
    pub fn flush_all_pages(&mut self) -> DBResult<()> {
        let page_ids: Vec<u64> = self.buffer_pool.keys().copied().collect();

        for page_id in page_ids {
            self.flush_page(page_id)?;
        }

        self.file.sync_all()?;
        Ok(())
    }

    /// Evicts a page from the buffer pool. This can only be done if there
    /// is some page in the pool with 0 pins.
    fn evict_page(&mut self) -> DBResult<()> {
        // Find an unpinned page to evict
        if let Some((&page_id, page)) = self
            .buffer_pool
            .iter()
            .find(|(_, page)| page.pin_count == 0)
        {
            // Flush if dirty
            if page.dirty {
                self.flush_page(page_id)?;
            }

            // Remove from buffer pool
            self.buffer_pool.remove(&page_id);
        } else {
            return Err(DBError::from(StorageError::BufferPoolFull(self.num_pages)));
        }

        Ok(())
    }
}

impl Drop for FileManager {
    fn drop(&mut self) {
        // Attempt to flush all pages when FileManager is dropped
        if let Err(e) = self.flush_all_pages() {
            eprintln!("Error flushing pages during shutdown: {}", e);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_manager_new_valid() -> DBResult<()> {
        let _ = FileManager::new("fm_test.db", 4092, 100)?;

        Ok(())
    }

    #[test]
    fn test_file_manager_new_errs() {
        // Test that an empty string is an invalid input
        let result = FileManager::new("", 4092, 100);
        assert!(result.is_err());

        // Test page size
        let result = FileManager::new("fm_test.db", 0, 402);
        assert!(result.is_err());

        // Test max_pages_in_pool
        let result = FileManager::new("fm_test.db", 4092, 0);
        assert!(result.is_err());
        // ^ It is not necessary to test negative values because
        // they are not `usize`s in Rust
    }
}
