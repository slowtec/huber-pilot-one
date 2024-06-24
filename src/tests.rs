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
        b'{'
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
        b'M'
    );
    assert_eq!(
        Command {
            sender: Slave,
            address,
            data,
        }
        .into_bytes()[1],
        b'S'
    );
}

#[test]
fn test_to_upper_hex() {
    assert_eq!(to_upper_hex(0) as char, '0');
    assert_eq!(to_upper_hex(9) as char, '9');
    assert_eq!(to_upper_hex(10) as char, 'A');
    assert_eq!(to_upper_hex(15) as char, 'F');
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
    assert_eq!(cmd[2], b'3');
    assert_eq!(cmd[3], b'1');

    let cmd = Command {
        sender,
        address: 0x7,
        data,
    }
    .into_bytes();
    assert_eq!(cmd[2], b'0');
    assert_eq!(cmd[3], b'7');

    let cmd = Command {
        sender,
        address: 255,
        data,
    }
    .into_bytes();
    assert_eq!(cmd[2], b'F');
    assert_eq!(cmd[3], b'F');
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
        data: Some(core::u16::MAX),
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
    let expected = &[
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
        assert_eq!(*addr as u8, *nr);
    }
    let byte: u8 = Address::OperationLock.into();
    assert_eq!(byte, 0x17);
}

#[test]
fn decode_address_enum() {
    use self::Address::*;
    let expected = &[
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
        assert_eq!(Address::from_u8(*nr).unwrap(), *addr);
    }
    assert!(Address::from_u8(255).is_none());
}
