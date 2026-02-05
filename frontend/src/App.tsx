import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useVh } from "./hooks/use-vh";
import { useEffect } from "react";

function App() {
  useVh();

  const handleShowMonitors = async () => {
    try {
      const monitors = await invoke("get_list");
      console.log(monitors);
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="full-height">
      {/*<Decorations />*/}
      <main className="container">
        <h1>Welcome to Tauri + React</h1>

        <button onClick={handleShowMonitors}>Hello world</button>
      </main>
    </div>
  );
}

export default App;

const Decorations = () => {
  // const minimizeButtonRef = useRef<HTMLButtonElement>(null);
  // const maximizeButtonRef = useRef<HTMLButtonElement>(null);
  // const closeButtonRef = useRef<HTMLButtonElement>(null);
  const appWindow = getCurrentWindow();
  const handleMinimize = () => {
    appWindow.minimize();
  };

  const handleMaximize = () => {
    appWindow.toggleMaximize();
  };

  const handleClose = () => {
    appWindow.close();
  };

  return (
    <div className="titlebar">
      <div data-tauri-drag-region></div>
      <div className="controls">
        <button
          onClick={handleMinimize}
          id="titlebar-minimize"
          title="minimize"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
          >
            <path fill="currentColor" d="M19 13H5v-2h14z" />
          </svg>
        </button>
        <button
          onClick={handleMaximize}
          id="titlebar-maximize"
          title="maximize"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
          >
            <path fill="currentColor" d="M4 4h16v16H4zm2 4v10h12V8z" />
          </svg>
        </button>
        <button onClick={handleClose} id="titlebar-close" title="close">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
          >
            <path
              fill="currentColor"
              d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z"
            />
          </svg>
        </button>
      </div>
    </div>
  );
};
