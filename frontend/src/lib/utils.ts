import { clsx, type ClassValue } from "clsx";
import { toast } from "sonner";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export async function tauriSafe<T>(p: Promise<T>): Promise<T | null> {
  try {
    return await p;
  } catch (e: any) {
    console.error("Tauri runtime error:", e);
    toast.error("App Error", { description: e?.message ?? String(e) });
    return null;
  }
}
