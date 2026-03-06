#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

pub const BLOCK_SIZE: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockBackend {
    VirtioBlk,
    Ahci,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiskInfo {
    pub backend: BlockBackend,
    pub blocks: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockError {
    OutOfBounds,
    InvalidBuffer,
    Io,
}

pub trait BlockDevice {
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn block_count(&self) -> u64;

    fn read_block(&self, lba: u64, out: &mut [u8]) -> Result<(), BlockError>;

    fn write_block(&mut self, lba: u64, data: &[u8]) -> Result<(), BlockError>;

    fn info(&self) -> DiskInfo;
}

#[cfg(feature = "std")]
pub struct RamDisk {
    backend: BlockBackend,
    storage: std::vec::Vec<u8>,
}

#[cfg(feature = "std")]
impl RamDisk {
    pub fn new(backend: BlockBackend, blocks: u64) -> Self {
        Self {
            backend,
            storage: vec![0; blocks as usize * BLOCK_SIZE],
        }
    }
}

#[cfg(feature = "std")]
impl BlockDevice for RamDisk {
    fn block_count(&self) -> u64 {
        (self.storage.len() / BLOCK_SIZE) as u64
    }

    fn read_block(&self, lba: u64, out: &mut [u8]) -> Result<(), BlockError> {
        if out.len() != BLOCK_SIZE {
            return Err(BlockError::InvalidBuffer);
        }

        let start = lba as usize * BLOCK_SIZE;
        let end = start + BLOCK_SIZE;
        if end > self.storage.len() {
            return Err(BlockError::OutOfBounds);
        }

        out.copy_from_slice(&self.storage[start..end]);
        Ok(())
    }

    fn write_block(&mut self, lba: u64, data: &[u8]) -> Result<(), BlockError> {
        if data.len() != BLOCK_SIZE {
            return Err(BlockError::InvalidBuffer);
        }

        let start = lba as usize * BLOCK_SIZE;
        let end = start + BLOCK_SIZE;
        if end > self.storage.len() {
            return Err(BlockError::OutOfBounds);
        }

        self.storage[start..end].copy_from_slice(data);
        Ok(())
    }

    fn info(&self) -> DiskInfo {
        DiskInfo {
            backend: self.backend,
            blocks: self.block_count(),
        }
    }
}

#[cfg(feature = "std")]
pub struct FileDisk {
    backend: BlockBackend,
    file: std::fs::File,
    blocks: u64,
}

#[cfg(feature = "std")]
impl FileDisk {
    pub fn open_or_create(
        path: &std::path::Path,
        backend: BlockBackend,
        blocks: u64,
    ) -> Result<Self, BlockError> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|_| BlockError::Io)?;

        let required = blocks * BLOCK_SIZE as u64;
        let len = file.metadata().map_err(|_| BlockError::Io)?.len();
        if len < required {
            file.set_len(required).map_err(|_| BlockError::Io)?;
            file.sync_all().map_err(|_| BlockError::Io)?;
        }

        file.sync_data().map_err(|_| BlockError::Io)?;
        Ok(Self {
            backend,
            file,
            blocks,
        })
    }
}

#[cfg(feature = "std")]
impl BlockDevice for FileDisk {
    fn block_count(&self) -> u64 {
        self.blocks
    }

    fn read_block(&self, lba: u64, out: &mut [u8]) -> Result<(), BlockError> {
        use std::io::{Read, Seek, SeekFrom};

        if out.len() != BLOCK_SIZE {
            return Err(BlockError::InvalidBuffer);
        }
        if lba >= self.blocks {
            return Err(BlockError::OutOfBounds);
        }

        let mut f = self.file.try_clone().map_err(|_| BlockError::Io)?;
        f.seek(SeekFrom::Start(lba * BLOCK_SIZE as u64))
            .map_err(|_| BlockError::Io)?;
        f.read_exact(out).map_err(|_| BlockError::Io)?;
        Ok(())
    }

    fn write_block(&mut self, lba: u64, data: &[u8]) -> Result<(), BlockError> {
        use std::io::{Seek, SeekFrom, Write};

        if data.len() != BLOCK_SIZE {
            return Err(BlockError::InvalidBuffer);
        }
        if lba >= self.blocks {
            return Err(BlockError::OutOfBounds);
        }

        self.file
            .seek(SeekFrom::Start(lba * BLOCK_SIZE as u64))
            .map_err(|_| BlockError::Io)?;
        self.file.write_all(data).map_err(|_| BlockError::Io)?;
        self.file.sync_data().map_err(|_| BlockError::Io)?;
        Ok(())
    }

    fn info(&self) -> DiskInfo {
        DiskInfo {
            backend: self.backend,
            blocks: self.blocks,
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn ramdisk_read_write_roundtrip() {
        let mut d = RamDisk::new(BlockBackend::VirtioBlk, 8);
        let mut w = [0u8; BLOCK_SIZE];
        w[..4].copy_from_slice(b"FSOK");
        d.write_block(0, &w).expect("write");

        let mut r = [0u8; BLOCK_SIZE];
        d.read_block(0, &mut r).expect("read");
        assert_eq!(&r[..4], b"FSOK");
    }
}
