import { Event, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { useSyncExternalStore, useRef, useCallback } from "react";

interface rinnaReply {
  type: "Reply";
  id: number;
  response: string;
}

interface rinnaEnd {
  type: "End";
  id: number;
}

type rinnaEvent = rinnaReply | rinnaEnd;

export function queryRinna(id: number, prompt: string): string {
  const rinnnaResponseAccum = useRef<string>("");
  const callable_id = useRef<number>(-1);
  const unlisten = useRef<(() => void) | undefined>(undefined);

  const subscribe = useCallback(
    (callback: () => void): (() => void) => {
      (async () => {
        if (id < callable_id.current) {
          return;
        }
        callable_id.current = id + 1;

        try {
          await invoke("rinna", { id, prompt });
        } catch (e) {
          console.error(e);
          return;
        }

        const unlsn = await listen(
          "rinna-response",
          (event: Event<rinnaEvent>) => {
            const r = event.payload;

            console.log("rinna-response", r);

            if (r.id !== id) {
              return;
            }

            switch (r.type) {
              case "Reply":
                rinnnaResponseAccum.current += r.response;
                break;
              case "End":
                // </s>はつけ直す
                rinnnaResponseAccum.current =
                  rinnnaResponseAccum.current.replace(/<\/s>/g, "");
                rinnnaResponseAccum.current += "</s>";

                if (unlisten.current) {
                  unlisten.current();
                  unlisten.current = undefined;
                }
                break;
            }

            callback();
          }
        );

        unlisten.current = unlsn;
      })();

      return () => {
        if (unlisten.current) {
          unlisten.current();
          unlisten.current = undefined;
        }
      };
    },
    [id]
  );

  const response = useSyncExternalStore<string>(
    subscribe,
    () => rinnnaResponseAccum.current
  );

  return response;
}
