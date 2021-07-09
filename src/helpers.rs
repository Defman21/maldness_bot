use std::convert::TryFrom;

use frankenstein::Message;
use serde::{Serialize, Deserialize};

use crate::errors::HandleUpdateError;

pub fn get_user_id(message: &Message) -> Result<i64, HandleUpdateError> {
    i64::try_from(
        message
            .from
            .as_ref()
            .ok_or_else(|| HandleUpdateError::Command("from in the message is empty".into()))?
            .id,
    )
        .map_err(|e| HandleUpdateError::Command(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Typing,
    UploadPhoto,
    RecordVideo,
    UploadVideo,
    RecordVoice,
    UploadVoice,
    UploadDocument,
    FindLocation,
    RecordVideoNote,
    UploadVideoNote,
}

impl From<&ActionType> for String {
    fn from(act: &ActionType) -> Self {
        match act {
            ActionType::Typing => "typing".into(),
            ActionType::UploadPhoto => "upload_photo".into(),
            ActionType::RecordVideo => "record_video".into(),
            ActionType::UploadVideo => "upload_video".into(),
            ActionType::RecordVoice => "record_voice".into(),
            ActionType::UploadVoice => "upload_voice".into(),
            ActionType::UploadDocument => "upload_document".into(),
            ActionType::FindLocation => "find_location".into(),
            ActionType::RecordVideoNote => "record_video_note".into(),
            ActionType::UploadVideoNote => "upload_video_note".into(),
        }
    }
}

impl From<ActionType> for String {
    fn from(act: ActionType) -> Self {
        (&act).into()
    }
}
