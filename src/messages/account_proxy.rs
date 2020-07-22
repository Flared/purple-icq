use super::{FdSender, SystemMessage};
use crate::purple::{account, Account};
use crate::Handle;
use async_std::sync::channel;

pub struct AccountProxy<'a> {
    pub handle: Handle,
    pub sender: &'a mut FdSender<SystemMessage>,
}
impl<'a> AccountProxy<'a> {
    pub async fn exec<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Account) -> T,
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
        rx.recv().await.ok().or_else(|| {
            log::error!("Failed to receive result");
            None
        })
    }

    pub async fn exec_no_return<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Account),
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

        rx.recv().await.ok().flatten()
    }

    pub async fn is_disconnected(&mut self) -> bool {
        self.exec(move |account| account.is_disconnected())
            .await
            .unwrap_or(false)
    }

    pub async fn set_settings<T: 'static + serde::Serialize + Send>(
        &mut self,
        settings: T,
    ) -> account::settings::Result<()> {
        self.exec(move |account| account.set_settings(&settings))
            .await
            .transpose()
            .and_then(|option| {
                option.ok_or_else(|| {
                    account::settings::Error::Message("Failed to receive result".into())
                })
            })
    }
}
