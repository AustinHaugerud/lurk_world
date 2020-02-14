use crate::client::{Client, ClientFactory, ClientEvent};
use crate::read::LurkReadEvent;
use rlua::prelude::LuaTable;
use rlua::Value::Nil;
use rlua::{Lua, UserData, UserDataMethods};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::io::Error;
use std::net::TcpStream;
use crate::lua::ClientEventBuffer;
use crate::cli::Args;

pub fn server(args: &Args) {
    use std::collections::HashMap;
    use std::net::TcpListener;
    use std::fs::read_to_string;

    let mut client_factory = ClientFactory::default();

    let server_address = format!("0.0.0.0:{}", args.port);

    let main_script_path = format!("{}/main.lua", &args.module);
    let script_src = read_to_string(&main_script_path).expect("Failed to load server script 'main.lua'.");

    let listener: TcpListener = TcpListener::bind(server_address).unwrap_or_else(|_| panic!(""));

    listener
        .set_nonblocking(true)
        .expect("Failed to set listener to non-blocking.");

    let mut events_buffer = ClientEventBuffer::default();
    let buffer_lua_handle = events_buffer.clone();

    let lua = Lua::new();
    lua.context(|ctx| {
        ctx.globals()
            .set("Events", buffer_lua_handle)
            .expect("Failed to globalize Lua events table.");
    });

    let mut clients: HashMap<u128, Client> = HashMap::new();

    let running = true;

    while running {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    if s.set_nonblocking(true).is_ok() {
                        if let Ok(new_client) = client_factory.create(s) {
                            events_buffer.add(new_client.join());
                            clients.insert(new_client.id(), new_client);
                        }
                    }
                }
                _ => { break; }
            }
        }

        for (_, client) in clients.iter_mut() {
            while let Some(client_event) = client.poll_event() {
                events_buffer.add(client_event);
            }
        }

        lua.context(|ctx| {
            ctx.load(&script_src)
                .set_name("main")
                .unwrap()
                .exec()
                .unwrap();
        });
    }
}
