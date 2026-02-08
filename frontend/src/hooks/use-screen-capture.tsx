// src/hooks/useScreenCapture.ts
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef, useCallback } from "react";

export interface CaptureSource {
  id: string;
  title: string;
  thumbnail: string;
  icon: string | null;
  source_type: "monitor" | "window";
  width: number;
  height: number;
}

export interface PreviewOffer {
  id: string;
  sdp: string;
}

export const useScreenCapture = () => {
  const [monitors, setMonitors] = useState<CaptureSource[]>([]);
  const [windows, setWindows] = useState<CaptureSource[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const peerConnectionsRef = useRef<Map<string, RTCPeerConnection>>(new Map());

  // Fetch monitors with preview
  const fetchMonitors = useCallback(
    async (fps: number = 10, width: number = 320, height: number = 180) => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<CaptureSource[]>("get_monitors", {
          fps,
          width,
          height,
        });
        setMonitors(result);
      } catch (err) {
        setError(err as string);
        console.error("Failed to fetch monitors:", err);
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  // Fetch windows with preview
  const fetchWindows = useCallback(
    async (fps: number = 10, width: number = 320, height: number = 180) => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<CaptureSource[]>("get_windows", {
          fps,
          width,
          height,
        });
        setWindows(result);
      } catch (err) {
        setError(err as string);
        console.error("Failed to fetch windows:", err);
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  // Get specific monitor
  const getMonitorById = useCallback(
    async (
      id: string,
      fps: number = 10,
      width: number = 320,
      height: number = 180,
    ) => {
      try {
        return await invoke<CaptureSource>("get_monitor_by_id", {
          id,
          fps,
          width,
          height,
        });
      } catch (err) {
        console.error("Failed to get monitor:", err);
        throw err;
      }
    },
    [],
  );

  // Get specific window
  const getWindowById = useCallback(
    async (
      id: string,
      fps: number = 10,
      width: number = 320,
      height: number = 180,
    ) => {
      try {
        return await invoke<CaptureSource>("get_window_by_id", {
          id,
          fps,
          width,
          height,
        });
      } catch (err) {
        console.error("Failed to get window:", err);
        throw err;
      }
    },
    [],
  );

  // Start monitor preview stream
  const startMonitorPreview = useCallback(
    async (
      hmonitor: number,
      fps: number = 10,
      width: number = 320,
      height: number = 180,
    ) => {
      try {
        await invoke("start_monitor_preview", { hmonitor, fps, width, height });
      } catch (err) {
        console.error("Failed to start monitor preview:", err);
        throw err;
      }
    },
    [],
  );

  // Start window preview stream
  const startWindowPreview = useCallback(
    async (
      hwnd: number,
      fps: number = 10,
      width: number = 320,
      height: number = 180,
    ) => {
      try {
        await invoke("start_window_preview", { hwnd, fps, width, height });
      } catch (err) {
        console.error("Failed to start window preview:", err);
        throw err;
      }
    },
    [],
  );

  // Stop preview stream
  const stopPreview = useCallback(async (id: string) => {
    try {
      await invoke("stop_preview", { id });

      // Close WebRTC connection
      const pc = peerConnectionsRef.current.get(id);
      if (pc) {
        pc.close();
        peerConnectionsRef.current.delete(id);
      }
    } catch (err) {
      console.error("Failed to stop preview:", err);
      throw err;
    }
  }, []);

  // Setup WebRTC connection for preview
  const setupPreviewConnection = useCallback(
    async (id: string, videoElement: HTMLVideoElement) => {
      try {
        console.log(`[WebRTC] Setting up connection for ${id}`);

        // Create peer connection
        const pc = new RTCPeerConnection({
          iceServers: [
            {
              urls: ["stun:stun.l.google.com:19302"],
            },
          ],
        });

        // Debug: Log all events
        pc.ontrack = (event) => {
          console.log({ event });
          console.log(`[WebRTC] Track received for ${id}:`, {
            kind: event.track.kind,
            id: event.track.id,
            readyState: event.track.readyState,
            muted: event.track.muted,
            enabled: event.track.enabled,
            streams: event.streams.length,
          });

          if (event.streams[0]) {
            console.log(
              `[WebRTC] Setting srcObject for ${id}`,
              event.streams[0],
            );
            // videoElement.srcObject = null;
            // videoElement.removeAttribute("srcObject");
            // videoElement.src = "https://www.w3schools.com/html/mov_bbb.mp4";
            videoElement.srcObject = event.streams[0];

            // Force video to play
            videoElement
              .play()
              .then(() => {
                console.log(`[WebRTC] Video playing for ${id}`);
              })
              .catch((err) => {
                console.error(`[WebRTC] Failed to play video for ${id}:`, err);
              });

            console.log(`[WebRTC] Video element for ${id}:`, videoElement);
          } else {
            console.warn(`[WebRTC] No stream in track event for ${id}`);
          }
        };

        pc.onicecandidate = async (event) => {
          if (event.candidate) {
            console.log(
              `[WebRTC] ICE candidate for ${id}:`,
              event.candidate.candidate,
            );
            await invoke("add_preview_ice_candidate", {
              id,
              candidate: event.candidate.candidate,
              sdpMid: event.candidate.sdpMid,
              sdpMLineIndex: event.candidate.sdpMLineIndex,
            });
          } else {
            console.log(`[WebRTC] ICE gathering complete for ${id}`);
          }
        };

        pc.oniceconnectionstatechange = () => {
          console.log(
            `[WebRTC] ICE connection state for ${id}:`,
            pc.iceConnectionState,
          );
        };

        pc.onconnectionstatechange = () => {
          console.log(
            `[WebRTC] Connection state for ${id}:`,
            pc.connectionState,
          );
        };

        pc.onsignalingstatechange = () => {
          console.log(`[WebRTC] Signaling state for ${id}:`, pc.signalingState);
        };

        pc.onicegatheringstatechange = () => {
          console.log(
            `[WebRTC] ICE gathering state for ${id}:`,
            pc.iceGatheringState,
          );
        };

        // Get offer from backend
        console.log(`[WebRTC] Getting offer from backend for ${id}`);
        const offer = await invoke<PreviewOffer>("get_preview_offer", { id });
        console.log(
          `[WebRTC] Received offer for ${id}:`,
          offer.sdp.substring(0, 100) + "...",
        );

        // Set remote description
        console.log(`[WebRTC] Setting remote description for ${id}`);
        await pc.setRemoteDescription({ type: "offer", sdp: offer.sdp });

        // Create answer
        console.log(`[WebRTC] Creating answer for ${id}`);
        const answer = await pc.createAnswer();
        console.log(
          `[WebRTC] Answer created for ${id}:`,
          answer.sdp?.substring(0, 100) + "...",
        );

        await pc.setLocalDescription(answer);
        console.log(`[WebRTC] Local description set for ${id}`);

        // Send answer to backend
        console.log(`[WebRTC] Sending answer to backend for ${id}`);
        await invoke("accept_preview_answer", { id, sdp: answer.sdp });
        console.log(`[WebRTC] Answer accepted by backend for ${id}`);

        // Store peer connection
        peerConnectionsRef.current.set(id, pc);

        // Wait a bit and check video element
        setTimeout(() => {
          console.log(`[WebRTC] Video element state for ${id}:`, {
            srcObject: videoElement.srcObject,
            readyState: videoElement.readyState,
            paused: videoElement.paused,
            videoWidth: videoElement.videoWidth,
            videoHeight: videoElement.videoHeight,
          });
        }, 2000);

        const stats = await pc.getStats();

        stats.forEach((r) => {
          if (r.type === "inbound-rtp" && r.kind === "video") {
            console.log("IN video:", {
              packetsReceived: r.packetsReceived,
              packetsLost: r.packetsLost,
              framesDecoded: r.framesDecoded,
              keyFramesDecoded: r.keyFramesDecoded,
              frameWidth: r.frameWidth,
              frameHeight: r.frameHeight,
              bytesReceived: r.bytesReceived,
            });
          }

          if (r.type === "outbound-rtp" && r.kind === "video") {
            console.log("OUT video:", {
              packetsSent: r.packetsSent,
              framesEncoded: r.framesEncoded,
              bytesSent: r.bytesSent,
            });
          }

          if (r.type === "candidate-pair" && r.state === "succeeded") {
            console.log("ICE selected pair:", {
              currentRoundTripTime: r.currentRoundTripTime,
              availableOutgoingBitrate: r.availableOutgoingBitrate,
            });
          }
        });

        return pc;
      } catch (err) {
        console.error(
          `[WebRTC] Failed to setup preview connection for ${id}:`,
          err,
        );
        throw err;
      }
    },
    [],
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      peerConnectionsRef.current.forEach((pc) => pc.close());
      peerConnectionsRef.current.clear();
    };
  }, []);

  return {
    monitors,
    windows,
    loading,
    error,
    fetchMonitors,
    fetchWindows,
    getMonitorById,
    getWindowById,
    setupPreviewConnection,
    startMonitorPreview,
    startWindowPreview,
    stopPreview,
  };
};
