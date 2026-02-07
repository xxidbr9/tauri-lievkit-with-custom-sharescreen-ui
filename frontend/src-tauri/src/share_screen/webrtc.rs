// src/share_screen/webrtc.rs
use crate::share_screen::dto::{CaptureError, PreviewOffer, Result};
// use std::collections::HashMap;
use dashmap::DashMap;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
// use webrtc::track::track_local::TrackLocal;

pub struct PreviewConnection {
    peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection>,
    track: Arc<TrackLocalStaticSample>,
}

pub struct WebRTCServer {
    preview_connections: DashMap<String, PreviewConnection>,
}

impl WebRTCServer {
    pub fn new() -> Self {
        Self {
            preview_connections: DashMap::new(),
        }
    }

    pub async fn create_preview_track(
        &mut self,
        id: &str,
        mut frame_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    ) -> Result<()> {
        let track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: "video/VP8".to_owned(),
                ..Default::default()
            },
            format!("video-{}", id),
            format!("preview-{}", id),
        ));

        let track_clone = track.clone();
        tokio::spawn(async move {
            while let Some(data) = frame_rx.recv().await {
                let sample = webrtc::media::Sample {
                    data: data.into(),
                    duration: std::time::Duration::from_millis(100),
                    ..Default::default()
                };
                let _ = track_clone.write_sample(&sample).await;
            }
        });

        // Store track (PC will be created on get_preview_offer)
        self.preview_connections.insert(
            id.to_string(),
            PreviewConnection {
                peer_connection: Arc::new(unsafe { std::mem::zeroed() }),
                track,
            },
        );

        Ok(())
    }

    pub async fn get_preview_offer(&mut self, id: &str) -> Result<PreviewOffer> {
        let mut m = MediaEngine::default();
        m.register_default_codecs()
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        let mut registry = webrtc::interceptor::registry::Registry::new();

        registry = register_default_interceptors(registry, &mut m)
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                // TODO: handle this to be using livekit TURN/STUN
                urls: vec!["".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| CaptureError::WebRTCError(e.to_string()))?,
        );

        let connection = self
            .preview_connections
            .get(id)
            .ok_or_else(|| CaptureError::SourceNotFound(id.to_string()))?;

        let track = connection.track.clone();

        peer_connection
            .add_track(track)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        if let Some(mut conn) = self.preview_connections.get_mut(id) {
            conn.peer_connection = peer_connection;
        }

        Ok(PreviewOffer {
            id: id.to_string(),
            sdp: offer.sdp,
        })
    }

    pub async fn accept_preview_answer(&mut self, id: &str, sdp: String) -> Result<()> {
        let connection = self
            .preview_connections
            .get(id)
            .ok_or_else(|| CaptureError::SourceNotFound(id.to_string()))?;

        let answer = RTCSessionDescription::answer(sdp)
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        connection
            .peer_connection
            .set_remote_description(answer)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        Ok(())
    }

    pub async fn close_preview(&mut self, id: &str) {
        if let Some((_key, connection)) = self.preview_connections.remove(id) {
            let _ = connection.peer_connection.close().await;
        }
    }

    pub async fn close_all_previews(&mut self) {
        let keys: Vec<_> = self
            .preview_connections
            .iter()
            .map(|e| e.key().clone())
            .collect();

        for k in keys {
            if let Some((_, conn)) = self.preview_connections.remove(&k) {
                let _ = conn.peer_connection.close().await;
            }
        }
    }
}
