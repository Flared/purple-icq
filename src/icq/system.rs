use super::protocol;
use crate::messages::{
    AccountHandle, AccountInfo, FdSender, ICQSystemHandle, PurpleMessage, SystemMessage,
};
use crate::purple;
use async_std::sync::{channel, Receiver};

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
        }
    }

    async fn login(&mut self, account_info: AccountInfo) {
        log::debug!("login");
        let mut registered_account_info = {
            account_info
                .account
                .proxy(&mut self.tx)
                .exec(|account| {
                    let token = account.get_string("token", "");
                    if token == "" {
                        None
                    } else {
                        Some(protocol::RegisteredAccountInfo {
                            token,
                            session_id: account.get_string("session_id", ""),
                            session_key: account.get_string("session_key", ""),
                            host_time: account.get_int("host_time", 0) as u32,
                        })
                    }
                })
                .await
        };
        if registered_account_info.is_none() {
            match protocol::register(&account_info.phone_number, || {
                log::debug!("read_code");
                self.read_code(&account_info.account)
            })
            .await
            {
                Ok(info) => {
                    account_info
                        .account
                        .proxy(&mut self.tx)
                        .set_settings(info.clone())
                        .await
                        .expect("Failed to write settings");
                    registered_account_info = Some(info);
                }
                Err(error) => {
                    log::error!("Failed to register account: {:?}", error);
                }
            }
        }

        log::debug!("Registered account info: {:?}", registered_account_info);

        if let Some(registered_account_info) = registered_account_info {
            let session_info = protocol::start_session(&registered_account_info);
            log::debug!("Session info: {:?}", session_info);
        }

        account_info
            .account
            .get_connection()
            .proxy(&mut self.tx)
            .set_state(purple::PurpleConnectionState::PURPLE_CONNECTING)
            .await;
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
