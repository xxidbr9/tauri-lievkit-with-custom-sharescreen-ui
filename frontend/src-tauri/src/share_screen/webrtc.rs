// src/share_screen/webrtc.rs
use crate::share_screen::dto::{CaptureError, PreviewOffer, Result};
// use std::collections::HashMap;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::media::Sample;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
// use webrtc::track::track_local::TrackLocal;

pub struct PreviewConnection {
    peer_conn: Option<Arc<RTCPeerConnection>>,
    track: Arc<TrackLocalStaticSample>,
}

pub struct WebRTCServer {
    preview_connections: Mutex<HashMap<String, PreviewConnection>>,
    // api: Arc<API>,
}

impl WebRTCServer {
    pub fn new() -> Self {
        Self {
            preview_connections: Mutex::new(HashMap::new()),
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
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            format!("video-{}", id),
            format!("preview-{}", id),
        ));

        let track_clone = track.clone();
        let id_clone = id.to_string();

        tokio::spawn(async move {
            let mut frame_count = 0u64;

            while let Some(vp8_data) = frame_rx.recv().await {
                frame_count += 1;

                let sample = Sample {
                    data: vp8_data.into(),
                    duration: std::time::Duration::from_millis(100),
                    timestamp: SystemTime::now(),
                    packet_timestamp: 0,
                    prev_dropped_packets: 0,
                    prev_padding_packets: 0,
                };

                if let Err(e) = track_clone.write_sample(&sample).await {
                    eprintln!(
                        "[WebRTC] Failed to write VP8 sample for {}: {}",
                        id_clone, e
                    );
                }
            }
        });

        let mut map = self.preview_connections.lock().await;
        // Store track (PC will be created on get_preview_offer)
        map.insert(
            id.to_string(),
            PreviewConnection {
                peer_conn: None,
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
            ..Default::default()
        };

        let peer_connection = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| CaptureError::WebRTCError(e.to_string()))?,
        );

        let track = {
            let mut map = self.preview_connections.lock().await;
            let conn = map
                .get_mut(id)
                .ok_or_else(|| CaptureError::SourceNotFound(id.to_string()))?;

            let track = conn.track.clone();
            track
        };

        peer_connection
            .add_track(track)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        {
            let mut map = self.preview_connections.lock().await;
            if let Some(conn) = map.get_mut(id) {
                conn.peer_conn = Some(peer_connection.clone());
            }
        }

        let offer = peer_connection
            .create_offer(None)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        peer_connection
            .set_local_description(offer.clone())
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        let preview_offer = PreviewOffer {
            id: id.to_string(),
            sdp: offer.sdp,
        };

        Ok(preview_offer)
    }

    pub async fn accept_preview_answer(&self, id: &str, sdp: String) -> Result<()> {
        // Lock the map to get the connection
        let peer_connection = {
            let map = self.preview_connections.lock().await;
            let connection = map
                .get(id)
                .ok_or_else(|| CaptureError::SourceNotFound(id.to_string()))?;

            // Clone the Arc so we can drop the lock before awaiting
            connection
                .peer_conn
                .as_ref()
                .ok_or_else(|| CaptureError::SourceNotFound(id.to_string()))?
                .clone()
        }; // lock dropped here

        let answer = RTCSessionDescription::answer(sdp)
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        peer_connection
            .set_remote_description(answer)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        Ok(())
    }

    pub async fn close_preview(&self, id: &str) {
        let connection = {
            let mut map = self.preview_connections.lock().await;
            map.remove(id)
        };

        if let Some(connection) = connection {
            if let Some(pc) = connection.peer_conn {
                let _ = pc.close().await;
            }
        }
    }

    pub async fn close_all_previews(&self) {
        let connections: Vec<PreviewConnection> = {
            let mut map = self.preview_connections.lock().await;
            map.drain().map(|(_, conn)| conn).collect()
        };

        for connection in connections {
            if let Some(pc) = connection.peer_conn {
                let _ = pc.close().await;
            }
        }
    }

    pub async fn add_preview_ice_candidate(
        &self,
        id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<()> {
        let pc = {
            let map = self.preview_connections.lock().await;
            map.get(&id)
                .ok_or(CaptureError::SourceNotFound(id.clone()))?
                .peer_conn
                .as_ref()
                .ok_or(CaptureError::SourceNotFound(id.clone()))?
                .clone()
        };

        let ice_candidate = RTCIceCandidateInit {
            candidate,
            sdp_mid,
            sdp_mline_index,
            ..Default::default()
        };

        pc.add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        Ok(())
    }
}
