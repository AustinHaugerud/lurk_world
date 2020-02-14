use std::sync::{Mutex, Arc};
use std::collections::VecDeque;
use crate::client::{ClientEvent, ClientEventKind};
use rlua::{UserData, UserDataMethods, MetaMethod};
use crate::read::LurkReadEvent;
use crate::write::LurkWriteMessage;
use crate::protocol::{LurkName, Message, Error, Accept, Room, Character, Game, Connection, Version};
use crate::protocol::CharacterFlags;
use rlua::prelude::LuaTable;

///////////////////////////////////////////////////////////////////////////////

impl UserData for LurkName {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Eq, |_, (lhs, rhs): (LurkName, LurkName)| {
            Ok(lhs.eq(&rhs))
        });

        methods.add_method("string", |ctx, this, ()| {
            let string = String::from_utf8_lossy(&this.bytes);
            Ok(string.to_string())
        });
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ClientEventBuffer {
    events: Arc<Mutex<VecDeque<ClientEvent>>>,
}

impl Default for ClientEventBuffer {
    fn default() -> Self {
        ClientEventBuffer {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl ClientEventBuffer {
    pub fn add(&mut self, event: ClientEvent) {
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
    }

    pub fn clear(&mut self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }

    pub fn pop(&mut self) -> Option<ClientEvent> {
        let mut events = self.events.lock().unwrap();
        events.pop_front()
    }
}

impl UserData for ClientEventBuffer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        use rlua::String;
        use std::string::String as StdString;
        methods.add_method_mut("poll", |ctx, buffer, ()| {
            let table = ctx.create_table()?;
            if let Some(event) = buffer.pop() {
                table.set("id", event.client_id()).unwrap();
                table.set("isSome", true).unwrap();
                match event.event() {
                    ClientEventKind::Read(read_event) => {
                        match read_event {
                            LurkReadEvent::Message(msg) => {
                                table.set("type", "message")?;
                                table.set("recipient", msg.recipient)?;
                                table.set("sender", msg.sender)?;
                                let msg_string = StdString::from_utf8_lossy(&msg.message);
                                table.set("message", msg_string.to_string())?;
                            }
                            LurkReadEvent::ChangeRoom(chgrm) => {
                                table.set("type", "change_room")?;
                                table.set("room_number", chgrm.room_number)?;
                            }
                            LurkReadEvent::Fight => {
                                table.set("type", "fight")?;
                            }
                            LurkReadEvent::PVPFight(pvpfight) => {
                                table.set("type", "pvp_fight")?;
                                table.set("target", pvpfight.target)?;
                            }
                            LurkReadEvent::Loot(loot) => {
                                table.set("type", "loot")?;
                                table.set("type", loot.target)?;
                            }
                            LurkReadEvent::Start => {
                                table.set("type", "start")?;
                            }
                            LurkReadEvent::Character(ch) => {
                                table.set("type", "character")?;
                                table.set("name", ch.name)?;

                                // Flags
                                table.set("alive", ch.flags.intersects(CharacterFlags::ALIVE))?;
                                table.set("join_battle", ch.flags.intersects(CharacterFlags::JOIN_BATTLE))?;
                                table.set("monster", ch.flags.intersects(CharacterFlags::MONSTER))?;
                                table.set("started", ch.flags.intersects(CharacterFlags::STARTED))?;
                                table.set("ready", ch.flags.intersects(CharacterFlags::READY))?;

                                table.set("attack", ch.attack)?;
                                table.set("defense", ch.defense)?;
                                table.set("regen", ch.regen)?;
                                table.set("health", ch.health)?;
                                table.set("gold", ch.gold)?;
                                table.set("room_number", ch.current_room_number)?;
                                table.set("description", ch.description.clone())?;
                            }
                            LurkReadEvent::Leave => {
                                table.set("type", "leave")?;
                            }
                            LurkReadEvent::Version(vers) => {
                                table.set("type", "version")?;
                                table.set("major", vers.major)?;
                                table.set("minor", vers.minor)?;
                            }
                        }
                    }
                    ClientEventKind::Join => {
                        table.set("type", "join")?;
                    }
                    ClientEventKind::Left => {
                        table.set("type", "left")?;
                    }
                }
            } else {
                table.set("isSome", false).unwrap();
            }
            Ok(table)
        });
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct ClientWriteBuffer {
    messages: Arc<Mutex<VecDeque<LurkWriteMessage>>>,
}

impl Default for ClientWriteBuffer {
    fn default() -> Self {
        ClientWriteBuffer {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl ClientWriteBuffer {
    fn add(&mut self, lurkmsg: LurkWriteMessage) {
        if let Ok(mut lock) = self.messages.lock() {
            lock.push_back(lurkmsg);
        }
        else {
            eprintln!("Cannot queue message, write buffer is poisoned.");
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.add(LurkWriteMessage::Message(message));
    }

    pub fn add_error(&mut self, error: Error) {
        self.add(LurkWriteMessage::Error(error));
    }

    pub fn add_accept(&mut self, accept: Accept) {
        self.add(LurkWriteMessage::Accept(accept));
    }

    pub fn add_room(&mut self, room: Room) {
        self.add(LurkWriteMessage::Room(room));
    }

    pub fn add_character(&mut self, character: Character) {
        self.add(LurkWriteMessage::Character(character));
    }

    pub fn add_game(&mut self, game: Game) {
        self.add(LurkWriteMessage::Game(game));
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.add(LurkWriteMessage::Connection(connection));
    }

    pub fn add_version(&mut self, version: Version) {
        self.add(LurkWriteMessage::Version(version));
    }
}

fn lossy_string_to_lurk_name(string: &str) -> LurkName {
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&string.as_bytes()[0..32.min(string.len())]);
    LurkName::from(buf)
}

impl UserData for ClientWriteBuffer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("send_message", |_, buffer, table: LuaTable| {
            let message: String = table.get("message")?;
            let recipient = {
                let data: String = table.get("recipient")?;
                lossy_string_to_lurk_name(&data)
            };
            let sender = {
                let data: String = table.get("sender")?;
                lossy_string_to_lurk_name(&data)
            };

            let message = Message {
                recipient,
                sender,
                message: message.into_bytes(),
            };
            buffer.add_message(message);
            Ok(())
        });

        methods.add_method_mut("send_message", |_, buffer, table: LuaTable| {
            let code: u8 = table.get("code")?;
            let message: String = table.get("message")?;
            let error = Error {
                code,
                message: message.into_bytes(),
            };
            buffer.add_error(error);
            Ok(())
        });

        methods.add_method_mut("send_accept", |_, buffer, table: LuaTable| {
            let code: u8 = table.get("action_type")?;
            let accept = Accept {
                code
            };
            buffer.add_accept(accept);
            Ok(())
        });

        methods.add_method_mut("send_room", |_, buffer, table: LuaTable| {
            let room_number: u16 = table.get("number")?;
            let room_name = {
                let data: String = table.get("name")?;
                lossy_string_to_lurk_name(&data)
            };
            let room_description: String = table.get("description")?;
            let room = Room {
                number: room_number,
                name: room_name,
                description: room_description.into_bytes(),
            };
            buffer.add_room(room);
            Ok(())
        });

        methods.add_method_mut("send_character", |_, buffer, table: LuaTable| {
            let name = {
                let data: String = table.get("name")?;
                lossy_string_to_lurk_name(&data)
            };
            let flags = {
                let mut flags = CharacterFlags::from(0u8);
                if let Ok(is_alive) = table.get::<&str, bool>("alive")  {
                    flags.set_alive(is_alive);
                }
                if let Ok(join_battle) = table.get::<&str, bool>("join_battle") {
                    flags.set_join_battle(join_battle);
                }
                if let Ok(is_monster) = table.get::<&str, bool>("monster") {
                    flags.set_monster(is_monster);
                }
                if let Ok(is_started) = table.get::<&str, bool>("started") {
                    flags.set_started(is_started);
                }
                if let Ok(is_ready) = table.get::<&str, bool>("ready") {
                    flags.set_ready(is_ready);
                }
                flags
            };
            let attack: u16 = table.get("attack")?;
            let defense: u16 = table.get("defense")?;
            let regen: u16 = table.get("regen")?;
            let health: i16 = table.get("health")?;
            let gold: u16 = table.get("gold")?;
            let current_room_number: u16 = table.get("room_number")?;
            let description: String = table.get("description")?;
            let character = Character {
                name,
                flags,
                attack,
                defense,
                regen,
                health,
                gold,
                current_room_number,
                description: description.into_bytes(),
            };
            buffer.add_character(character);
            Ok(())
        });
        
        methods.add_method_mut("send_game", |_, buffer, table: LuaTable| {
            let initial_points: u16 = table.get("initial_points")?;
            let stat_limit: u16 = table.get("stat_limit")?;
            let description: String = table.get("description")?;
            let game = Game {
                initial_points,
                stat_limit,
                description: description.into_bytes(),
            };
            buffer.add_game(game);
           Ok(()) 
        });
        
        methods.add_method_mut("send_connection", |_, buffer, table: LuaTable| {
            let room_number: u16 = table.get("number")?;
            let room_name = {
                let data: String = table.get("name")?;
                lossy_string_to_lurk_name(&data)
            };
            let description: String = table.get("description")?;
            let connection = Connection {
                room_number,
                room_name,
                description: description.into_bytes(),
            };
            buffer.add_connection(connection);
            Ok(())
        });

        methods.add_method_mut("send_version", |_, buffer, table: LuaTable| {
            let major: u8 = table.get("major")?;
            let minor: u8 = table.get("minor")?;
            let extensions: LuaTable = table.get("extensions")?;
            let sequence = extensions.sequence_values::<String>();
            
            let mut loaded: Vec<Vec<u8>> = vec![];
            
            for entry in sequence {
                let ext = entry?;
                loaded.push(ext.into_bytes());
            }
            
            let version = Version {
                major,
                minor,
                extensions: loaded,
            };
            buffer.add_version(version);
           Ok(())
        });
    }
}
