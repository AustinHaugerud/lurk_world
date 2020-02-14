use std::io::{BufWriter, Write};
use crate::protocol::{Message, Error, Room, Character, Game, Connection, Accept, TypeCode, LurkName, Version};

use std::io;
use std::net::TcpStream;
use byteorder::{WriteBytesExt, LittleEndian};

pub type LurkWriteResult = io::Result<()>;

pub enum LurkWriteMessage {
    Message(Message),
    Error(Error),
    Accept(Accept),
    Room(Room),
    Character(Character),
    Game(Game),
    Connection(Connection),
    Version(Version),
}

pub trait LurkWrite {
    fn write_lurk_name(&mut self, name: &LurkName) -> LurkWriteResult;
    fn write_message(&mut self, msg: &Message) -> LurkWriteResult;
    fn write_error(&mut self, err: &Error) -> LurkWriteResult;
    fn write_accept(&mut self, accept: &Accept) -> LurkWriteResult;
    fn write_room(&mut self, room: &Room) -> LurkWriteResult;
    fn write_character(&mut self, ch: &Character) -> LurkWriteResult;
    fn write_game(&mut self, game: &Game) -> LurkWriteResult;
    fn write_connection(&mut self, conn: &Connection) -> LurkWriteResult;
    fn write_version(&mut self, version: &Version) -> LurkWriteResult;
}

impl LurkWrite for BufWriter<TcpStream> {

    fn write_lurk_name(&mut self, name: &LurkName) -> LurkWriteResult {
        self.write_all(&name.bytes)?;
        Ok(())
    }

    fn write_message(&mut self, msg: &Message) -> LurkWriteResult {
        self.write_u8(Message::type_code())?;
        self.write_u16::<LittleEndian>(msg.message.len() as u16)?;
        self.write_lurk_name(&msg.recipient)?;
        self.write_lurk_name(&msg.sender)?;
        self.write_all(&msg.message)?;
        Ok(())
    }

    fn write_error(&mut self, err: &Error) -> LurkWriteResult {
        self.write_u8(Error::type_code())?;
        self.write_u8(err.code)?;
        self.write_u16::<LittleEndian>(err.message.len() as u16)?;
        self.write_all(&err.message)?;
        Ok(())
    }

    fn write_accept(&mut self, accept: &Accept) -> LurkWriteResult {
        self.write_u8(Accept::type_code())?;
        self.write_u8(accept.code)?;
        Ok(())
    }

    fn write_room(&mut self, room: &Room) -> LurkWriteResult {
        self.write_u8(Room::type_code())?;
        self.write_u16::<LittleEndian>(room.number)?;
        self.write_lurk_name(&room.name)?;
        self.write_u16::<LittleEndian>(room.description.len() as u16)?;
        self.write_all(&room.description)?;
        Ok(())
    }

    fn write_character(&mut self, ch: &Character) -> LurkWriteResult {
        self.write_u8(Character::type_code())?;
        self.write_lurk_name(&ch.name)?;
        self.write_u8(ch.flags.to_u8())?;
        self.write_u16::<LittleEndian>(ch.attack)?;
        self.write_u16::<LittleEndian>(ch.defense)?;
        self.write_u16::<LittleEndian>(ch.regen)?;
        self.write_i16::<LittleEndian>(ch.health)?;
        self.write_u16::<LittleEndian>(ch.gold)?;
        self.write_u16::<LittleEndian>(ch.current_room_number)?;
        self.write_u16::<LittleEndian>(ch.description.len() as u16)?;
        self.write_all(&ch.description)?;
        Ok(())
    }

    fn write_game(&mut self, game: &Game) -> LurkWriteResult {
        self.write_u8(Game::type_code())?;
        self.write_u16::<LittleEndian>(game.initial_points)?;
        self.write_u16::<LittleEndian>(game.stat_limit)?;
        self.write_u16::<LittleEndian>(game.description.len() as u16)?;
        self.write_all(&game.description)?;
        Ok(())
    }

    fn write_connection(&mut self, conn: &Connection) -> LurkWriteResult {
        self.write_u8(Connection::type_code())?;
        self.write_u16::<LittleEndian>(conn.room_number)?;
        self.write_lurk_name(&conn.room_name)?;
        self.write_u16::<LittleEndian>(conn.description.len() as u16)?;
        self.write_all(&conn.description)?;
        Ok(())
    }

    fn write_version(&mut self, version: &Version) -> LurkWriteResult {
        self.write_u8(Version::type_code())?;
        self.write_u8(version.major)?;
        self.write_u8(version.minor)?;
        self.write_u16::<LittleEndian>(version.extensions.len() as u16)?;
        for extension in version.extensions.iter() {
            self.write_all(extension)?;
        }
        Ok(())
    }
}
