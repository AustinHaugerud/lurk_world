use crate::protocol::{ChangeRoom, Character, Fight, Leave, Loot, LurkReadable, Message, PVPFight, Start, TypeCode, Version};

#[derive(PartialEq)]
pub enum LurkPollState {
    Partial,
    Complete,
    NoMatch,
}

pub enum LurkPollTypeState {
    Match,
    NoMatch,
}

impl LurkPollTypeState {
    fn is_match(&self) -> bool {
        match self {
            LurkPollTypeState::Match => true,
            LurkPollTypeState::NoMatch => false,
        }
    }
}

impl LurkPollState {
    fn good(&self) -> bool {
        match self {
            LurkPollState::Partial => true,
            LurkPollState::Complete => true,
            LurkPollState::NoMatch => false,
        }
    }
}

type LurkReadResult<T> = Result<T, ()>;

pub enum LurkPollEvent {
    Message(Message),
    ChangeRoom(ChangeRoom),
    Fight,
    PVPFight(PVPFight),
    Loot(Loot),
    Start,
    Character(Character),
    Leave,
    Version(Version),
    Pending,
    Bad,
}

pub enum LurkReadEvent {
    Message(Message),
    ChangeRoom(ChangeRoom),
    Fight,
    PVPFight(PVPFight),
    Loot(Loot),
    Start,
    Character(Character),
    Leave,
    Version(Version),
}

pub trait LurkRead {
    fn poll_message(&self) -> LurkPollState;
    fn read_message(&mut self) -> LurkReadResult<Message>;

    fn poll_fight(&self) -> LurkPollTypeState;

    fn poll_changeroom(&self) -> LurkPollState;
    fn read_changeroom(&mut self) -> LurkReadResult<ChangeRoom>;

    fn poll_pvpfight(&self) -> LurkPollState;
    fn read_pvpfight(&mut self) -> LurkReadResult<PVPFight>;

    fn poll_start(&self) -> LurkPollTypeState;

    fn poll_loot(&self) -> LurkPollState;
    fn read_loot(&mut self) -> LurkReadResult<Loot>;

    fn poll_character(&self) -> LurkPollState;
    fn read_character(&mut self) -> LurkReadResult<Character>;

    fn poll_leave(&self) -> LurkPollTypeState;

    fn poll_version(&self) -> LurkPollState;
    fn read_version(&mut self) -> LurkReadResult<Version>;

    fn poll_lurk(&mut self) -> LurkReadResult<LurkPollEvent>;
}

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{BufReader, Read, BufRead};
use std::net::TcpStream;
use crate::read_buffer::ReadBuffer;
use std::error::Error;

fn start_match<T: TypeCode>(buffer: &[u8]) -> bool {
    if buffer.len() > 0 {
        return buffer[0] == T::type_code();
    }
    panic!("Don't poll on an empty buffer.");
}

fn poll<T: TypeCode + LurkReadable>(buffer: &[u8]) -> LurkPollState {
    if start_match::<T>(buffer) {
        if buffer.len() >= T::static_block_size() {
            if T::has_var_block() {
                let len_desc_buf_start = T::static_block_size() - 2;
                let b1 = buffer[len_desc_buf_start];
                let b2 = buffer[len_desc_buf_start + 1];
                let buf = [b1, b2];
                let len = u16::from_le_bytes(buf);
                if buffer.len() >= T::static_block_size() + len as usize {
                    return LurkPollState::Complete;
                } else {
                    return LurkPollState::Partial;
                }
            } else {
                return LurkPollState::Complete;
            }
        } else {
            LurkPollState::Partial
        }
    } else {
        LurkPollState::NoMatch
    }
}

impl LurkRead for ReadBuffer {
    fn poll_message(&self) -> LurkPollState {
        let buffer = self.buffer();

        let min_size = 2 + 32 + 32;

        if start_match::<Message>(buffer) {
            if buffer.len() >= min_size {
                let len_desc_buf = [buffer[1], buffer[2]];
                let msg_len: u16 = u16::from_le_bytes(len_desc_buf);
                let total_expected_size = min_size + msg_len as usize;
                if buffer.len() >= total_expected_size {
                    return LurkPollState::Complete;
                } else {
                    return LurkPollState::Partial;
                }
            }
            LurkPollState::Partial
        } else {
            LurkPollState::NoMatch
        }
    }

