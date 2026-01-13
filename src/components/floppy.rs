//! Floppy disk image handling
//!
//! Supports raw sector images (.img) with auto-detected geometry.
//! Common formats: 160KB, 180KB, 320KB, 360KB, 720KB, 1.2MB, 1.44MB

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

// =============================================================================
// Constants
// =============================================================================

/// Standard bytes per sector for IBM PC floppy disks
pub const BYTES_PER_SECTOR: u16 = 512;

// =============================================================================
// DiskGeometry
// =============================================================================

/// Floppy disk geometry (CHS layout)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskGeometry {
    /// Number of cylinders (tracks per side)
    pub cylinders: u8,
    /// Number of heads (sides)
    pub heads: u8,
    /// Sectors per track
    pub sectors_per_track: u8,
    /// Bytes per sector (typically 512)
    pub bytes_per_sector: u16,
}

impl DiskGeometry {
    /// Create a new geometry
    pub fn new(cylinders: u8, heads: u8, sectors_per_track: u8, bytes_per_sector: u16) -> Self {
        Self {
            cylinders,
            heads,
            sectors_per_track,
            bytes_per_sector,
        }
    }

    /// Total size in bytes
    pub fn total_size(&self) -> usize {
        (self.cylinders as usize)
            * (self.heads as usize)
            * (self.sectors_per_track as usize)
            * (self.bytes_per_sector as usize)
    }

    /// Detect geometry from file size
    pub fn from_size(size: usize) -> Option<Self> {
        match size {
            // 5.25" Single-sided Double-density
            163_840 => Some(Self::new(40, 1, 8, 512)), // 160KB
            184_320 => Some(Self::new(40, 1, 9, 512)), // 180KB

            // 5.25" Double-sided Double-density
            327_680 => Some(Self::new(40, 2, 8, 512)), // 320KB
            368_640 => Some(Self::new(40, 2, 9, 512)), // 360KB

            // 3.5" Double-sided Double-density
            737_280 => Some(Self::new(80, 2, 9, 512)), // 720KB

            // 5.25" Double-sided High-density
            1_228_800 => Some(Self::new(80, 2, 15, 512)), // 1.2MB

            // 3.5" Double-sided High-density
            1_474_560 => Some(Self::new(80, 2, 18, 512)), // 1.44MB

            // 3.5" Extended density (rare)
            2_949_120 => Some(Self::new(80, 2, 36, 512)), // 2.88MB

            _ => None,
        }
    }

    /// Convert CHS address to linear byte offset
    pub fn chs_to_offset(&self, cylinder: u8, head: u8, sector: u8) -> Option<usize> {
        // Sector numbers are 1-based
        if sector == 0 || sector > self.sectors_per_track {
            return None;
        }
        if head >= self.heads {
            return None;
        }
        if cylinder >= self.cylinders {
            return None;
        }

        let sector_index = (sector - 1) as usize; // Convert to 0-based
        let head_index = head as usize;
        let cylinder_index = cylinder as usize;

        // LBA = (C * heads + H) * sectors_per_track + (S - 1)
        let lba = (cylinder_index * (self.heads as usize) + head_index)
            * (self.sectors_per_track as usize)
            + sector_index;

        Some(lba * (self.bytes_per_sector as usize))
    }
}

// =============================================================================
// FloppyDisk
// =============================================================================

/// A floppy disk image
#[derive(Debug)]
pub struct FloppyDisk {
    /// Raw sector data
    data: Vec<u8>,
    /// Disk geometry
    geometry: DiskGeometry,
    /// Write protection flag
    write_protected: bool,
    /// Modified since load
    dirty: bool,
    /// Source file path (for saving)
    path: Option<PathBuf>,
}

impl FloppyDisk {
    /// Create a new empty floppy disk with the given geometry
    pub fn new(geometry: DiskGeometry) -> Self {
        let size = geometry.total_size();
        Self {
            data: vec![0; size],
            geometry,
            write_protected: false,
            dirty: false,
            path: None,
        }
    }

