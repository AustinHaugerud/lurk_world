use crate::read::{LurkPollEvent, LurkRead, LurkReadEvent};
use std::io::BufReader;
use std::io::BufWriter;
use std::net::TcpStream;

pub struct ClientFactory {
    id_cursor: u128,
}

impl Default for ClientFactory {
    fn default() -> ClientFactory {
        ClientFactory { id_cursor: 0 }
    }
}

impl ClientFactory {
    pub fn create(&mut self, stream: TcpStream) -> Result<Client, ()> {
        self.id_cursor += 1;
        let write_handle = stream.try_clone().map_err(|_| {})?;
        Ok(Client {
            id: self.id_cursor,
            read: BufReader::new(stream),
            write: BufWriter::new(write_handle),
            poisoned: false,
        })
    }
}

const CLIENT_BUFFER_LIMIT: usize = 1024 * 1024;

pub struct Client {
    id: u128,
    read: BufReader<TcpStream>,
    write: BufWriter<TcpStream>,
    poisoned: bool,
}

impl Client {
    pub fn id(&self) -> u128 {
        self.id
    }
}

pub enum ClientEventKind {
    Read(LurkReadEvent),
    Join,
    Left,
}

pub struct ClientEvent {
    event: ClientEventKind,
    client_id: u128,
}

impl ClientEvent {
    pub fn event(&self) -> &ClientEventKind {
        &self.event
    }

    pub fn client_id(&self) -> u128 {
        self.client_id
    }
}

impl Client {
    pub fn poison(&mut self) {
        self.poisoned = true;
    }

    pub fn poisoned(&self) -> bool {
        self.poisoned
    }

    pub fn poll_event(&mut self) -> Option<ClientEvent> {
        if self.read.buffer().len() > CLIENT_BUFFER_LIMIT {
            self.poison();
        }

        if let Some(event) = self.poll_lurk() {
            Some(ClientEvent {
                event: ClientEventKind::Read(event),
                client_id: self.id,
            })
        } else {
            None
        }
    }

    fn poll_lurk(&mut self) -> Option<LurkReadEvent> {
        if self.poisoned() {
            return None;
        }

        if let Ok(event) = self.read.poll_lurk() {
            match event {
                LurkPollEvent::Pending => None,
                LurkPollEvent::Message(msg) => Some(LurkReadEvent::Message(msg)),
                LurkPollEvent::ChangeRoom(chgrm) => Some(LurkReadEvent::ChangeRoom(chgrm)),
                LurkPollEvent::PVPFight(pvpfight) => Some(LurkReadEvent::PVPFight(pvpfight)),
                LurkPollEvent::Loot(loot) => Some(LurkReadEvent::Loot(loot)),
                LurkPollEvent::Character(ch) => Some(LurkReadEvent::Character(ch)),
                LurkPollEvent::Fight => Some(LurkReadEvent::Fight),
                LurkPollEvent::Start => Some(LurkReadEvent::Start),
                LurkPollEvent::Leave => Some(LurkReadEvent::Leave),
                LurkPollEvent::Version(vers) => Some(LurkReadEvent::Version(vers)),
                LurkPollEvent::Bad => {
                    self.poison();
                    None
                }
            }
        } else {
            self.poison();
            None
        }
    }

    pub fn join(&self) -> ClientEvent {
        ClientEvent {
            event: ClientEventKind::Join,
            client_id: self.id,
        }
    }

    pub fn left(&self) -> ClientEvent {
        ClientEvent {
            event: ClientEventKind::Left,
            client_id: self.id,
        }
    }
}
