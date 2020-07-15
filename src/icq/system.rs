use super::protocol;
use crate::messages::{
    AccountHandle, AccountInfo, FdSender, ICQSystemHandle, PurpleMessage, SystemMessage,
};
use async_std::sync::{channel, Receiver};
use log;

const CHANNEL_CAPACITY: usize = 1024;

pub fn spawn() -> ICQSystemHandle {
    let (input_rx, input_tx) = os_pipe::pipe().unwrap();
    let (system_tx, system_rx) = channel(CHANNEL_CAPACITY);
    let (purple_tx, purple_rx) = channel(CHANNEL_CAPACITY);

    let fd_sender = FdSender::new(input_tx, system_tx);

    log::debug!("Starting async thread.");
    std::thread::spawn(move || run(fd_sender, purple_rx));

    ICQSystemHandle {
        input_rx,
        rx: system_rx,
        tx: purple_tx,
    }
}

pub fn run(tx: FdSender<SystemMessage>, rx: Receiver<PurpleMessage>) {
    log::info!("Starting ICQ");
    let mut system = ICQSystem::new(tx, rx);
    async_std::task::block_on(system.run());
}

pub struct ICQSystem {
    tx: FdSender<SystemMessage>,
    rx: Receiver<PurpleMessage>,
}

impl ICQSystem {
    fn new(tx: FdSender<SystemMessage>, rx: Receiver<PurpleMessage>) -> Self {
        Self { tx, rx }
    }

    async fn run(&mut self) {
        loop {
            let purple_message = match self.rx.recv().await {
                Ok(r) => r,
                Err(error) => {
                    log::error!("Failed to receive message: {:?}", error);
                    break;
                }
            };
            log::info!("Message: {:?}", purple_message);
            match purple_message {
                PurpleMessage::Login(account_info) => self.login(account_info).await,
            }
            self.ping().await
        }
    }

    async fn ping(&mut self) {
        self.tx.send(SystemMessage::Ping).await;
    }

    async fn login(&mut self, account_info: AccountInfo) {
        log::debug!("login");
        match protocol::register(&account_info.phone_number, || {
            log::debug!("read_code");
            self.read_code(&account_info.account)
        })
        .await
        {
            Ok(_) => (),
            Err(error) => {
                log::error!("Failed to register account: {:?}", error);
            }
        }
    }

    async fn read_code(&mut self, account: &AccountHandle) -> Option<String> {
        let code = account
            .proxy(&mut self.tx)
            .request_input(
                Some("SMS Code".into()),
                Some("Enter SMS code".into()),
                Some("You will be sent an SMS message containing your auth code.".into()),
                None,
                false,
                false,
                None,
                "Login".into(),
                "Cancel".into(),
                None,
            )
            .await;
        log::info!("Code: {:?}", code);
        code
    }
}
