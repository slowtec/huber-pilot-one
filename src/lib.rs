use std::str::FromStr;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub sender: Sender,
    pub address: u8,
    pub data: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sender {
    Master,
    Slave,
}

impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 10 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }
        if !s.is_ascii() {
            return Err(Error::new(ErrorKind::InvalidData, "None-ASCII str"));
        }

        let (_start, tail) = s.split_at(1);
        let (sender, tail) = tail.split_at(1);
        let (addr, tail) = tail.split_at(2);
        let (data, _) = tail.split_at(4);

        let sender = match sender {
            "M" => Sender::Master,
            "S" => Sender::Slave,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid sender '{}'", sender),
                ));
            }
        };

        let data = match data {
            "****" => None,
            _ => Some(u16::from_str_radix(data, 16)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid command data"))?),
        };

        let address = u8::from_str_radix(addr, 16)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid address"))?;

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
}
