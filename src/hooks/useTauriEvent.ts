import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

/**
 * Subscribe to a Tauri event with automatic cleanup on unmount.
 * The callback is called with the event payload each time the event fires.
 */
export function useTauriEvent<T>(
  eventName: string,
  callback: (payload: T) => void,
) {
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<T>(eventName, (event) => {
      callback(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [eventName]);
}
