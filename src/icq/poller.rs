use super::client::events::Event;
use super::client::events::EventData;
use super::protocol;
use crate::messages::{AccountInfo, FdSender, SystemMessage};
use std::time::Duration;

pub async fn fetch_events_loop(mut tx: FdSender<SystemMessage>, account_info: AccountInfo) {
    //TODO: Where to begin?
    let mut seq_num = 0;

    loop {
        // Skip if we are disconnected.
        if tx
            .account_proxy(&account_info.handle.clone())
            .is_disconnected()
            .await
        {
            log::info!("Account is disconnected. Not fetching events.");
            async_std::task::sleep(Duration::from_secs(5)).await;
        }

        // Fetch Events
        log::info!("Fetching events...");

        let session = {
            account_info
                .protocol_data
                .lock()
                .await
                .session
                .clone()
                .unwrap()
        };

        match protocol::fetch_events(&session, seq_num).await {
            Err(error) => {
                log::error!("Failed to fetch events: {:?}", error);
                async_std::task::sleep(Duration::from_secs(5)).await;
            }
            Ok(events) => {
                log::info!("Fetched Events: {:?}", events);
                seq_num = process_events(&tx, &account_info, events).await;
            }
        }
    }
}

pub async fn process_events(
    _tx: &FdSender<SystemMessage>,
    _account_info: &AccountInfo,
    events: Vec<Event>,
) -> u32 {
    let mut next_seq_num = 0;

    for event in events {
        log::info!("Processing event: {:?}", event);
        next_seq_num = std::cmp::max(next_seq_num, event.seq_num + 1);

        match event.event_data {
            EventData::BuddyList(_event_data) => {
                // TODO
            }
            EventData::HistDlgState(_event_data) => {
                // TODO
            }
            EventData::MyInfo(_event_data) => {
                // TODO
                // purple_notify_userinfo
            }
            EventData::PermitDeny(_event_data) => {
                // TODO
            }
            EventData::Presence(_event_data) => {
                // TODO
            }
            EventData::GalleryNotify(_event_data) => {
                // TODO
            }
        }
    }

    next_seq_num
}
