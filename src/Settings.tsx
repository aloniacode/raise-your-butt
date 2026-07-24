import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ConfigDto {
  interval_min: number;
  autostart: boolean;
  intensity: number;
}

export default function Settings() {
  const [interval, setInterval] = useState<number>(30);
  const [intensity, setIntensity] = useState<number>(5);
  const [autostart, setAutostart] = useState<boolean>(false);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const cfg = await invoke<ConfigDto>("get_config");
        setInterval(cfg.interval_min);
        setIntensity(cfg.intensity);
        setAutostart(cfg.autostart);
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
    }).catch(() => {});
  }, [loaded, interval, intensity, autostart]);

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
        每 {interval} 分钟提醒一次，强度 {intensity} / 10
      </div>
    </div>
  );
}
