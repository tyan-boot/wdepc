use std::ptr::null_mut;
use libc::{open, ioctl};
use libc::{c_int, c_uchar, c_uint, c_ushort, c_void, c_ulong};

pub const ATA_16_LEN: usize = 16;
pub const ATA_16: u8 = 0x85;

pub const SG_IO: c_ulong = 0x2285;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SgIoHdr {
    pub interface_id: c_int,
    pub dxfer_direction: c_int,
    pub cmd_len: c_uchar,
    pub mx_sb_len: c_uchar,
    pub iovec_count: c_ushort,
    pub dxfer_len: c_uint,
    pub dxferp: *mut c_void,
    pub cmdp: *mut c_uchar,
    pub sbp: *mut c_uchar,
    pub timeout: c_uint,
    pub flags: c_uint,
    pub pack_id: c_int,
    pub usr_ptr: *mut c_void,
    pub status: c_uchar,
    pub masked_status: c_uchar,
    pub msg_status: c_uchar,
    pub sb_len_wr: c_uchar,
    pub host_status: c_ushort,
    pub driver_status: c_ushort,
    pub resid: c_int,
    pub duration: c_uint,
    pub info: c_uint,
}

impl Default for SgIoHdr {
    fn default() -> Self {
        SgIoHdr {
            interface_id: 'S' as c_int,
            dxfer_direction: 0,
            cmd_len: 0,
            mx_sb_len: 0,
            iovec_count: 0,
            dxfer_len: 0,
            dxferp: null_mut(),
            cmdp: null_mut(),
            sbp: null_mut(),
            timeout: 0,
            flags: 0,
            pack_id: 0,
            usr_ptr: null_mut(),
            status: 0,
            masked_status: 0,
            msg_status: 0,
            sb_len_wr: 0,
            host_status: 0,
            driver_status: 0,
            resid: 0,
            duration: 0,
            info: 0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum AtaCmd {
    ReadLogExt = 0x2f,
    ReadLogExtDma = 0x47,
}

impl AtaCmd {
    pub fn ck_cond(&self) -> bool {
        match self {
            AtaCmd::ReadLogExt => false,
            AtaCmd::ReadLogExtDma => false
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Protocol {
    InDma = 10 << 1,
    OutDma = 11 << 1,
    None = 3 << 1,
    PioIn = 4 << 1,
    PioOut = 5 << 1,
    Dma = 6 << 1,
}

impl Protocol {
    pub fn t_dir(&self) -> u8 {
        match self {
            Protocol::InDma => 1,
            Protocol::OutDma => 0,
            Protocol::None => 0,
            Protocol::PioIn => 1,
            Protocol::PioOut => 0,
            Protocol::Dma => 0,
        }
    }
}

pub fn build_ata_passthrough(cmd: AtaCmd, protocol: Protocol, sector_count: c_ushort, sector_number: c_ushort, cylinder: c_ushort) -> [u8; ATA_16_LEN] {
    let mut cdb: [u8; ATA_16_LEN] = [0; ATA_16_LEN];
    cdb[0] = ATA_16;  // opcode
    cdb[1] = protocol as u8 | 1;  // proto, extend
    // off_line = 0, ck_cond = ?, t_dir = ?, byt_blok = 1, t_length = 02h(sector count)
    cdb[2] = 0b000 << 6 | if cmd.ck_cond() { 1 << 5 } else { 0 } | protocol.t_dir() << 3 | 1 << 2 | 0x2;
    cdb[3] = 0;
    cdb[4] = 0;

    // sector_count
    cdb[5] = (sector_count >> 8) as u8;
    cdb[6] = sector_count as u8;

    // lba_low
    cdb[7] = (sector_number >> 8) as u8;
    cdb[8] = sector_number as u8;

    //lba_mid
    cdb[9] = (cylinder >> 8) as u8;
    cdb[10] = cylinder as u8;

    // lba_high
    cdb[11] = 0;
    cdb[12] = 0;

    // device
    cdb[13] = 0xa0;

    // command
    cdb[14] = cmd as u8;

    // control
    cdb[15] = 0;

    cdb
}

