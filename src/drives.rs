mod drive_imp;
pub mod drive_mod;

use crate::drives::drive_mod::Drive;
use std::ffi::OsString;
use std::path::PathBuf;
use sysinfo::{Disk, Disks};

pub fn get_removable_disks() -> Vec<Drive> {
    let disks_list = Disks::new_with_refreshed_list();
    let mut drives = Vec::new();

    let disks_list: Vec<&Disk> = disks_list
        .into_iter()
        .filter(|e| e.is_removable())
        .collect();
    for disk in disks_list {
        drives.push(Drive::new(
            get_disk_label(disk).into_string().unwrap(),
            PathBuf::from(disk.mount_point()),
        ))
    }

    drives
}

#[cfg(target_os = "windows")]
fn get_disk_label(disk: &Disk) -> OsString {
    disk.name().to_os_string()
}

#[cfg(target_os = "linux")]
fn get_disk_label(disk: &Disk) -> OsString {
    OsString::from(
        disk.mount_point()
            .display()
            .to_string()
            .split("/")
            .last()
            .unwrap(),
    )
}
