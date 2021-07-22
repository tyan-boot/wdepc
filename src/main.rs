mod ffi;

use ffi::*;

use libc::{open, ioctl};
use std::ffi::CString;
use anyhow::Result;
use std::ptr::null_mut;

fn main() -> Result<()> {
    let dev = CString::new("/dev/sda")?;

    unsafe {
        let fd = open(dev.as_ptr(), libc::O_RDWR);

        anyhow::ensure!(fd > 0, "open dev failed {}", std::io::Error::from_raw_os_error(*libc::__errno_location()));
        let mut cdb = build_ata_passthrough(AtaCmd::ReadLogExtDma, Protocol::InDma, 1, 8, 0);
        println!("cdb = {:02x?}", cdb);
        let mut hdr = SgIoHdr::default();
        let mut sense = [0u8; 32];
        let mut buffer = [0u8; 1024];

        hdr.cmd_len = cdb.len() as u8;

        hdr.mx_sb_len = sense.len() as u8;
        hdr.dxfer_direction = -3;
        hdr.dxfer_len = buffer.len() as u32;
        hdr.dxferp = buffer.as_mut_ptr() as *mut libc::c_void;
        hdr.cmdp = cdb.as_mut_ptr();
        hdr.sbp = sense.as_mut_ptr();

        hdr.timeout = 6000;

        let r = ioctl(fd, SG_IO, &hdr);
        dbg!(r);
        dbg!(hdr);
        dbg!(sense);
        println!("buffer = {:02X?}", buffer);
    }

    Ok(())
}
