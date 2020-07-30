use super::poller;
use super::protocol;
use crate::logging;
use crate::messages::{
    AccountInfo, FdSender, GetChatInfoMessage, ICQSystemHandle, JoinChatMessage, PurpleMessage,
    SendMsgMessage, SystemMessage,
};
use crate::purple;
use crate::{ChatInfo, Handle};
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
    logging::set_thread_logger(logging::RemoteLogger(tx.clone()));
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
            let result = match purple_message {
                PurpleMessage::Login(account_info) => self.login(account_info).await,
                PurpleMessage::JoinChat(m) => self.join_chat(m).await,
                PurpleMessage::SendMsg(m) => self.send_msg(m).await,
                PurpleMessage::GetChatInfo(m) => self.get_chat_info(m).await,
            };
            if let Err(error) = result {
                log::error!("Error handling message: {}", error);
            }
            logging::flush();
        }
    }

    async fn login(&mut self, account_info: AccountInfo) -> std::result::Result<(), String> {
        log::debug!("login");
        let phone_number = { account_info.protocol_data.phone_number.clone() };
        let handle = &account_info.handle;
        let mut registered_account_info = {
            self.tx
                .account_proxy(&handle)
                .exec(|account| {
                    let token =
                        account.get_string(protocol::RegistrationData::TOKEN_SETTING_KEY, "");
                    if token == "" {
                        None
                    } else {
                        Some(protocol::RegistrationData {
                            token,
                            session_id: account
                                .get_string(protocol::RegistrationData::SESSION_ID_SETTING_KEY, ""),
                            session_key: account.get_string(
                                protocol::RegistrationData::SESSION_KEY_SETTING_KEY,
                                "",
                            ),
                            host_time: account
                                .get_int(protocol::RegistrationData::HOST_TIME_SETTING_KEY, 0)
                                as u32,
                        })
                    }
                })
                .await
                .ok_or_else(|| "Failed to read settings".to_string())?
        };
        if registered_account_info.is_none() {
            let info = protocol::register(&phone_number, || {
                log::debug!("read_code");
                self.read_code(&account_info.handle)
            })
            .await
            .map_err(|e| format!("Failed to register account: {:?}", e))?;

            self.tx
                .account_proxy(&handle)
                .set_settings(info.clone())
                .await
                .map_err(|e| format!("Failed to write settings: {:?}", e))?;

            registered_account_info = Some(info);
        }

        log::debug!("Registered account info: {:?}", registered_account_info);
        if registered_account_info.is_none() {
            self.tx
                .connection_proxy(&handle)
                .error_reason(
                    purple::PurpleConnectionError::PURPLE_CONNECTION_ERROR_AUTHENTICATION_FAILED,
                    "Failed to register account".into(),
                )
                .await;
            return Err("Failed to register account".into());
        }

        if let Some(registered_account_info) = registered_account_info {
            self.tx
                .connection_proxy(&handle)
                .set_state(purple::PurpleConnectionState::PURPLE_CONNECTING)
                .await;

            let session_info = protocol::start_session(&registered_account_info).await;
            log::debug!("Session info: {:?}", session_info);
            match session_info {
                Ok(session) => {
                    self.tx
                        .connection_proxy(&handle)
                        .set_state(purple::PurpleConnectionState::PURPLE_CONNECTED)
                        .await;
                    (*account_info.protocol_data.session.write().await) = Some(session);
                    async_std::task::spawn_local(poller::fetch_events_loop(
                        self.tx.clone(),
                        account_info.clone(),
                    ));
                }
                Err(error) => {
                    let error_message = format!("Failed to start session: {:?}", error);
                    self.tx
                        .connection_proxy(&handle)
                        .error_reason(purple::PurpleConnectionError::PURPLE_CONNECTION_ERROR_AUTHENTICATION_FAILED,
                                      error_message.clone()).await;
                    return Err(error_message);
                }
            }
        }
        Ok(())
    }

    async fn read_code(&mut self, handle: &Handle) -> Option<String> {
        let code = self
            .tx
            .account_proxy(&handle)
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

    async fn get_chat_info(&mut self, message: GetChatInfoMessage) -> Result<(), String> {
        log::info!("Get chat info sn: {}", message.message_data.sn);
        let session = { message.protocol_data.session.read().await.clone().unwrap() };
        let chat_info_response = protocol::get_chat_info_by_sn(&session, &message.message_data.sn)
            .await
            .map_err(|e| format!("Failed to get chat info: {:?}", e))?;

        self.tx
            .handle_proxy(&message.handle)
            .exec_no_return(move |plugin, protocol_data| {
                let chat_info = ChatInfo::from(chat_info_response);
                let connection = &mut protocol_data.connection;
                plugin.load_chat_info(connection, &chat_info);
            })
            .await;

        Ok(())
    }

    async fn join_chat(&mut self, message: JoinChatMessage) -> Result<(), String> {
        log::info!("Joining stamp: {}", message.message_data.stamp);
        let session = { message.protocol_data.session.read().await.clone().unwrap() };
        let stamp = message.message_data.stamp;
        // Handle shareable URLs: https://icq.im/XXXXXXXXXXXXXX
        let stamp = if stamp.contains("icq.im/") {
            stamp.rsplit('/').next().unwrap().into()
        } else {
            stamp
        };

        protocol::join_chat(&session, &stamp)
            .await
            .map_err(|e| format!("Failed to join chat: {:?}", e))?;
        let chat_info_response = protocol::get_chat_info(&session, &stamp)
            .await
            .map_err(|e| format!("Failed to get chat info: {:?}", e))?;

        self.tx
            .handle_proxy(&message.handle)
            .exec_no_return(move |plugin, protocol_data| {
                let chat_info = ChatInfo::from(chat_info_response);
                let partial_info = chat_info.as_partial();
                let connection = &mut protocol_data.connection;
                plugin.chat_joined(connection, &partial_info);
                plugin.conversation_joined(connection, &partial_info);
                plugin.load_chat_info(connection, &chat_info);
            })
            .await;

        Ok(())
    }

    async fn send_msg(&mut self, message: SendMsgMessage) -> Result<(), String> {
        log::info!("send_msg({:?})", message);
        let to_sn = &message.message_data.to_sn;
        let message_body = &message.message_data.message;
        let session = { message.protocol_data.session.read().await.clone().unwrap() };
        let _msg_info = protocol::send_im(&session, to_sn, message_body)
            .await
            .map_err(|e| format!("Failed to send msg: {:?}", e))?;
        Ok(())
    }
}
