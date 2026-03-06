use sadas_block::{BlockBackend, BlockDevice, BlockError, FileDisk, BLOCK_SIZE};

const MAGIC: &[u8; 8] = b"SADASFS1";
const DIR_ENTRIES: usize = 64;
const NAME_LEN: usize = 32;
const ENTRY_SIZE: usize = 64;
const DIR_BLOCKS: u64 = 8;
const DATA_START_BLOCK: u64 = 1 + DIR_BLOCKS;
const ROOTFS_BLOCKS: u64 = 4096;

pub const ROOT_MOUNT: &str = "/";
pub const ESP_MOUNT: &str = "/efi";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub start_block: u64,
    pub size: u64,
}

#[derive(Debug)]
pub enum FsError {
    Block(BlockError),
    NotFormatted,
    CorruptMetadata,
    NameTooLong,
    NoSpace,
    NotFound,
}

impl From<BlockError> for FsError {
    fn from(value: BlockError) -> Self {
        FsError::Block(value)
    }
}

pub struct SimpleFs<D: BlockDevice> {
    disk: D,
}

impl<D: BlockDevice> SimpleFs<D> {
    pub fn new(disk: D) -> Self {
        Self { disk }
    }

    pub fn into_inner(self) -> D {
        self.disk
    }

    pub fn format(&mut self) -> Result<(), FsError> {
        let mut superblock = [0u8; BLOCK_SIZE];
        superblock[..8].copy_from_slice(MAGIC);
        self.disk.write_block(0, &superblock)?;

        let empty = [0u8; BLOCK_SIZE];
        for lba in 1..=DIR_BLOCKS {
            self.disk.write_block(lba, &empty)?;
        }
        Ok(())
    }

    pub fn check(&self) -> Result<(), FsError> {
        let mut sb = [0u8; BLOCK_SIZE];
        self.disk.read_block(0, &mut sb)?;
        if &sb[..8] != MAGIC {
            return Err(FsError::NotFormatted);
        }
        Ok(())
    }

