use super::client::events;
use super::client::events::Event;
use super::client::events::EventData;
use super::client::try_result;
use super::client::try_result::TryResult;
use super::protocol;
use crate::messages::{AccountInfo, FdSender, SystemMessage};
use crate::ChatInfo;
use crate::MsgInfo;
use std::time::Duration;

pub async fn fetch_events_loop(mut tx: FdSender<SystemMessage>, account_info: AccountInfo) {
    let mut fetch_base_url = {
        account_info
            .protocol_data
            .lock()
            .await
            .session
            .as_ref()
            .unwrap()
            .fetch_base_url
            .clone()
    };

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

        match protocol::fetch_events(&fetch_base_url).await {
            Err(error) => {
                log::error!("Failed to fetch events: {:?}", error);
                async_std::task::sleep(Duration::from_secs(5)).await;
            }
            Ok(fetch_events_response_data) => {
                log::info!("Fetched Events: {:?}", fetch_events_response_data.events);
                process_events(tx.clone(), &account_info, fetch_events_response_data.events).await;
                fetch_base_url = fetch_events_response_data.fetch_base_url;
            }
        }
    }
}

pub async fn process_events(
    tx: FdSender<SystemMessage>,
    account_info: &AccountInfo,
    events: Vec<TryResult<Event>>,
) {
    for event in events {
        log::info!("Processing event: {:?}", event);

        match event {
            try_result::TryResult(Ok(event)) => {
                match event.event_data {
                    EventData::BuddyList(event_data) => {
                        process_event_buddy_list(&event_data).await;
                    }
                    EventData::HistDlgState(event_data) => {
                        process_event_hist_dlg_state(tx.clone(), account_info, &event_data).await;
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
                    EventData::ImStates(_event_data) => {
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

fn find_author_friendly<'a>(
    author_sn: &'a str,
    persons: &'a [events::HistDlgStatePerson],
) -> &'a str {
    let mut author_friendly = author_sn;
    for person in persons {
        if person.sn.eq(author_sn) {
            author_friendly = &person.friendly;
            break;
        }
    }
    author_friendly
}

pub async fn process_event_hist_dlg_state(
    mut tx: FdSender<SystemMessage>,
    account_info: &AccountInfo,
    event_data: &events::HistDlgStateData,
) {
    // Create the chat if necessary
    let chat_sn = event_data.sn.clone();
    let chat_friendly = find_author_friendly(&chat_sn, &event_data.persons).to_string();

    let chat_info = ChatInfo {
        sn: chat_sn.clone(),
        stamp: None,
        title: chat_friendly,
        group: None,
    };

    tx.handle_proxy(&account_info.handle)
        .exec(move |plugin, protocol_data| {
            let connection = &mut protocol_data.connection;
            plugin.chat_joined(connection, &chat_info);
        })
        .await;

    // Create Chat Entries
    for message in &event_data.messages {
        // For group conversation:
        // - event_data.sn is the Group's sn
        // - message.chat.sender is the author's sn
        //
        // For DMs:
        // - message.chat is not there
        // - event_data.sn is the author's sn
        let author_sn = {
            match &message.chat {
                Some(chat) => chat.sender.clone(),
                None => event_data.sn.clone(),
            }
        };
        let author_friendly = find_author_friendly(&author_sn, &event_data.persons).to_string();

        let chat_input = MsgInfo {
            chat_sn: chat_sn.clone(),
            author_sn,
            author_friendly,
            text: message.text.clone(),
            time: message.time,
        };

        tx.connection_proxy(&account_info.handle)
            .exec(move |connection| {
                connection.serv_got_chat_in(chat_input);
            })
            .await;
    }
}
