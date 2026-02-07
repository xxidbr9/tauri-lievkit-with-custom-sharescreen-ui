// src/components/LivePreviewSelector.tsx (Alternative with live video immediately)
import { useEffect, useRef, useState } from "react";
import {
  useScreenCapture,
  // type CaptureSource,
} from "@/hooks/use-screen-capture";

export const DummiesLayout = () => {
  const {
    monitors,
    windows,
    loading,
    fetchMonitors,
    fetchWindows,
    startMonitorPreview,
    startWindowPreview,
    setupPreviewConnection,
    // stopPreview,
  } = useScreenCapture();

  const [activeTab, setActiveTab] = useState<"monitors" | "windows">(
    "monitors",
  );
  const [selectedSource, setSelectedSource] = useState<string | null>(null);
  const videoRefs = useRef<Map<string, HTMLVideoElement>>(new Map());
  const [initializedPreviews, setInitializedPreviews] = useState<Set<string>>(
    new Set(),
  );

  // Initialize all previews on load
  useEffect(() => {
    const initPreviews = async () => {
      await fetchMonitors(10, 320, 180);
      await fetchWindows(10, 320, 180);
    };

    initPreviews();
  }, [fetchMonitors, fetchWindows]);

  // Auto-start previews for all sources
  useEffect(() => {
    if (!monitors) return;
    if (!monitors.length) return;

    const sources = [monitors[0]];
    sources.forEach(async (source) => {
      if (initializedPreviews.has(source.id)) return;

      try {
        const handle = parseInt(source.id.split("_")[1]);

        // Start preview
        if (source.source_type === "monitor") {
          await startMonitorPreview(handle, 10, 320, 180);
        } else {
          await startWindowPreview(handle, 10, 320, 180);
        }

        // Wait for stream
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Setup WebRTC
        const videoElement = videoRefs.current.get(source.id);

        if (videoElement) {
          await setupPreviewConnection(source.id, videoElement);
          setInitializedPreviews((prev) => new Set(prev).add(source.id));
        }
      } catch (err) {
        console.error("Failed to initialize preview:", err);
      }
    });
  }, [
    monitors,
    windows,
    setupPreviewConnection,
    initializedPreviews,
    startMonitorPreview,
    startWindowPreview,
  ]);

  const sources = activeTab === "monitors" ? monitors : windows;

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-3xl font-bold mb-8">Select Screen to Share</h1>

        <div className="flex gap-4 mb-8">
          <button
            onClick={() => setActiveTab("monitors")}
            className={`px-6 py-2 rounded-lg font-medium transition ${
              activeTab === "monitors"
                ? "bg-blue-600 text-white"
                : "bg-gray-800 text-gray-300 hover:bg-gray-700"
            }`}
          >
            Monitors ({monitors.length})
          </button>
          <button
            onClick={() => setActiveTab("windows")}
            className={`px-6 py-2 rounded-lg font-medium transition ${
              activeTab === "windows"
                ? "bg-blue-600 text-white"
                : "bg-gray-800 text-gray-300 hover:bg-gray-700"
            }`}
          >
            Windows ({windows.length})
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
          </div>
        ) : (
          <div className="grid grid-cols-3 gap-4">
            {sources.map((source) => (
              <div
                key={source.id}
                onClick={() => setSelectedSource(source.id)}
                className={`cursor-pointer border-2 rounded-lg overflow-hidden transition-all ${
                  selectedSource === source.id
                    ? "border-blue-500 shadow-lg shadow-blue-500/50 scale-105"
                    : "border-gray-700 hover:border-gray-500"
                }`}
              >
                <div className="aspect-video bg-gray-800 relative">
                  <video
                    ref={(el) => {
                      if (el) videoRefs.current.set(source.id, el);
                    }}
                    autoPlay
                    playsInline
                    muted
                    className="w-full h-full object-contain"
                  />

                  <div className="absolute top-2 right-2 bg-red-600 text-white px-2 py-1 rounded text-xs font-bold flex items-center gap-1">
                    <div className="w-2 h-2 bg-white rounded-full animate-pulse"></div>
                    LIVE
                  </div>

                  {selectedSource === source.id && (
                    <div className="absolute inset-0 border-4 border-blue-500 pointer-events-none"></div>
                  )}
                </div>

                <div className="p-3 bg-gray-800">
                  <p className="text-sm font-medium truncate">{source.title}</p>
                  <p className="text-xs text-gray-400">
                    {source.width} × {source.height}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
