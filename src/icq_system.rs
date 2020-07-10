use actix::prelude::*;
use log;
use std::io::Write;
use std::sync::mpsc;

#[allow(non_snake_case)]
pub mod PurpleMessage {
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Login {}
}

#[derive(Debug)]
pub enum SystemMessage {
    SystemStarted(actix::Addr<ICQSystemActor>),
    Ping,
}

pub struct ICQSystemHandle {
    pub input_rx: os_pipe::PipeReader,
    pub rx: mpsc::Receiver<SystemMessage>,
    pub tx: actix::Addr<ICQSystemActor>,
}

pub fn spawn() -> ICQSystemHandle {
    let (input_rx, input_tx) = os_pipe::pipe().unwrap();
    let (system_tx, system_rx) = std::sync::mpsc::channel();

    log::debug!("Starting actix thread.");
    std::thread::spawn(move || match run(input_tx, system_tx) {
        Ok(_) => log::info!("IcqSystem done"),
        Err(error) => log::error!("IcqSystem failed: {:?}", error),
    });

    log::debug!("Getting actor address...");
    #[allow(unreachable_patterns)]
    let addr = match system_rx.recv().expect("Failed to recv actor addr") {
        SystemMessage::SystemStarted(addr) => addr,
        _ => panic!("SystemStarted must be the first message"),
    };

    ICQSystemHandle {
        input_rx,
        rx: system_rx,
        tx: addr,
    }
}

pub fn run(input_tx: os_pipe::PipeWriter, tx: mpsc::Sender<SystemMessage>) -> std::io::Result<()> {
    log::info!("Starting ICQ");
    let actix = actix_rt::System::new("icq");
    let icq_system = ICQSystemActor::new(input_tx, tx.clone());
    let addr = icq_system.start();
    tx.send(SystemMessage::SystemStarted(addr))
        .expect("Failed to send started event");
    actix.run()
}

pub struct ICQSystemActor {
    input_tx: os_pipe::PipeWriter,
    tx: mpsc::Sender<SystemMessage>,
}

impl ICQSystemActor {
    fn new(input_tx: os_pipe::PipeWriter, tx: mpsc::Sender<SystemMessage>) -> Self {
        Self { input_tx, tx }
    }

    fn ping(&mut self, _context: &mut Context<Self>) {
        self.send_to_purple(SystemMessage::Ping);
    }

    fn send_to_purple(&mut self, message: SystemMessage) {
        self.tx.send(message).unwrap();
        self.input_tx.write(&[0]).unwrap();
    }
}

impl Actor for ICQSystemActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Context<Self>) {
        actix::utils::IntervalFunc::new(std::time::Duration::from_millis(1000), Self::ping)
            .finish()
            .spawn(context);
    }
}

impl std::fmt::Debug for ICQSystemActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ICQSystemActor")
    }
}

impl Handler<PurpleMessage::Login> for ICQSystemActor {
    type Result = ();
    async fn handle(&mut self, _msg: PurpleMessage::Login, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("Login");
        actix::clock::delay_for(std::time::Duration::from_millis(1000)).await;
        log::info!("Login After delay");
    }
}

