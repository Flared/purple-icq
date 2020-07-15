use super::{FdSender, SystemMessage};
use crate::purple::Account;
use async_std::sync::channel;
#[derive(Debug, Clone)]
pub struct AccountHandle(*mut purple_sys::PurpleAccount);

// AccountHandle are safe to clone and send to other thread.
unsafe impl Send for AccountHandle {}

impl AccountHandle {
    pub fn as_account(&self) -> Account {
        unsafe { Account::from_raw(self.0) }
    }
    pub fn proxy<'a>(&self, sender: &'a mut FdSender<SystemMessage>) -> AccountProxy<'a> {
        AccountProxy {
            handle: self.clone(),
            sender,
        }
    }
}

pub struct AccountProxy<'a> {
    handle: AccountHandle,
    sender: &'a mut FdSender<SystemMessage>,
}
impl<'a> AccountProxy<'a> {
    pub async fn exec<F>(&mut self, f: F)
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
        self.exec(move |account| {
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
                    tx.try_send(input_value.map(|v| v.into_owned()));
                },
                who.as_deref(),
            )
        })
        .await;

        rx.recv().await.unwrap()
    }
}

impl std::convert::From<&Account> for AccountHandle {
    fn from(account: &Account) -> Self {
        return Self(account.as_ptr());
    }
}
