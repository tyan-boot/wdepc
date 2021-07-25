use std::convert::TryInto;
use std::ffi::CString;

use anyhow::{Context, Result};
use libc::c_int;
use libc::{ioctl, open, O_RDONLY};

use crate::ffi::{
    build_ata_passthrough12, build_ata_passthrough16, AtaCmd, Protocol, SgIoHdr, SG_DXFER_FROM_DEV,
    SG_DXFER_NONE, SG_DXFER_TO_DEV, SG_IO,
};

pub struct Device {
    fd: c_int,
}

#[derive(Debug, Copy, Clone)]
pub enum PowerMode {
    Active,
    IdleA,
    IdleB,
    IdleC,
    StandbyY,
    StandbyZ,
    Unknown,
}

impl PowerMode {
    pub fn id(&self) -> u8 {
        match self {
            PowerMode::Active => 0x81,
            PowerMode::IdleA => 0x81,
            PowerMode::IdleB => 0x82,
            PowerMode::IdleC => 0x83,
            PowerMode::StandbyY => 0x01,
            PowerMode::StandbyZ => 0x00,
            PowerMode::Unknown => panic!("unknown power mode"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PowerCondDescriptor {
    pub supported: bool,
    pub savable: bool,
    pub changeable: bool,

    pub default_enable: bool,
    pub saved_enable: bool,
    pub current_enable: bool,

    pub default_timer: u32,
    pub saved_timer: u32,
    pub current_timer: u32,

    pub recovery_time: u32,

    pub min_timer: u32,
    pub max_timer: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct EPCSetting {
    pub idle_a: PowerCondDescriptor,
    pub idle_b: PowerCondDescriptor,
    pub idle_c: PowerCondDescriptor,

    pub standby_y: PowerCondDescriptor,
    pub standby_z: PowerCondDescriptor,
}

impl Device {
    /// Open device with given path
    ///
    /// **Require root**
    pub fn open(device: impl AsRef<str>) -> Result<Device> {
        let device = device.as_ref();

        let device_ffi = CString::new(device)?;

        let fd = unsafe { open(device_ffi.as_ptr(), O_RDONLY) };

        anyhow::ensure!(
            fd > 0,
            "open {} failed: {}",
            device,
            std::io::Error::last_os_error()
        );

        Ok(Device { fd })
    }

    /// Query current power mode
    pub fn query_mode(&self) -> Result<PowerMode> {
        // todo: check EPC enable
        let mut cdb = build_ata_passthrough12(AtaCmd::CheckPowerMode, Protocol::None, 0, 0, 0, 0);
        let (_hdr, sense) = self.sg_io(&mut cdb, None, None)?;

        let sense = parse_sense(&sense)?;
        match sense.sector_count {
            0xff => Ok(PowerMode::Active),
            0x81 => Ok(PowerMode::IdleA),
            0x82 => Ok(PowerMode::IdleB),
            0x83 => Ok(PowerMode::IdleC),
            0x01 => Ok(PowerMode::StandbyY),
            0x00 => Ok(PowerMode::StandbyZ),
            _ => Ok(PowerMode::Unknown),
        }
    }

    /// Query device EPC setting
    pub fn query_epc_setting(&self) -> Result<EPCSetting> {
        let pcl = self.read_log_dma_ext(0x08)?;

        let idle_power_cond = &pcl[0..512];
        let standby_power_cond = &pcl[512..];

        let idle_a = parse_power_cond_desc(&idle_power_cond[0..=63]);
        let idle_b = parse_power_cond_desc(&idle_power_cond[64..=127]);
        let idle_c = parse_power_cond_desc(&idle_power_cond[128..=191]);

        let standby_y = parse_power_cond_desc(&standby_power_cond[384..=447]);
        let standby_z = parse_power_cond_desc(&standby_power_cond[448..=511]);

        Ok(EPCSetting {
            idle_a,
            idle_b,
            idle_c,
            standby_y,
            standby_z,
        })
    }

    fn read_log_dma_ext(&self, page: u8) -> Result<Vec<u8>> {
        let general_log = self.read_general_log();
        let max_size = general_log[page as usize * 2] as u16
            | (general_log[page as usize * 2 + 1] as u16) << 8;

        let mut buffer = Vec::with_capacity(512 * max_size as usize);
        buffer.resize(512 * max_size as usize, 0);
        let mut cdb = build_ata_passthrough16(
            AtaCmd::ReadLogExtDma,
            Protocol::InDma,
            0,
            max_size,
            page as u16,
            0,
        );

        let (_hdr, _sense) = self.sg_io(&mut cdb, None, Some(&mut buffer))?;

        Ok(buffer)
    }

    fn sg_io(
        &self,
        cdb: &mut [u8],
        in_data: Option<&[u8]>,
        out_data: Option<&mut [u8]>,
    ) -> Result<(SgIoHdr, [u8; 32])> {
        let mut hdr = SgIoHdr::default();
        let mut sense = [0u8; 32];

        hdr.cmd_len = cdb.len() as u8;

        hdr.mx_sb_len = sense.len() as u8;
        hdr.cmdp = cdb.as_mut_ptr();
        hdr.sbp = sense.as_mut_ptr();

        match (in_data, out_data) {
            (Some(in_data), None) => {
                hdr.dxfer_direction = SG_DXFER_TO_DEV;
                hdr.dxfer_len = in_data.len() as u32;
                hdr.dxferp = in_data.as_ptr() as *mut _; // safe, no write to in_data
            }
            (None, Some(out_data)) => {
                hdr.dxfer_direction = SG_DXFER_FROM_DEV;
                hdr.dxfer_len = out_data.len() as u32;
                hdr.dxferp = out_data.as_mut_ptr() as *mut _;
            }
            (None, None) => {
                hdr.dxfer_direction = SG_DXFER_NONE;
            }
            (Some(_), Some(_)) => {
                anyhow::bail!("only one direction allowed");
            }
        }

        let _r = unsafe { ioctl(self.fd, SG_IO, &mut hdr) };
        // todo check r

        Ok((hdr, sense))
    }

    fn read_general_log(&self) -> &[u8] {
        use once_cell::sync::OnceCell;
        static GENERAL_LOG: OnceCell<[u8; 512]> = OnceCell::new();

        GENERAL_LOG.get_or_init(|| {
            let mut buffer = [0u8; 512];
            let mut cdb =
                build_ata_passthrough16(AtaCmd::ReadLogExtDma, Protocol::InDma, 0, 1, 0, 0);
            let (_hdr, _sense) = self
                .sg_io(&mut cdb, None, Some(&mut buffer))
                .context("unable do sg_io")
                .unwrap();

            buffer
        })
    }

    /// Set device to specific power mode
    pub fn goto_cond(&mut self, mode: PowerMode) -> Result<()> {
        let mut cdb = build_ata_passthrough12(
            AtaCmd::SetFeature,
            Protocol::None,
            0b0100_1010,
            mode.id() as u16,
            1,
            0,
        );
        let (_hdr, _sense) = self.sg_io(&mut cdb, None, None)?;

        Ok(())
    }

    /// Set specific power mode timer
    ///
    /// if `enable` set true, enable current timer else disable
    ///
    /// if `save` set true, save current timer setting
    pub fn set_timer(
        &mut self,
        mode: PowerMode,
        timer: u16,
        enable: bool,
        save: bool,
    ) -> Result<()> {
        let enable = if enable { 1 } else { 0 };
        let save = if save { 1 } else { 0 };
        let sector_number = enable << 5 | save << 4 | 0x02;
        let mut cdb = build_ata_passthrough12(
            AtaCmd::SetFeature,
            Protocol::None,
            0b0100_1010,
            mode.id() as u16,
            sector_number,
            timer,
        );

        let (_hdr, _sense) = self.sg_io(&mut cdb, None, None)?;

        Ok(())
    }

    /// Set specific power mode state
    ///
    /// if `enable` set to true, enable specific power mode
    ///
    /// if `save` set to true, save setting
    pub fn set_state(&mut self, mode: PowerMode, enable: bool, save: bool) -> Result<()> {
        let enable = if enable { 1 } else { 0 };
        let save = if save { 1 } else { 0 };
        let sector_number = enable << 5 | save << 4 | 0x02;
        let mut cdb = build_ata_passthrough12(
            AtaCmd::SetFeature,
            Protocol::None,
            0b0100_1010,
            mode.id() as u16,
            sector_number,
            0,
        );

        let (_hdr, _sense) = self.sg_io(&mut cdb, None, None)?;

        Ok(())
    }

    /// Enable EPC feature
    ///
    /// **This will disable APM**
    pub fn enable_epc(&mut self) -> Result<()> {
        let mut cdb =
            build_ata_passthrough12(AtaCmd::SetFeature, Protocol::None, 0b0100_1010, 0, 0x04, 0);

        let (_hdr, _sense) = self.sg_io(&mut cdb, None, None)?;

        Ok(())
    }

    /// Disable EPC feature
    ///
    /// **This doesn't re-enable APM, you must enable APM manually on demand**
    pub fn disable_epc(&mut self) -> Result<()> {
        let mut cdb =
            build_ata_passthrough12(AtaCmd::SetFeature, Protocol::None, 0b0100_1010, 0, 0x05, 0);

        let (_hdr, _sense) = self.sg_io(&mut cdb, None, None)?;

        Ok(())
    }

    pub fn restore(&mut self, mode: PowerMode, default: bool, save: bool) -> Result<()> {
        let default = if default { 1 } else { 0 };
        let save = if save { 1 } else { 0 };
        let sector_number = default << 6 | save << 4;

        let mut cdb = build_ata_passthrough12(
            AtaCmd::SetFeature,
            Protocol::None,
            0b0100_1010,
            mode.id() as u16,
            sector_number,
            0,
        );
        let (_hdr, _sense) = self.sg_io(&mut cdb, None, None)?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SenseData {
    pub sector_count: u16,
}

fn parse_sense(sense: &[u8]) -> Result<SenseData> {
    assert!(sense.len() >= 18);

    let code = sense[0];

    match code {
        0x72 | 0x73 => {
            let _sense_key = sense[1] & 0b1111;
            // todo check sense_key

            let _asc = sense[2]; // ADDITIONAL SENSE CODE
            let _ascq = sense[3]; // ADDITIONAL SENSE CODE QUALIFIER

            let sense_desc = &sense[8..];

            let _desc_code = sense_desc[0];
            // todo check descriptor code
            let sector_count = sense_desc[5] as u16 | (sense_desc[4] as u16) << 8;

            return Ok(SenseData { sector_count });
        }
        0x70 | 0x71 => {}
        _ => {
            unreachable!();
        }
    }
    anyhow::bail!("unexpected error");
}

fn parse_power_cond_desc(raw: &[u8]) -> PowerCondDescriptor {
    let flag = raw[1];
    let default_timer = u32::from_le_bytes(raw[4..=7].try_into().unwrap());
    let saved_timer = u32::from_le_bytes(raw[8..=11].try_into().unwrap());
    let current_timer = u32::from_le_bytes(raw[12..=15].try_into().unwrap());
    let recovery_time = u32::from_le_bytes(raw[16..=19].try_into().unwrap());
    let min_timer = u32::from_le_bytes(raw[20..=23].try_into().unwrap());
    let max_timer = u32::from_le_bytes(raw[24..=27].try_into().unwrap());

    PowerCondDescriptor {
        supported: flag & 0b1000_0000 != 0,
        savable: flag & 0b0100_0000 != 0,
        changeable: flag & 0b0010_0000 != 0,
        default_enable: flag & 0b0001_0000 != 0,
        saved_enable: flag & 0b0000_1000 != 0,
        current_enable: flag & 0b0000_0100 != 0,
        default_timer,
        saved_timer,
        current_timer,
        recovery_time,
        min_timer,
        max_timer,
    }
}