    pub fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), FsError> {
        self.check()?;
        if name.is_empty() || name.len() > NAME_LEN {
            return Err(FsError::NameTooLong);
        }

        let mut entries = self.read_dir_table()?;
        let idx = entries
            .iter()
            .position(|e| e.as_ref().map(|x| x.name == name).unwrap_or(false))
            .or_else(|| entries.iter().position(|e| e.is_none()))
            .ok_or(FsError::NoSpace)?;

        let start_block = self.find_data_start(&entries);
        let needed_blocks = blocks_for_len(data.len());
        let end_block = start_block + needed_blocks as u64;
        if end_block > self.disk.block_count() {
            return Err(FsError::NoSpace);
        }

        for i in 0..needed_blocks {
            let mut block = [0u8; BLOCK_SIZE];
            let begin = i * BLOCK_SIZE;
            let end = core::cmp::min(begin + BLOCK_SIZE, data.len());
            if end > begin {
                block[..(end - begin)].copy_from_slice(&data[begin..end]);
            }
            self.disk.write_block(start_block + i as u64, &block)?;
        }

        entries[idx] = Some(DirEntry {
            name: name.to_string(),
            start_block,
            size: data.len() as u64,
        });
        self.write_dir_table(&entries)?;
        Ok(())
    }

    pub fn read_file(&self, name: &str) -> Result<Vec<u8>, FsError> {
        self.check()?;
        let entries = self.read_dir_table()?;
        let entry = entries
            .iter()
            .flatten()
            .find(|e| e.name == name)
            .ok_or(FsError::NotFound)?;

        let mut out = vec![0u8; entry.size as usize];
        let needed_blocks = blocks_for_len(entry.size as usize);
        for i in 0..needed_blocks {
            let mut block = [0u8; BLOCK_SIZE];
            self.disk
                .read_block(entry.start_block + i as u64, &mut block)?;
            let begin = i * BLOCK_SIZE;
            let end = core::cmp::min(begin + BLOCK_SIZE, out.len());
            if end > begin {
                out[begin..end].copy_from_slice(&block[..(end - begin)]);
            }
        }
        Ok(out)
    }

    pub fn list_dir(&self) -> Result<Vec<String>, FsError> {
        self.check()?;
        let entries = self.read_dir_table()?;
        Ok(entries.into_iter().flatten().map(|e| e.name).collect())
    }

    fn read_dir_table(&self) -> Result<Vec<Option<DirEntry>>, FsError> {
        let mut raw = vec![0u8; (DIR_BLOCKS as usize) * BLOCK_SIZE];
        for i in 0..DIR_BLOCKS as usize {
            let mut block = [0u8; BLOCK_SIZE];
            self.disk.read_block(1 + i as u64, &mut block)?;
            let start = i * BLOCK_SIZE;
            raw[start..start + BLOCK_SIZE].copy_from_slice(&block);
        }

        let mut entries = vec![None; DIR_ENTRIES];
        for (i, slot) in entries.iter_mut().enumerate() {
            let base = i * ENTRY_SIZE;
            if raw[base] == 0 {
                continue;
            }
            let name_end = raw[base + 1..base + 1 + NAME_LEN]
                .iter()
                .position(|b| *b == 0)
                .unwrap_or(NAME_LEN);
            let name = core::str::from_utf8(&raw[base + 1..base + 1 + name_end])
                .map_err(|_| FsError::CorruptMetadata)?
                .to_string();

            let start_range = &raw[base + 33..base + 41];
            let size_range = &raw[base + 41..base + 49];
            if start_range.len() != 8 || size_range.len() != 8 {
                return Err(FsError::CorruptMetadata);
            }

            let mut start_bytes = [0u8; 8];
            start_bytes.copy_from_slice(start_range);
            let mut size_bytes = [0u8; 8];
            size_bytes.copy_from_slice(size_range);

            *slot = Some(DirEntry {
                name,
                start_block: u64::from_le_bytes(start_bytes),
                size: u64::from_le_bytes(size_bytes),
            });
        }
        Ok(entries)
    }

    fn write_dir_table(&mut self, entries: &[Option<DirEntry>]) -> Result<(), FsError> {
        let mut raw = vec![0u8; (DIR_BLOCKS as usize) * BLOCK_SIZE];
        for (i, entry) in entries.iter().enumerate() {
            if let Some(e) = entry {
                let base = i * ENTRY_SIZE;
                raw[base] = 1;
                let name_bytes = e.name.as_bytes();
                let n = core::cmp::min(NAME_LEN, name_bytes.len());
                raw[base + 1..base + 1 + n].copy_from_slice(&name_bytes[..n]);
                raw[base + 33..base + 41].copy_from_slice(&e.start_block.to_le_bytes());
                raw[base + 41..base + 49].copy_from_slice(&e.size.to_le_bytes());
            }
        }

        for i in 0..DIR_BLOCKS as usize {
            let mut block = [0u8; BLOCK_SIZE];
            let start = i * BLOCK_SIZE;
            block.copy_from_slice(&raw[start..start + BLOCK_SIZE]);
            self.disk.write_block(1 + i as u64, &block)?;
        }
        Ok(())
    }

    fn find_data_start(&self, entries: &[Option<DirEntry>]) -> u64 {
        let mut max_end = DATA_START_BLOCK;
        for e in entries.iter().flatten() {
            let end = e.start_block + blocks_for_len(e.size as usize) as u64;
            if end > max_end {
                max_end = end;
            }
        }
        max_end
    }
}

fn blocks_for_len(len: usize) -> usize {
    if len == 0 {
        1
    } else {
        len.div_ceil(BLOCK_SIZE)
    }
}

pub fn open_default_root(path: &std::path::Path) -> Result<SimpleFs<FileDisk>, FsError> {
    let disk = FileDisk::open_or_create(path, BlockBackend::VirtioBlk, ROOTFS_BLOCKS)?;
    let mut fs = SimpleFs::new(disk);
    if fs.check().is_err() {
        fs.format()?;
    }
    Ok(fs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sadas_block::RamDisk;

    #[test]
    fn persists_file_content_across_remount() {
        let disk = RamDisk::new(BlockBackend::VirtioBlk, 1024);
        let disk = {
            let mut fs = SimpleFs::new(disk);
            fs.format().expect("format");
            fs.write_file("hello.txt", b"persist me").expect("write");
            fs.into_inner()
        };

        let fs = SimpleFs::new(disk);
        let data = fs.read_file("hello.txt").expect("read");
        assert_eq!(&data, b"persist me");
    }
}