    /// Load a floppy disk image from file
    ///
    /// Geometry is auto-detected from file size.
    /// The disk is read-only by default; use `set_write_protected(false)` to enable writes.
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let geometry = DiskGeometry::from_size(data.len()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unknown disk image size: {} bytes. Expected standard floppy size.",
                    data.len()
                ),
            )
        })?;

        Ok(Self {
            data,
            geometry,
            write_protected: true, // Read-only by default
            dirty: false,
            path: Some(path.to_path_buf()),
        })
    }

    /// Get the disk geometry
    pub fn geometry(&self) -> DiskGeometry {
        self.geometry
    }

    /// Check if the disk is write-protected
    pub fn is_write_protected(&self) -> bool {
        self.write_protected
    }

    /// Set write protection status
    pub fn set_write_protected(&mut self, protected: bool) {
        self.write_protected = protected;
    }

    /// Check if the disk has been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Read a sector from the disk
    ///
    /// Returns None if the CHS address is invalid.
    pub fn read_sector(&self, cylinder: u8, head: u8, sector: u8) -> Option<&[u8]> {
        let offset = self.geometry.chs_to_offset(cylinder, head, sector)?;
        let end = offset + (self.geometry.bytes_per_sector as usize);

        if end <= self.data.len() {
            Some(&self.data[offset..end])
        } else {
            None
        }
    }

    /// Write a sector to the disk
    ///
    /// Returns an error if write-protected or CHS address is invalid.
    pub fn write_sector(
        &mut self,
        cylinder: u8,
        head: u8,
        sector: u8,
        data: &[u8],
    ) -> io::Result<()> {
        if self.write_protected {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Disk is write-protected",
            ));
        }

        let offset = self.geometry.chs_to_offset(cylinder, head, sector).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid CHS address: C={}, H={}, S={}", cylinder, head, sector),
            )
        })?;

        let sector_size = self.geometry.bytes_per_sector as usize;
        let end = offset + sector_size;

        if end > self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Sector extends beyond disk image",
            ));
        }

        // Copy data, padding or truncating as needed
        let copy_len = data.len().min(sector_size);
        self.data[offset..offset + copy_len].copy_from_slice(&data[..copy_len]);

        // Zero-pad if data is shorter than sector
        if copy_len < sector_size {
            self.data[offset + copy_len..end].fill(0);
        }

        self.dirty = true;
        Ok(())
    }

    /// Format a track (fill all sectors with a pattern)
    pub fn format_track(
        &mut self,
        cylinder: u8,
        head: u8,
        fill_byte: u8,
    ) -> io::Result<()> {
        if self.write_protected {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Disk is write-protected",
            ));
        }

        let sector_size = self.geometry.bytes_per_sector as usize;
        let fill_data = vec![fill_byte; sector_size];

        for sector in 1..=self.geometry.sectors_per_track {
            if let Some(offset) = self.geometry.chs_to_offset(cylinder, head, sector) {
                let end = offset + sector_size;
                if end <= self.data.len() {
                    self.data[offset..end].copy_from_slice(&fill_data);
                }
            }
        }

        self.dirty = true;
        Ok(())
    }

    /// Save changes back to the source file
    pub fn save(&mut self) -> io::Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No source file path")
        })?;

        if self.write_protected {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Disk is write-protected",
            ));
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;

        file.write_all(&self.data)?;
        self.dirty = false;
        Ok(())
    }

    /// Get the file path (if loaded from file)
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_sizes() {
        // 360KB
        let g = DiskGeometry::new(40, 2, 9, 512);
        assert_eq!(g.total_size(), 368_640);

        // 720KB
        let g = DiskGeometry::new(80, 2, 9, 512);
        assert_eq!(g.total_size(), 737_280);

        // 1.44MB
        let g = DiskGeometry::new(80, 2, 18, 512);
        assert_eq!(g.total_size(), 1_474_560);
    }

    #[test]
    fn test_geometry_detection() {
        assert_eq!(
            DiskGeometry::from_size(368_640),
            Some(DiskGeometry::new(40, 2, 9, 512))
        );
        assert_eq!(
            DiskGeometry::from_size(737_280),
            Some(DiskGeometry::new(80, 2, 9, 512))
        );
        assert_eq!(
            DiskGeometry::from_size(1_474_560),
            Some(DiskGeometry::new(80, 2, 18, 512))
        );
        assert_eq!(DiskGeometry::from_size(12345), None);
    }

    #[test]
    fn test_chs_to_offset() {
        let g = DiskGeometry::new(40, 2, 9, 512);

        // First sector (C=0, H=0, S=1)
        assert_eq!(g.chs_to_offset(0, 0, 1), Some(0));

        // Second sector (C=0, H=0, S=2)
        assert_eq!(g.chs_to_offset(0, 0, 2), Some(512));

        // First sector of second side (C=0, H=1, S=1)
        assert_eq!(g.chs_to_offset(0, 1, 1), Some(9 * 512));

        // First sector of second track (C=1, H=0, S=1)
        assert_eq!(g.chs_to_offset(1, 0, 1), Some(2 * 9 * 512));

        // Invalid sector 0
        assert_eq!(g.chs_to_offset(0, 0, 0), None);

        // Invalid sector > sectors_per_track
        assert_eq!(g.chs_to_offset(0, 0, 10), None);

        // Invalid head
        assert_eq!(g.chs_to_offset(0, 2, 1), None);

        // Invalid cylinder
        assert_eq!(g.chs_to_offset(40, 0, 1), None);
    }

    #[test]
    fn test_floppy_new() {
        let g = DiskGeometry::new(40, 2, 9, 512);
        let disk = FloppyDisk::new(g);

        assert_eq!(disk.geometry(), g);
        assert_eq!(disk.data.len(), 368_640);
        assert!(!disk.is_write_protected());
        assert!(!disk.is_dirty());
    }

    #[test]
    fn test_read_write_sector() {
        let g = DiskGeometry::new(40, 2, 9, 512);
        let mut disk = FloppyDisk::new(g);

        // Write some data
        let test_data = vec![0xAA; 512];
        disk.write_sector(0, 0, 1, &test_data).unwrap();

        assert!(disk.is_dirty());

        // Read it back
        let read_data = disk.read_sector(0, 0, 1).unwrap();
        assert_eq!(read_data, &test_data[..]);

        // Verify other sectors are still zeros
        let other_sector = disk.read_sector(0, 0, 2).unwrap();
        assert!(other_sector.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_write_protected() {
        let g = DiskGeometry::new(40, 2, 9, 512);
        let mut disk = FloppyDisk::new(g);
        disk.set_write_protected(true);

        let test_data = vec![0xAA; 512];
        let result = disk.write_sector(0, 0, 1, &test_data);

        assert!(result.is_err());
        assert!(!disk.is_dirty());
    }

    #[test]
    fn test_format_track() {
        let g = DiskGeometry::new(40, 2, 9, 512);
        let mut disk = FloppyDisk::new(g);

        disk.format_track(0, 0, 0xF6).unwrap();

        // All 9 sectors on track 0, head 0 should be filled
        for sector in 1..=9 {
            let data = disk.read_sector(0, 0, sector).unwrap();
            assert!(data.iter().all(|&b| b == 0xF6));
        }

        // Other tracks should still be zeros
        let other_track = disk.read_sector(1, 0, 1).unwrap();
        assert!(other_track.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_invalid_chs() {
        let g = DiskGeometry::new(40, 2, 9, 512);
        let disk = FloppyDisk::new(g);

        // Invalid reads return None
        assert!(disk.read_sector(0, 0, 0).is_none()); // Sector 0 invalid
        assert!(disk.read_sector(0, 0, 10).is_none()); // Sector > SPT
        assert!(disk.read_sector(40, 0, 1).is_none()); // Cylinder out of range
    }
}
