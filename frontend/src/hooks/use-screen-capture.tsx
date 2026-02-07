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
        // Create peer connection
        const offer = await invoke<PreviewOffer>("get_preview_offer", { id });

        const pc = new RTCPeerConnection();

        pc.ontrack = (event) => {
          // should return at least 1 track
          if (event.streams[0]) {
            videoElement.srcObject = event.streams[0];
          }
        };

        pc.onicecandidate = (event) => {
          if (event.candidate) {
            console.log("ICE candidate:", event.candidate);
          }
        };

        pc.onconnectionstatechange = () => {
          console.log(`Connection state for ${id}:`, pc.connectionState);
        };

        // Get offer from backend

        await pc.setRemoteDescription({ type: "offer", sdp: offer.sdp });
        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);

        // Send answer to backend
        await invoke("accept_preview_answer", { id, sdp: answer.sdp });

        // Store peer connection
        peerConnectionsRef.current.set(id, pc);

        return pc;
      } catch (err) {
        console.error("Failed to setup preview connection:", err);
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
