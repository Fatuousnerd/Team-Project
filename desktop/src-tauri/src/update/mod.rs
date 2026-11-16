use serde::Serialize;
use std::sync::Mutex;
use tauri::{ipc::Channel,AppHandle,State};
use tauri_plugin_updater::{Updater,UpdaterExt};

#[derive(Debug,thiserror::Error,Serialize)]
pub enum Error {
    #[error(transparent)]
    Updater(#[from] tauri_plugin_updater::Error),

    #[error("there is no pending update")]
    NoPendingUpdate
}

type Result<T> = std::result::Result<T,Error>;

#[derive(Clone,Serialize)]
#[serde(tag="event",content="data")]
pub enum DownloadEvent{
    #[serde(rename_all="camelCase")]
    Started{content_length:Option<u64>},
    
    #[serde(rename_all="camelCase")]
    Progress{chunk_length:usize},
    
    Finished,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct UpdateMetadata{
    pub version:String,
    pub current_version:String
}

#[derive(Default)]
pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[tauri::command]
pub async fn fetch_update(app:AppHandle,pending_update:State<'_,PendingUpdate>)->Result<Option<UpdateMetadata>>{
    let update=app.updater()?.check().await.map_err(Error::from)?;

    let update_metadata=update.as_ref().map(|u|UpdateMetadata{
        version:u.version.clone(),current_version:u.current_version.clone()
    });

    *pending_update.0.lock().unwrap()=update;

    Ok(update_metadata)
}

#[tauri::command]
pub async fn install_update(
    pending_update:State<'_,PendingUpdate>,
    on_event:Channel<DownloadEvent>
)->Result<()>{
    let Some(update)=pending_update.0.lock().unwrap().take() else{
        return Err(Error::NoPendingUpdate);
    };

    update.download_and_install(|chunk_length,content_length|{
        let _=on_event.send(DownloadEvent::Started{content_length});
        let _=on_event.send(DownloadEvent::Progress{content_length});
    }||{
        let _=on_event.send(DownloadEvent::Finished);
    }).await.map_err(Error::from)?;

    Ok(())
}