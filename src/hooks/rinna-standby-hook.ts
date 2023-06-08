import { useState, useEffect, useRef } from "react";
import { listen, Event } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

type RinnaStandbyRes = [boolean, (standby: boolean) => void];

export default function useRinnaStandby(): RinnaStandbyRes {
  const [standby, setStandby] = useState(false);
  const unlisten = useRef<(() => void) | undefined>(undefined);

  useEffect(() => {
    (async () => {
      if (unlisten.current) {
        return;
      }
      unlisten.current = () => {};

      await invoke("react_ready", {});

      console.log("react_ready invoked");

      const standby = await invoke<boolean>("is_rinna_standby", {});
      setStandby(standby);

      const unlsn = await listen("rinna-standby", (_event: Event<void>) => {
        setStandby(true);
      });

      unlisten.current = unlsn;
    })();

    return () => {
      if (unlisten.current) {
        unlisten.current();
        unlisten.current = undefined;
      }
    };
  }, []);

  return [standby, setStandby];
}
