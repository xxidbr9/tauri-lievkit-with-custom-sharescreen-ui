import { useEffect } from "react";
import { toast } from "sonner";
import { listen } from "@tauri-apps/api/event";

export const useAppError = () => {
  useEffect(() => {
    const handleWindowError = (e: ErrorEvent) => {
      console.error("handleWindowError: ", e);
      toast.error("Proccess Error!", {
        description: e.message,
      });
    };

    const handlePromiseRejection = (e: PromiseRejectionEvent) => {
      console.error("handlePromiseRejection: ", e);
      const msg =
        typeof e.reason === "string"
          ? e.reason
          : e.reason?.message || "Unhandled Promise Rejection";

      toast.warning("App Error!", {
        description: msg,
      });
    };

    window.addEventListener("error", handleWindowError);
    window.addEventListener("unhandledrejection", handlePromiseRejection);

    return () => {
      window.removeEventListener("error", handleWindowError);
      window.removeEventListener("unhandledrejection", handlePromiseRejection);
    };
  }, []);

  useEffect(() => {
    const unlistenPromise = listen<string>("rust-panic", (e) => {
      console.error("Rust panic:", e.payload);
      toast.error("App Panic!", { description: e.payload });
    });

    return () => {
      unlistenPromise.then((u) => u());
    };
  }, []);

  // useEffect(() => {
  //   window.electron.onGlobalError((msg: string) => {
  //     console.error("window.electron.onGlobalError: ", msg);
  //     toast("Electron Error", {
  //       description: msg,
  //       icon: <AlertTriangle className="text-red-700" />,
  //     });
  //   });
  // }, []);
};