    fn read_message(&mut self) -> Result<Message, ()> {
        let _type = self.read_u8().map_err(|_| {})?;
        let msg_len = self.read_u16::<LittleEndian>().map_err(|_| {})?;
        let mut recipient_buffer = [0u8; 32];
        self.read(&mut recipient_buffer).map_err(|_| {})?;
        let mut sender_buffer = [0u8; 32];
        self.read(&mut sender_buffer).map_err(|_| {})?;
        let mut message_buffer: Vec<u8> = vec![0u8; msg_len as usize];
        self.read(&mut message_buffer).map_err(|_| {})?;
        Ok(Message {
            recipient: recipient_buffer.into(),
            sender: sender_buffer.into(),
            message: message_buffer,
        })
    }

    fn poll_fight(&self) -> LurkPollTypeState {
        if start_match::<Fight>(self.buffer()) {
            LurkPollTypeState::Match
        } else {
            LurkPollTypeState::NoMatch
        }
    }

    fn poll_changeroom(&self) -> LurkPollState {
        poll::<ChangeRoom>(self.buffer())
    }

    fn read_changeroom(&mut self) -> LurkReadResult<ChangeRoom> {
        let _type = self.read_u8().map_err(|_| {})?;
        let room_number = self.read_u16::<LittleEndian>().map_err(|_| {})?;
        Ok(ChangeRoom { room_number })
    }

    fn poll_pvpfight(&self) -> LurkPollState {
        poll::<PVPFight>(self.buffer())
    }

    fn read_pvpfight(&mut self) -> LurkReadResult<PVPFight> {
        let _type = self.read_u8().map_err(|_| {})?;
        let mut target_buffer = [0u8; 32];
        self.read(&mut target_buffer).map_err(|_| {})?;
        Ok(PVPFight {
            target: target_buffer.into(),
        })
    }

    fn poll_start(&self) -> LurkPollTypeState {
        if start_match::<Start>(self.buffer()) {
            LurkPollTypeState::Match
        } else {
            LurkPollTypeState::NoMatch
        }
    }

    fn poll_loot(&self) -> LurkPollState {
        poll::<Loot>(self.buffer())
    }

    fn read_loot(&mut self) -> LurkReadResult<Loot> {
        let _type = self.read_u8().map_err(|_| {})?;
        let mut target_buffer = [0u8; 32];
        self.read(&mut target_buffer).map_err(|_| {})?;
        Ok(Loot {
            target: target_buffer.into(),
        })
    }

    fn poll_character(&self) -> LurkPollState {
        poll::<Character>(self.buffer())
    }

    fn read_character(&mut self) -> LurkReadResult<Character> {
        let _type = self.read_u8().map_err(|_| {})?;

        let mut name_buffer = [0u8; 32];
        self.read(&mut name_buffer).map_err(|_| {})?;

        let flags = self.read_u8().map_err(|_| {})?;

        let attack = self.read_u16::<LittleEndian>().map_err(|_| {})?;
        let defense = self.read_u16::<LittleEndian>().map_err(|_| {})?;
        let regen = self.read_u16::<LittleEndian>().map_err(|_| {})?;
        let health = self.read_i16::<LittleEndian>().map_err(|_| {})?;
        let gold = self.read_u16::<LittleEndian>().map_err(|_| {})?;
        let current_room_number = self.read_u16::<LittleEndian>().map_err(|_| {})?;

        let desc_len = self.read_u16::<LittleEndian>().map_err(|_| {})?;

        let mut description_buffer: Vec<u8> = vec![0u8; desc_len as usize];
        self.read(&mut description_buffer).map_err(|_| {})?;

        Ok(Character {
            name: name_buffer.into(),
            flags: flags.into(),
            attack,
            defense,
            regen,
            health,
            gold,
            current_room_number,
            description: description_buffer,
        })
    }

    fn poll_leave(&self) -> LurkPollTypeState {
        if start_match::<Leave>(self.buffer()) {
            LurkPollTypeState::Match
        } else {
            LurkPollTypeState::NoMatch
        }
    }

