import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface ShakeStartPayload {
  intensity: number;
  manual: boolean;
}

export default function Overlay() {
  const [tick, setTick] = useState(0);
  const [manual, setManual] = useState(false);

  useEffect(() => {
    const unlistenP = listen<ShakeStartPayload>("shake-start", (e) => {
      // `manual` decides whether we render a close button; the card remounts
      // via `key` so the CSS shake keyframes restart on every shake.
      setManual(e.payload.manual);
      setTick((t) => t + 1);
    });
    return () => {
      unlistenP.then((u) => u()).catch(() => {});
    };
  }, []);

  const close = () => {
    invoke("close_overlay").catch(() => {});
  };

  return (
    <div className="overlay-bg">
      <div className="card shake-anim" key={tick}>
        <div className="emoji">🧍</div>
        <h2>该起身活动啦！</h2>
        <p>久坐伤身，起来走走吧</p>
        {manual && (
          <button className="close-btn" onClick={close}>
            我知道了
          </button>
        )}
      </div>
    </div>
  );
}
