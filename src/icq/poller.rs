use super::client;
use super::client::events;
use super::client::events::Event;
use super::client::events::EventData;
use super::client::try_result;
use super::client::try_result::TryResult;
use super::protocol;
use crate::messages::{AccountInfo, FdSender, SystemMessage};
use crate::MsgInfo;
use crate::PartialChatInfo;
use futures::future;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::time::Duration;

static FILES_URL_PREFIX: &str = "https://files.icq.net/get/";

pub async fn fetch_events_loop(mut tx: FdSender<SystemMessage>, account_info: AccountInfo) {
    let mut fetch_base_url = {
        account_info
            .protocol_data
            .session
            .read()
            .await
            .as_ref()
            .unwrap()
            .fetch_base_url
            .clone()
    };

    while !account_info
        .protocol_data
        .session_closed
        .load(Ordering::Relaxed)
    {
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
                        process_event_buddy_list(tx.clone(), account_info, &event_data).await;
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
                    EventData::ImState(_event_data) => {
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

pub async fn process_event_buddy_list(
    mut tx: FdSender<SystemMessage>,
    account_info: &AccountInfo,
    event_data: &events::BuddyListData,
) {
    for group in &event_data.groups {
        for buddy in &group.buddies {
            match &buddy.user_type {
                events::UserType::Chat => {
                    let chat_info = PartialChatInfo {
                        sn: buddy.aim_id.clone(),
                        title: match &buddy.friendly {
                            Some(friendly) => friendly.clone(),
                            None => buddy.aim_id.clone(),
                        },
                        group: Some(group.name.clone()),
                    };
                    tx.handle_proxy(&account_info.handle)
                        .exec_no_return(move |plugin, protocol_data| {
                            let connection = &mut protocol_data.connection;
                            plugin.chat_joined(connection, &chat_info);
                        })
                        .await;
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
    let session = {
        account_info
            .protocol_data
            .session
            .read()
            .await
            .as_ref()
            .unwrap()
            .clone()
    };

    // Create the chat if necessary
    let chat_sn = event_data.sn.clone();
    let chat_friendly = find_author_friendly(&chat_sn, &event_data.persons).to_string();

    let chat_info = PartialChatInfo {
        sn: chat_sn.clone(),
        title: chat_friendly,
        ..Default::default()
    };
    let info_version = event_data.mchat_state.clone().map(Into::into);

    tx.handle_proxy(&account_info.handle)
        .exec_no_return(move |plugin, protocol_data| {
            let connection = &mut protocol_data.connection;
            plugin.chat_joined(connection, &chat_info);
        })
        .await;

    // Create Chat Entries
    for message in &event_data.messages {
        let message_text = match message.text.as_ref() {
            Some(m) => m,
            None => continue,
        };
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

        let message_text = htmlescape::encode_minimal(&message_text);
        let message_text = process_message_files(Cow::from(&message_text), &session).await;

        let chat_input = MsgInfo {
            chat_sn: chat_sn.clone(),
            author_sn,
            author_friendly,
            text: message_text.into_owned(),
            time: message.time,
        };

        tx.handle_proxy(&account_info.handle)
            .exec_no_return(move |plugin, protocol_data| {
                let connection = &mut protocol_data.connection;
                plugin.serv_got_chat_in(connection, chat_input);
            })
            .await;
    }

    tx.handle_proxy(&account_info.handle)
        .exec_no_return(move |plugin, protocol_data| {
            let connection = &mut protocol_data.connection;
            if let Some(ref info_version) = info_version {
                plugin.check_chat_info(connection, &chat_sn, info_version);
            }
        })
        .await;
}

// Clippy false positive, lifetimes are required for this code to compile.
#[allow(clippy::needless_lifetimes)]
async fn process_message_files<'a>(
    mut message: Cow<'a, str>,
    session: &protocol::SessionInfo,
) -> Cow<'a, str> {
    // Extract files URL from message.
    let message_files = message
        .match_indices(FILES_URL_PREFIX)
        .map(|(index, _)| {
            let id_starts_at = index + FILES_URL_PREFIX.len();
            let id_ends_at = message[id_starts_at..]
                .find(' ')
                .map(|i| id_starts_at + i)
                .unwrap_or_else(|| message.len());
            log::info!("{}..{}..{}", index, id_starts_at, id_ends_at);
            let file_id = message[id_starts_at..id_ends_at].to_string();
            (index..id_ends_at, file_id)
        })
        .collect::<Vec<_>>();

    // Fetch all files info through ICQ API.
    let files_info_futures = message_files
        .iter()
        .map(|(_, file_id)| file_id.to_string())
        .collect::<HashSet<String>>()
        .into_iter()
        .map(|file_id| fetch_file_info(session, file_id));

    let files_info = future::join_all(files_info_futures)
        .await
        .into_iter()
        .collect::<HashMap<_, _>>();

    // Replace message files reference with valid link.
    for (range, file_id) in message_files.into_iter().rev() {
        if let Ok(file_info) = files_info.get(&file_id).unwrap() {
            let file_html = format_file_info(file_info);
            message.to_mut().replace_range(range, &file_html);
        }
    }

    message
}

async fn fetch_file_info(
    session: &protocol::SessionInfo,
    file_id: String,
) -> (String, protocol::Result<client::FilesInfoResponseData>) {
    (
        file_id.clone(),
        protocol::files_info(session, &file_id).await,
    )
}

fn format_file_info(file_info: &client::FilesInfoResponseData) -> String {
    let info = &file_info.info;
    let pretty_mime = info.mime.rsplit('/').next().unwrap();
    format!(
        "<a href=\"{href}\" class=\"file\" data-mime=\"{mime}\" data-md5=\"{md5}\" data-filesize=\"{size}\">{name} [{pretty_mime} {pretty_size}]</a>",
        href = htmlescape::encode_attribute(&info.dlink),
        md5 = htmlescape::encode_attribute(&info.md5),
        mime = htmlescape::encode_attribute(&info.mime),
        pretty_mime = htmlescape::encode_minimal(&pretty_mime),
        name = htmlescape::encode_minimal(&info.file_name),
        size = info.file_size,
        pretty_size = pretty_size(info.file_size)
    )
}

fn pretty_size(size: usize) -> String {
    if size < 100_000 {
        (size / 1000).to_string() + "kb"
    } else if size < 1_000_000_000 {
        (size / 1_000_000).to_string() + "mb"
    } else {
        (size / 1_000_000_000).to_string() + "gb"
    }
}
