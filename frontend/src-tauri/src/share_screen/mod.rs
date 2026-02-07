// src/share_screen/mod.rs
pub mod capture;
pub mod command;
pub mod dto;
pub mod manager;
pub mod webrtc;

use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref MANAGER: Arc<RwLock<manager::CaptureManager>> =
        Arc::new(RwLock::new(manager::CaptureManager::new()));
    pub static ref WEBRTC_SERVER: Arc<RwLock<webrtc::WebRTCServer>> =
        Arc::new(RwLock::new(webrtc::WebRTCServer::new()));
}
