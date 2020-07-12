use async_std::sync::{channel, Receiver, Sender};
use log;

const CHANNEL_CAPACITY: usize = 1024;

#[derive(Debug)]
pub enum PurpleMessage {
    Login,
}

#[derive(Debug)]
pub enum SystemMessage {
    Ping,
}

pub struct ICQSystemHandle {
    pub input_rx: os_pipe::PipeReader,
    pub rx: Receiver<SystemMessage>,
    pub tx: Sender<PurpleMessage>,
}

pub fn spawn() -> ICQSystemHandle {
    let (input_rx, input_tx) = os_pipe::pipe().unwrap();
    let (system_tx, system_rx) = channel(CHANNEL_CAPACITY);
    let (purple_tx, purple_rx) = channel(CHANNEL_CAPACITY);

    log::debug!("Starting async thread.");
    std::thread::spawn(move || run(input_tx, system_tx, purple_rx));

    ICQSystemHandle {
        input_rx,
        rx: system_rx,
        tx: purple_tx,
    }
}

pub fn run(input_tx: os_pipe::PipeWriter, tx: Sender<SystemMessage>, rx: Receiver<PurpleMessage>) {
    log::info!("Starting ICQ");
    let mut system = ICQSystem::new(input_tx, tx, rx);
    async_std::task::block_on(system.run());
}

pub struct ICQSystem {
    input_tx: os_pipe::PipeWriter,
    tx: Sender<SystemMessage>,
    rx: Receiver<PurpleMessage>,
}

impl ICQSystem {
    fn new(
        input_tx: os_pipe::PipeWriter,
        tx: Sender<SystemMessage>,
        rx: Receiver<PurpleMessage>,
    ) -> Self {
        Self { input_tx, tx, rx }
    }

    async fn run(&mut self) {
        loop {
            let purple_message = self.rx.recv().await;
            log::info!("Message: {:?}", purple_message);
            self.ping().await;
        }
    }

    async fn ping(&mut self) {
        self.send_to_purple(SystemMessage::Ping).await;
    }

    async fn send_to_purple(&mut self, message: SystemMessage) {
        self.tx.send(message).await;
        use std::io::Write;
        self.input_tx.write(&[0]).unwrap();
    }
}
