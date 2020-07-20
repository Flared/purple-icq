use crate::Handle;
use super::{FdSender, SystemMessage};
use crate::purple::{account, Account};
use async_std::sync::channel;

pub struct AccountProxy<'a> {
    handle: Handle,
    sender: &'a mut FdSender<SystemMessage>,
}
impl<'a> AccountProxy<'a> {
    pub async fn exec<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(Account) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel(1);
        self.exec_no_return(move |account| {
            if let Err(error) = tx.try_send(f(account)) {
                log::error!("Failed to send result: {:?}", error);
            }
        })
        .await;
        rx.recv().await.expect("Failed to receive result")
    }

    pub async fn exec_no_return<F>(&mut self, f: F)
    where
        F: FnOnce(Account),
        F: Send + 'static,
    {
        self.sender
            .send(SystemMessage::ExecAccount {
                handle: self.handle.clone(),
                function: Box::new(f),
            })
            .await;
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn request_input(
        &mut self,
        title: Option<String>,
        primary: Option<String>,
        secondary: Option<String>,
        default_value: Option<String>,
        multiline: bool,
        masked: bool,
        hint: Option<String>,
        ok_text: String,
        cancel_text: String,
        who: Option<String>,
    ) -> Option<String> {
        let (tx, rx) = channel(1);
        self.exec_no_return(move |account| {
            account.request_input(
                title.as_deref(),
                primary.as_deref(),
                secondary.as_deref(),
                default_value.as_deref(),
                multiline,
                masked,
                hint.as_deref(),
                &ok_text,
                &cancel_text,
                move |input_value| {
                    if let Err(error) = tx.try_send(input_value.map(|v| v.into_owned())) {
                        log::error!("Failed to send result: {:?}", error);
                    }
                },
                who.as_deref(),
            )
        })
        .await;

        rx.recv().await.expect("Failed to receive result")
    }

    pub async fn set_settings<T: 'static + serde::Serialize + Send>(
        &mut self,
        settings: T,
    ) -> account::settings::Result<()> {
        self.exec(move |account| account.set_settings(&settings))
            .await
    }
}


