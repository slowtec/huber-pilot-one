use std::{
    fmt,
    io::{Error, ErrorKind},
    str::FromStr,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn from_u8(x: u8) -> Option<Self> {
        use self::Address::*;
        match x {
            0x00 => Some(SetpointTempControl),
            0x01 => Some(InternalTemp),
            0x05 => Some(ErrorReport),
            0x06 => Some(WarningMessage),
            0x09 => Some(SetProcessTemp),
            0x13 => Some(TempControlMode),
            0x14 => Some(TempControl),
            0x17 => Some(OperationLock),
            0x19 => Some(ProcessTempActualSettingMode),
            _ => None,
        }
    }
}

impl Into<u8> for Address {
    fn into(self: Address) -> u8 {
        self as u8
    }
}

impl Into<u8> for Sender {
    fn into(self: Sender) -> u8 {
        use self::Sender::*;
        match self {
            Master => b'M',
            Slave => b'S',
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
    pub fn into_bytes(self) -> [u8; 10] {
        let mut addr = format!("{:X}", self.address);
        if addr.len() < 2 {
            addr = format!("0{}", addr);
        }
        let addr = addr.as_bytes();
        let addr: [u8; 2] = [addr[0], addr[1]];
        let mut data = EMPTY_DATA;
        if let Some(d) = self.data {
            let mut hex = format!("{:X}", d);
            match hex.len() {
                1 => {
                    hex = format!("000{}", hex);
                }
                2 => {
                    hex = format!("00{}", hex);
                }
                3 => {
                    hex = format!("0{}", hex);
                }
                4 => { /* nothing to do */ }
                _ => {
                    unreachable!();
                }
            }
            let hex = hex.as_bytes();
            for (idx, h) in hex.iter().enumerate() {
                data[idx] = *h;
            }
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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match *self {
            MessageLength => write!(f, "Invalid message length"),
            NonAsciiStr => write!(f, "Non-ASCII str"),
            Sender => write!(f, "Invalid sender"),
            CommandData => write!(f, "Invalid command data"),
            Address => write!(f, "Invalid command address"),
        }
    }
}

impl std::error::Error for ParseError {}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sender::*;

    #[test]
    fn from_str_with_invalid_len() {
        assert!(Command::from_str("{M31*****\r\n").is_err());
        assert!(Command::from_str("{M31***\r\n").is_err());
    }

    #[test]
    fn from_bad_str() {
        let msg = "{�ۿ�3";
        assert!(Command::from_str(msg).is_err());
    }

    #[test]
    fn from_str_with_bad_sender() {
        assert!(Command::from_str("{X31****\r\n").is_err());
    }

    #[test]
    fn from_str_with_bad_data() {
        assert!(Command::from_str("{M310z00\r\n").is_err());
    }

    #[test]
    fn from_str_with_bad_address() {
        assert!(Command::from_str("{MTT0000\r\n").is_err());
    }

    #[test]
    fn from_str() {
        let cmd = Command::from_str("{M31****\r\n").unwrap();
        assert_eq!(cmd.sender, Sender::Master);
        assert_eq!(cmd.address, 0x31);
        assert_eq!(cmd.data, None);

        let cmd = Command::from_str("{S223219\r\n").unwrap();
        assert_eq!(cmd.sender, Sender::Slave);
        assert_eq!(cmd.address, 0x22);
        assert_eq!(cmd.data, Some(0x3219));
    }

    #[test]
    fn encode_start_char() {
        assert_eq!(
            Command {
                sender: Master,
                address: 0x31,
                data: None,
            }
            .into_bytes()[0],
            '{' as u8
        );
    }

    #[test]
    fn sender_into_u8() {
        let x: u8 = Master.into();
        assert_eq!(x, b'M');
        let x: u8 = Slave.into();
        assert_eq!(x, b'S');
    }

    #[test]
    fn encode_sender() {
        let address = 0x31;
        let data = None;
        assert_eq!(
            Command {
                sender: Master,
                address,
                data,
            }
            .into_bytes()[1],
            'M' as u8
        );
        assert_eq!(
            Command {
                sender: Slave,
                address,
                data,
            }
            .into_bytes()[1],
            'S' as u8
        );
    }

    #[test]
    fn encode_address() {
        let data = None;
        let sender = Slave;
        let cmd = Command {
            sender,
            address: 0x31,
            data,
        }
        .into_bytes();
        assert_eq!(cmd[2], '3' as u8);
        assert_eq!(cmd[3], '1' as u8);

        let cmd = Command {
            sender,
            address: 0x7,
            data,
        }
        .into_bytes();
        assert_eq!(cmd[2], '0' as u8);
        assert_eq!(cmd[3], '7' as u8);
    }

    #[test]
    fn encode_data() {
        let address = 0x19;
        let sender = Slave;

        let cmd = Command {
            sender,
            address,
            data: Some(1),
        }
        .into_bytes();
        assert_eq!(&cmd[4..8], b"0001");

        let cmd = Command {
            sender,
            address,
            data: Some(0xab),
        }
        .into_bytes();
        assert_eq!(&cmd[4..8], b"00AB");

        let cmd = Command {
            sender,
            address,
            data: Some(::std::u16::MAX),
        }
        .into_bytes();
        assert_eq!(&cmd[4..8], b"FFFF");

        let cmd = Command {
            sender,
            address,
            data: None,
        }
        .into_bytes();
        assert_eq!(&cmd[4..8], b"****");
    }

    #[test]
    fn encode_delimiters() {
        let cmd = Command {
            sender: Master,
            address: 0x22,
            data: None,
        }
        .into_bytes();
        assert_eq!(&cmd[8..10], b"\r\n");
    }

    #[test]
    fn encode_complete_command() {
        let cmd = Command {
            sender: Sender::Master,
            address: 0x09,
            data: Some(0x05E8),
        }
        .into_bytes();
        assert_eq!(cmd, *b"{M0905E8\r\n");

        let cmd = Command {
            sender: Sender::Slave,
            address: 0x19,
            data: Some(0x0001),
        }
        .into_bytes();
        assert_eq!(cmd, *b"{S190001\r\n");
    }

    #[test]
    fn encode_address_enum() {
        use self::Address::*;
        let expected = vec![
            (SetpointTempControl, 0x00),
            (InternalTemp, 0x01),
            (ErrorReport, 0x05),
            (WarningMessage, 0x06),
            (SetProcessTemp, 0x09),
            (TempControlMode, 0x13),
            (TempControl, 0x14),
            (OperationLock, 0x17),
            (ProcessTempActualSettingMode, 0x19),
        ];
        for (addr, nr) in expected {
            assert_eq!(addr as u8, nr);
        }
        let byte: u8 = Address::OperationLock.into();
        assert_eq!(byte, 0x17);
    }

    #[test]
    fn decode_address_enum() {
        use self::Address::*;
        let expected = vec![
            (SetpointTempControl, 0x00),
            (InternalTemp, 0x01),
            (ErrorReport, 0x05),
            (WarningMessage, 0x06),
            (SetProcessTemp, 0x09),
            (TempControlMode, 0x13),
            (TempControl, 0x14),
            (OperationLock, 0x17),
            (ProcessTempActualSettingMode, 0x19),
        ];
        for (addr, nr) in expected {
            assert_eq!(Address::from_u8(nr).unwrap(), addr);
        }
        assert!(Address::from_u8(255).is_none());
    }
}
