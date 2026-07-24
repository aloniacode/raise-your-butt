import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export default function Overlay() {
  const [tick, setTick] = useState(0);

  useEffect(() => {
    const unlistenP = listen<number>("shake-start", () => {
      // remount the card so the keyframes restart each shake
      setTick((t) => t + 1);
    });
    return () => {
      unlistenP.then((u) => u()).catch(() => {});
    };
  }, []);

  return (
    <div className="overlay-bg">
      <div className="card shake-anim" key={tick}>
        <div className="emoji">🧍</div>
        <h2>该起身活动啦！</h2>
        <p>久坐伤身，起来走走吧</p>
      </div>
    </div>
  );
}
