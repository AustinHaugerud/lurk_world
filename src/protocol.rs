use std::io::BufRead;
use std::io::Write;

use lurk_macros::{LurkReadable, TypeCode};
use rlua::prelude::LuaTable;
use rlua::Table;

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct LurkName {
    pub bytes: [u8; 32],
}

impl From<[u8; 32]> for LurkName {
    fn from(bytes: [u8; 32]) -> Self {
        LurkName { bytes }
    }
}

pub trait TypeCode {
    fn type_code() -> u8;
}

pub trait LurkReadable {
    fn static_block_size() -> usize;
    fn has_var_block() -> bool;
}

#[derive(TypeCode, LurkReadable)]
#[Code = 1]
#[StaticBlockSize = 67]
#[VarBlock = true]
pub struct Message {
    pub recipient: LurkName,
    pub sender: LurkName,
    pub message: Vec<u8>,
}

#[derive(TypeCode, LurkReadable)]
#[Code = 2]
#[StaticBlockSize = 3]
#[VarBlock = false]
pub struct ChangeRoom {
    pub room_number: u16,
}

#[derive(TypeCode)]
#[Code = 3]
pub struct Fight;

#[derive(TypeCode, LurkReadable)]
#[Code = 4]
#[StaticBlockSize = 33]
#[VarBlock = false]
pub struct PVPFight {
    pub target: LurkName,
}

#[derive(TypeCode, LurkReadable)]
#[Code = 5]
#[StaticBlockSize = 33]
#[VarBlock = false]
pub struct Loot {
    pub target: LurkName,
}

#[derive(TypeCode)]
#[Code = 6]
pub struct Start;

#[derive(TypeCode)]
#[Code = 7]
pub struct Error {
    pub code: u8,
    pub message: Vec<u8>,
}

#[derive(TypeCode)]
#[Code = 8]
pub struct Accept {
    pub code: u8,
}

#[derive(TypeCode)]
#[Code = 9]
pub struct Room {
    pub number: u16,
    pub name: LurkName,
    pub description: Vec<u8>,
}

const FLAG: u8 = 0b1000_0000;
bitflags! {
    pub struct CharacterFlags: u8 {
        const ALIVE       = FLAG;
        const JOIN_BATTLE = FLAG >> 1;
        const MONSTER     = FLAG >> 2;
        const STARTED     = FLAG >> 3;
        const READY       = FLAG >> 4;
    }
}

impl CharacterFlags {
    pub fn to_u8(&self) -> u8 {
        self.bits
    }

    fn set_flag(&mut self, mask: CharacterFlags, status: bool) {
        if (status) {
            self.bits = self.bits| mask.to_u8();
        }
    }

    pub fn set_alive(&mut self, status: bool) {
        self.set_flag(CharacterFlags::ALIVE, status);
    }

    pub fn set_join_battle(&mut self, status: bool) {
        self.set_flag(CharacterFlags::JOIN_BATTLE, status);
    }

    pub fn set_monster(&mut self, status: bool) {
        self.set_flag(CharacterFlags::MONSTER, status);
    }

    pub fn set_started(&mut self, status: bool) {
        self.set_flag(CharacterFlags::STARTED, status);
    }

    pub fn set_ready(&mut self, status: bool) {
        self.set_flag(CharacterFlags::READY, status);
    }
}

impl From<u8> for CharacterFlags {
    fn from(bits: u8) -> Self {
        CharacterFlags { bits }
    }
}

#[derive(TypeCode, LurkReadable)]
#[Code = 10]
#[StaticBlockSize = 86]
#[VarBlock = true]
pub struct Character {
    pub name: LurkName,
    pub flags: CharacterFlags,
    pub attack: u16,
    pub defense: u16,
    pub regen: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room_number: u16,
    pub description: Vec<u8>,
}

#[derive(TypeCode)]
#[Code = 11]
pub struct Game {
    pub initial_points: u16,
    pub stat_limit: u16,
    pub description: Vec<u8>,
}

#[derive(TypeCode)]
#[Code = 12]
pub struct Leave;

#[derive(TypeCode)]
#[Code = 13]
pub struct Connection {
    pub room_number: u16,
    pub room_name: LurkName,
    pub description: Vec<u8>,
}

#[derive(TypeCode, LurkReadable)]
#[Code = 14]
#[StaticBlockSize = 4]
#[VarBlock = true]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub extensions: Vec<Vec<u8>>,
}
