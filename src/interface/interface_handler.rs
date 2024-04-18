use std::{
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

use glib::Receiver;

use crate::node::interface::interface_communicator::InterfaceMessages;

use super::screen::Screen;

pub struct InterfaceHandler {}

impl InterfaceHandler {
    pub fn start(
        sender_to_node: Sender<InterfaceMessages>,
        receiver_from_node: Receiver<InterfaceMessages>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            if let Err(err) = gtk::init() {
                eprintln!("Error initializing GTK: {}", err);
            }

            Screen::start(sender_to_node, receiver_from_node);
        })
    }
}
