// Support using the library without the standard library
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::{
    fmt,
    io::{Error, ErrorKind},
};

use core::str::FromStr;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command {
    pub sender: Sender,
    pub address: u8,
    pub data: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sender {
    Master,
    Slave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    /// Setpoint temperature controller
    SetpointTempControl = 0x00,

    /// Internal temperature
    InternalTemp = 0x01,

    /// Error report
    ErrorReport = 0x05,

    /// Warning message
    WarningMessage = 0x06,

    /// Setting process temperature
    SetProcessTemp = 0x09,

    /// Temperature control mode
    TempControlMode = 0x13,

    /// Temperature control
    TempControl = 0x14,

    /// Operating lock
    OperationLock = 0x17,

    /// Process temperature actual value setting mode
    ProcessTempActualSettingMode = 0x19,
}

impl Address {
    #[must_use]
    pub const fn from_u8(x: u8) -> Option<Self> {
        match x {
            0x00 => Some(Self::SetpointTempControl),
            0x01 => Some(Self::InternalTemp),
            0x05 => Some(Self::ErrorReport),
            0x06 => Some(Self::WarningMessage),
            0x09 => Some(Self::SetProcessTemp),
            0x13 => Some(Self::TempControlMode),
            0x14 => Some(Self::TempControl),
            0x17 => Some(Self::OperationLock),
            0x19 => Some(Self::ProcessTempActualSettingMode),
            _ => None,
        }
    }
}

impl From<Address> for u8 {
    fn from(from: Address) -> Self {
        from as u8
    }
}

impl From<Sender> for u8 {
    fn from(from: Sender) -> Self {
        match from {
            Sender::Master => b'M',
            Sender::Slave => b'S',
        }
    }
}

const EMPTY_DATA: [u8; 4] = [b'*'; 4];

const fn byte_cmd_msg(sender: u8, addr: [u8; 2], data: [u8; 4]) -> [u8; 10] {
    [
        b'{', sender, addr[0], addr[1], data[0], data[1], data[2], data[3], b'\r', b'\n',
    ]
}

impl Command {
    #[must_use]
    pub fn into_bytes(self) -> [u8; 10] {
        let addr: [u8; 2] = [
            to_upper_hex(self.address / 16),
            to_upper_hex(self.address % 16),
        ];
        let mut data = EMPTY_DATA;
        if let Some(d) = self.data {
            let hi = (d >> 8) as u8;
            let lo = (d & 0xFF) as u8;
            data[0] = to_upper_hex(hi / 16);
            data[1] = to_upper_hex(hi % 16);
            data[2] = to_upper_hex(lo / 16);
            data[3] = to_upper_hex(lo % 16);
        }
        byte_cmd_msg(self.sender.into(), addr, data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid message length
    MessageLength,
    /// Non-ASCII str
    NonAsciiStr,
    /// Invalid sender
    Sender,
    /// Invalid command data
    CommandData,
    /// Invalid command address
    Address,
}

#[cfg(feature = "std")]
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::MessageLength => write!(f, "Invalid message length"),
            Self::NonAsciiStr => write!(f, "Non-ASCII str"),
            Self::Sender => write!(f, "Invalid sender"),
            Self::CommandData => write!(f, "Invalid command data"),
            Self::Address => write!(f, "Invalid command address"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

#[cfg(feature = "std")]
impl From<ParseError> for Error {
    fn from(e: ParseError) -> Error {
        Error::new(ErrorKind::InvalidData, e)
    }
}

impl FromStr for Command {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        if s.len() != 10 {
            return Err(ParseError::MessageLength);
        }
        if !s.is_ascii() {
            return Err(ParseError::NonAsciiStr);
        }

        let (_start, tail) = s.split_at(1);
        let (sender, tail) = tail.split_at(1);
        let (addr, tail) = tail.split_at(2);
        let (data, _) = tail.split_at(4);

        let sender = match sender {
            "M" => Sender::Master,
            "S" => Sender::Slave,
            _ => {
                return Err(ParseError::Sender);
            }
        };

        let data = match data {
            "****" => None,
            _ => Some(u16::from_str_radix(data, 16).map_err(|_| ParseError::CommandData)?),
        };

        let address = u8::from_str_radix(addr, 16).map_err(|_| ParseError::Address)?;

        Ok(Command {
            sender,
            address,
            data,
        })
    }
}

// TODO: replace this conversation with other methods
// as soon as there is something available within core.
fn to_upper_hex(x: u8) -> u8 {
    match x {
        0..=9 => b'0' + x,
        10..=15 => b'A' + (x - 10),
        _ => unreachable!(),
    }
}
