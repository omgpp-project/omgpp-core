use crate::messages::general_message::general_omgpp_message::CmdRequest;
use std::{collections::HashMap, fmt::Debug};
use uuid::Uuid;

use crate::Endpoint;

type CmdHandlerCallback<T> =
    Box<dyn Fn(&T, &Uuid, &Endpoint, &CmdHandler<T>, &CmdRequest) + 'static>;

pub struct CmdHandler<T> {
    pub cmd: String,
    pub auth_required: bool,
    handler: CmdHandlerCallback<T>,
}
impl<T> CmdHandler<T> {
    pub fn new(cmd: &str, auth_required: bool, handler: CmdHandlerCallback<T>) -> CmdHandler<T> {
        CmdHandler::from_string(String::from(cmd), auth_required, handler)
    }
    pub fn from_string(cmd: String, auth_required: bool, handler: CmdHandlerCallback<T>) -> CmdHandler<T> {
        CmdHandler {
            cmd: cmd,
            auth_required,
            handler,
        }
    }
}
impl<T> Debug for CmdHandler<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CmdHandler")
            .field("cmd", &self.cmd)
            .field("auth_required", &self.auth_required)
            .finish()
    }
}

pub struct CmdHandlerContainer<T> {
    commands: HashMap<String, CmdHandler<T>>,
}
impl<T> CmdHandlerContainer<T> {
    pub fn new() -> CmdHandlerContainer<T> {
        CmdHandlerContainer {
            commands: Default::default(),
        }
    }
    pub fn register_handler(&mut self, cmd_handler: CmdHandler<T>) -> Result<(), String> {
        if self.commands.contains_key(&cmd_handler.cmd) {
            return Result::Err(
                format!("Command {:?} already registered", cmd_handler.cmd).to_string(),
            );
        }
        self.commands.insert(cmd_handler.cmd.clone(), cmd_handler);
        Ok(())
    }

    pub fn handle(&self, item: &T, uuid: &Uuid, endpoint: &Endpoint, cmd: &CmdRequest) {
        if let Some(cmd_handler) = self.commands.get(&cmd.cmd) {
            (cmd_handler.handler)(item, uuid, endpoint, cmd_handler, cmd);
        }
    }
}