    fn poll_version(&self) -> LurkPollState {
        let buffer = self.buffer();
        if start_match::<Version>(buffer) {
            if buffer.len() < Version::static_block_size() {
                return LurkPollState::Partial;
            }
            let num_ext_buf = [buffer[3], buffer[4]];
            let num_extensions = u16::from_le_bytes(num_ext_buf);
            let mut cursor = 5;
            let mut num_complete = 0u16;
            for _ in 0..num_extensions {
                let ext_len_buf = [buffer[cursor], buffer[cursor + 1]];
                let num_bytes: u16 = u16::from_le_bytes(ext_len_buf);
                cursor += num_bytes as usize;
                if cursor > buffer.len() {
                    break;
                }
                num_complete += 1;
            }
            if num_complete == num_extensions {
                return LurkPollState::Complete;
            } else {
                return LurkPollState::Partial;
            }
        }
        LurkPollState::NoMatch
    }

    fn read_version(&mut self) -> LurkReadResult<Version> {
        let _type = self.read_u8().map_err(|_| {})?;

        let major = self.read_u8().map_err(|_| {})?;
        let minor = self.read_u8().map_err(|_| {})?;

        let num_extensions = self.read_u16::<LittleEndian>().map_err(|_| {})?;

        let mut extensions: Vec<Vec<u8>> = vec![];

        for _ in 0..num_extensions {
            let ext_len = self.read_u16::<LittleEndian>().map_err(|_| {})?;
            let mut ext: Vec<u8> = vec![0u8; ext_len as usize];
            self.read(&mut ext).map_err(|_| {})?;
            extensions.push(ext);
        }

        Ok(Version {
            major,
            minor,
            extensions
        })
    }

    fn poll_lurk(&mut self) -> LurkReadResult<LurkPollEvent> {
        use std::io;
        // Attempt to fill the buffer if it's empty. If we don't get anything back,
        // (empty fill, would block error) then return pending.
        if self.buffer().is_empty() {
            match self.fill_buf() {
                Ok(fill) => if fill == 0 {
                    return Ok(LurkPollEvent::Pending);
                },
                Err(e) => if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(LurkPollEvent::Pending);
                } else {
                    eprintln!("{}", e.description());
                    return Err(());
                }
            }
        }

        match self.poll_message() {
            LurkPollState::Complete => {
                let msg = self.read_message()?;
                return Ok(LurkPollEvent::Message(msg));
            }
            LurkPollState::Partial => return Ok(LurkPollEvent::Pending),
            _ => {}
        }

        if self.poll_fight().is_match() {
            self.read_u8().map_err(|_| {})?;
            return Ok(LurkPollEvent::Fight);
        }

        match self.poll_changeroom() {
            LurkPollState::Complete => {
                let chgrm = self.read_changeroom()?;
                return Ok(LurkPollEvent::ChangeRoom(chgrm));
            }
            LurkPollState::Partial => return Ok(LurkPollEvent::Pending),
            _ => {}
        }

        match self.poll_pvpfight() {
            LurkPollState::Complete => {
                let pvpfight = self.read_pvpfight()?;
                return Ok(LurkPollEvent::PVPFight(pvpfight));
            }
            LurkPollState::Partial => return Ok(LurkPollEvent::Pending),
            _ => {}
        }

        match self.poll_loot() {
            LurkPollState::Complete => {
                let loot = self.read_loot()?;
                return Ok(LurkPollEvent::Loot(loot));
            }
            LurkPollState::Partial => return Ok(LurkPollEvent::Pending),
            _ => {}
        }

        if self.poll_start().is_match() {
            self.read_u8().map_err(|_| {})?;
            return Ok(LurkPollEvent::Start);
        }

        match self.poll_character() {
            LurkPollState::Complete => {
                let character = self.read_character()?;
                return Ok(LurkPollEvent::Character(character));
            }
            LurkPollState::Partial => return Ok(LurkPollEvent::Pending),
            _ => {}
        }

        if self.poll_leave().is_match() {
            self.read_u8().map_err(|_| {})?;
            return Ok(LurkPollEvent::Leave);
        }

        match self.poll_version() {
            LurkPollState::Complete => {
                let version = self.read_version()?;
                return Ok(LurkPollEvent::Version(version));
            },
            LurkPollState::Partial => return Ok(LurkPollEvent::Pending),
            _ => {},
        }

        Ok(LurkPollEvent::Bad)
    }
}
