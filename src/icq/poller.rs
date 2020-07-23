use super::client::events;
use super::client::events::Event;
use super::client::events::EventData;
use super::client::try_result;
use super::client::try_result::TryResult;
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
    events: Vec<TryResult<Event>>,
) -> u32 {
    let mut next_seq_num = 0;

    for event in events {
        log::info!("Processing event: {:?}", event);

        match event {
            try_result::TryResult(Ok(event)) => {
                next_seq_num = std::cmp::max(next_seq_num, event.seq_num + 1);

                match event.event_data {
                    EventData::BuddyList(event_data) => {
                        process_event_buddy_list(&event_data).await;
                    }
                    EventData::HistDlgState(event_data) => {
                        process_event_hist_dlg_state(&event_data).await;
                    }
                    EventData::MyInfo(event_data) => {
                        process_event_my_info(&event_data).await;
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

            try_result::TryResult(Err(unknown_event)) => {
                match serde_json::to_string(&unknown_event) {
                    Ok(event_str) => {
                        log::error!("Unknown event: {}", event_str);
                    }
                    Err(_err) => {
                        log::error!("Unknown event: {}", unknown_event);
                    }
                }
            }
        }
    }

    next_seq_num
}

pub async fn process_event_buddy_list(event_data: &events::BuddyListData) {
    for group in &event_data.groups {
        for buddy in &group.buddies {
            match &buddy.user_type {
                events::UserType::Chat => {
                    // Do we already have a chat?
                }
                events::UserType::ICQ => {}
                events::UserType::Unknown => {
                    log::error!("Got unknown user type!");
                }
            }
        }
    }
}

pub async fn process_event_my_info(_event_data: &events::MyInfoData) {
    // TODO
    // purple_notify_userinfo
}

pub async fn process_event_hist_dlg_state(_event_data: &events::HistDlgStateData) {
    // TODO
}
