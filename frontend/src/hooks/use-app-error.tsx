import { useEffect } from "react";
import { toast } from "sonner";
import { AlertTriangle } from "lucide-react";

export const useAppError = () => {
  useEffect(() => {
    const handleWindowError = (e: ErrorEvent) => {
      console.error("handleWindowError: ", e);
      toast("Renderer Error", {
        description: e.message,
        icon: <AlertTriangle className="text-red-500" />,
      });
    };

    const handlePromiseRejection = (e: PromiseRejectionEvent) => {
      console.error("handlePromiseRejection: ", e);
      const msg =
        typeof e.reason === "string"
          ? e.reason
          : e.reason?.message || "Unhandled Promise Rejection";

      toast("Unhandled Promise", {
        description: msg,
        icon: <AlertTriangle className="text-orange-500" />,
      });
    };

    window.addEventListener("error", handleWindowError);
    window.addEventListener("unhandledrejection", handlePromiseRejection);

    return () => {
      window.removeEventListener("error", handleWindowError);
      window.removeEventListener("unhandledrejection", handlePromiseRejection);
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
