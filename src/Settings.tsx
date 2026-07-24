import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ConfigDto {
  interval_min: number;
  autostart: boolean;
  intensity: number;
  overlay_mode: string; // "auto" | "manual"
  overlay_duration_sec: number;
}

export default function Settings() {
  const [interval, setInterval] = useState<number>(30);
  const [intensity, setIntensity] = useState<number>(5);
  const [autostart, setAutostart] = useState<boolean>(false);
  const [overlayMode, setOverlayMode] = useState<string>("auto");
  const [overlayDuration, setOverlayDuration] = useState<number>(5);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const cfg = await invoke<ConfigDto>("get_config");
        setInterval(cfg.interval_min);
        setIntensity(cfg.intensity);
        setAutostart(cfg.autostart);
        setOverlayMode(cfg.overlay_mode);
        setOverlayDuration(cfg.overlay_duration_sec);
      } finally {
        setLoaded(true);
      }
    })();
  }, []);

  // Push updates back to Rust (which persists and rebuilds the timer).
  // We only fire set_config once the initial load is done to avoid wiping
  // the store with default values on first render.
  useEffect(() => {
    if (!loaded) return;
    invoke("set_config", {
      intervalMin: interval,
      autostart,
      intensity,
      overlayMode,
      overlayDurationSec: overlayDuration,
    }).catch(() => {});
  }, [loaded, interval, intensity, autostart, overlayMode, overlayDuration]);

  const testShake = () => {
    invoke("test_shake", { intensity }).catch(() => {});
  };

  return (
    <div className="settings">
      <div className="settings-header">
        <h1>久坐提醒</h1>
        <div className="subtitle">设置项会自动保存</div>
      </div>

      <div className="field">
        <div className="field-label">提醒间隔（分钟）</div>
        <div className="field-control">
          <input
            type="number"
            min={1}
            max={180}
            value={interval}
            onChange={(e) => {
              const v = parseInt(e.target.value, 10);
              if (!Number.isNaN(v)) setInterval(Math.min(180, Math.max(1, v)));
            }}
          />
          <span className="field-value">分钟</span>
        </div>
      </div>

      <div className="field">
        <div className="field-label">屏幕抖动强度</div>
        <div className="field-control">
          <input
            type="range"
            min={1}
            max={10}
            step={1}
            value={intensity}
            onChange={(e) => setIntensity(parseInt(e.target.value, 10))}
          />
          <span className="field-value">{intensity}</span>
        </div>
      </div>

      <div className="field">
        <div className="field-label">提醒窗口关闭方式</div>
        <div className="segmented">
          <button
            className={`seg ${overlayMode === "auto" ? "active" : ""}`}
            onClick={() => setOverlayMode("auto")}
          >
            自动关闭
          </button>
          <button
            className={`seg ${overlayMode === "manual" ? "active" : ""}`}
            onClick={() => setOverlayMode("manual")}
          >
            手动关闭
          </button>
        </div>
      </div>

      {overlayMode === "auto" && (
        <div className="field">
          <div className="field-label">提醒窗口显示时长（秒）</div>
          <div className="field-control">
            <input
              type="number"
              min={2}
              max={30}
              value={overlayDuration}
              onChange={(e) => {
                const v = parseInt(e.target.value, 10);
                if (!Number.isNaN(v)) setOverlayDuration(Math.min(30, Math.max(2, v)));
              }}
            />
            <span className="field-value">秒</span>
          </div>
        </div>
      )}

      <div className="field toggle-row">
        <div className="field-label" style={{ alignSelf: "center" }}>
          开机自启动
        </div>
        <label className="switch">
          <input
            type="checkbox"
            checked={autostart}
            onChange={(e) => setAutostart(e.target.checked)}
          />
          <span className="slider" />
        </label>
      </div>

      <button className="test" onClick={testShake}>
        测试抖动
      </button>

      <div className="status">
        每 {interval} 分钟提醒一次，强度 {intensity} / 10，窗口
        {overlayMode === "auto"
          ? `自动关闭（${overlayDuration} 秒）`
          : "手动关闭"}
      </div>
    </div>
  );
}
