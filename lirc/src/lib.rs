use iocuddle::*;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::Path;

const LIRC: Group = Group::new(b'i');

const LIRC_SET_SEND_CARRIER: Ioctl<iocuddle::Write, &u32> = unsafe { LIRC.write(0x13) };
const LIRC_SET_SEND_DUTY_CYCLE: Ioctl<iocuddle::Write, &u32> = unsafe { LIRC.write(0x15) };
const LIRC_SET_TRANSMITTER_MASK: Ioctl<iocuddle::Write, &u32> = unsafe { LIRC.write(0x17) };
const LIRC_GET_FEATURES: Ioctl<iocuddle::Read, &u32> = unsafe { LIRC.read(0x00) };

const LIRC_CAN_SET_SEND_CARRIER: u32 = 0x100;
const LIRC_CAN_SET_SEND_DUTY_CYCLE: u32 = 0x200;
const LIRC_CAN_SET_TRANSMITTER_MASK: u32 = 0x400;
const LIRC_CAN_SEND_PULSE: u32 = 2;

pub struct Lirc {
    file: File,
    features: u32,
}

pub fn lirc_open(path: &Path) -> io::Result<Lirc> {
    let file = OpenOptions::new().read(true).write(true).open(path)?;

    if let Ok((0, features)) = LIRC_GET_FEATURES.ioctl(&file) {
        Ok(Lirc { file, features })
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            String::from("not a lirc device"),
        ))
    }
}

impl Lirc {
    pub fn send(&mut self, data: &[u32]) -> io::Result<()> {
        if (self.features & LIRC_CAN_SEND_PULSE) != 0 {
            let bs_length = if (data.len() % 2) == 0 {
                (data.len() - 1) * 4
            } else {
                data.len() * 4
            };

            // there must be a nicer way to write an array of u32s..
            let data = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, bs_length) };
            let res = self.file.write(data)?;

            if res != bs_length {
                Err(Error::new(
                    ErrorKind::Other,
                    String::from("send incomplete"),
                ))
            } else {
                Ok(())
            }
        } else {
            Err(Error::new(
                ErrorKind::Other,
                String::from("device does not support sending"),
            ))
        }
    }

    /// Does this lirc device support sending
    pub fn can_send(&self) -> bool {
        (self.features & LIRC_CAN_SEND_PULSE) != 0
    }

    /// Does this lirc device support setting send carrier
    pub fn can_set_send_carrier(&self) -> bool {
        (self.features & LIRC_CAN_SET_SEND_CARRIER) != 0
    }

    /// Does this lirc device support setting send duty cycle
    pub fn can_set_send_duty_cycle(&self) -> bool {
        (self.features & LIRC_CAN_SET_SEND_DUTY_CYCLE) != 0
    }

    /// Does this lirc device support setting transmitter mask
    pub fn can_set_send_transmitter_mask(&self) -> bool {
        (self.features & LIRC_CAN_SET_TRANSMITTER_MASK) != 0
    }

    /// Set the send carrier. A carrier of 0 means unmodulated
    pub fn set_send_carrier(&mut self, carrier: u32) -> io::Result<()> {
        // The ioctl should return 0, but on old kernels it may return the new carrier setting; just ignore
        LIRC_SET_SEND_CARRIER.ioctl(&mut self.file, &carrier)?;

        Ok(())
    }

    /// Set the send carrier. A carrier of 0 means unmodulated
    pub fn set_send_duty_cycle(&mut self, duty_cycle: u32) -> io::Result<()> {
        debug_assert!(duty_cycle > 1 && duty_cycle < 100);

        LIRC_SET_SEND_DUTY_CYCLE.ioctl(&mut self.file, &duty_cycle)?;

        Ok(())
    }

    pub fn num_transmitters(&mut self) -> io::Result<u32> {
        // If the LIRC_SET_TRANSMITTER_MASK is called with an invalid mask, the number of transmitters are returned
        LIRC_SET_TRANSMITTER_MASK.ioctl(&mut self.file, &!0)
    }

    /// Set the send carrier. A carrier of 0 means unmodulated
    pub fn set_transmitter_mask(&mut self, transmitter_mask: u32) -> io::Result<()> {
        let res = LIRC_SET_TRANSMITTER_MASK.ioctl(&mut self.file, &transmitter_mask)?;

        if res != 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!("device only supports {} transmitters", res),
            ))
        } else {
            Ok(())
        }
    }
}