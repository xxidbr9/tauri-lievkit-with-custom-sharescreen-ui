import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export const useDebugStreamFps = () => {
  const [fps, setFps] = useState(0);

  useEffect(() => {
    const unlisten = listen<number>("debug-stream-fps", (data) => {
      const newFPS = data.payload;
      console.log(`FPS: ${newFPS}`);
      setFps(newFPS);
    });

    return () => {
      unlisten.then(() => {});
    };
  }, []);

  return fps;
};
