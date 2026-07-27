import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Icon } from "@iconify/react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTheme, type Theme } from "@/hooks/use-theme";

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

  // Theme: light / dark / system. Defaults to "system" (follows the OS).
  const { theme, setTheme } = useTheme();

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
    // Root background adapts to the active theme (zinc-950 in dark, white in
    // light). body stays transparent so the overlay window keeps its own
    // transparent backdrop.
    <div className="bg-background text-foreground flex h-full w-full flex-col gap-4 overflow-y-auto p-5">
      {/* Reminder interval */}
      <div className="flex flex-col gap-2">
        <Label>提醒间隔</Label>
        <div className="flex items-center gap-2">
          <Input
            className="flex-1"
            type="number"
            min={1}
            max={180}
            value={interval}
            onChange={(e) => {
              const v = parseInt(e.target.value, 10);
              if (!Number.isNaN(v)) setInterval(Math.min(180, Math.max(1, v)));
            }}
          />
          <span className="text-sm text-muted-foreground tabular-nums">
            分钟
          </span>
        </div>
      </div>

      {/* Shake intensity */}
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <Label>屏幕抖动强度</Label>
          <span className="text-sm font-medium tabular-nums">{intensity}</span>
        </div>
        <Slider
          min={1}
          max={10}
          step={1}
          value={[intensity]}
          onValueChange={(v) => setIntensity(v[0])}
        />
      </div>

      {/* Overlay dismiss mode */}
      <div className="flex flex-col gap-2">
        <Label>提醒窗口关闭方式</Label>
        <Tabs value={overlayMode} onValueChange={setOverlayMode}>
          <TabsList className="w-full">
            <TabsTrigger value="auto" className="flex-1">
              自动关闭
            </TabsTrigger>
            <TabsTrigger value="manual" className="flex-1">
              手动关闭
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {/* Overlay duration (auto mode only) */}
      {overlayMode === "auto" && (
        <div className="flex flex-col gap-2">
          <Label>提醒窗口显示时长（秒）</Label>
          <div className="flex items-center gap-2">
            <Input
              type="number"
              min={2}
              max={30}
              value={overlayDuration}
              onChange={(e) => {
                const v = parseInt(e.target.value, 10);
                if (!Number.isNaN(v))
                  setOverlayDuration(Math.min(30, Math.max(2, v)));
              }}
            />
            <span className="text-sm text-muted-foreground tabular-nums">
              秒
            </span>
          </div>
        </div>
      )}

      {/* Theme: light / dark / system — defaults to following the OS */}
      <div className="flex items-center justify-between">
        <Label>外观</Label>
        <Tabs value={theme} onValueChange={(v) => setTheme(v as Theme)}>
          <TabsList className="h-10">
            <TabsTrigger
              value="light"
              className="size-8 p-0"
              aria-label="浅色"
            >
              <Icon icon="tabler:sun" className="size-4" />
            </TabsTrigger>
            <TabsTrigger
              value="dark"
              className="size-8 p-0"
              aria-label="深色"
            >
              <Icon icon="tabler:moon" className="size-4" />
            </TabsTrigger>
            <TabsTrigger
              value="system"
              className="size-8 p-0"
              aria-label="跟随系统"
            >
              <Icon icon="tabler:device-desktop" className="size-4" />
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {/* Autostart */}
      <div className="flex items-center justify-between">
        <Label>开机自启动</Label>
        <Switch checked={autostart} onCheckedChange={setAutostart} />
      </div>

      {/* Test button — pinned to the bottom of the column */}
      <Button className="mt-auto" onClick={testShake}>
        <Icon icon="tabler:device-mobile-vibrate" className="size-4" />
        测试抖动
      </Button>

      {/* Status summary */}
      <p className="text-center text-xs text-muted-foreground tabular-nums">
        每 {interval} 分钟提醒一次，强度 {intensity} / 10，窗口
        {overlayMode === "auto"
          ? `自动关闭（${overlayDuration} 秒）`
          : "手动关闭"}
      </p>
    </div>
  );
}
