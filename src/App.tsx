import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import Settings from "./Settings";
import Overlay from "./Overlay";

// Single-page React app: branch on the Tauri window label so the same bundle
// serves both the settings popup and the shake overlay without extra entries.
export default function App() {
  const label = getCurrentWebviewWindow().label;
  return label === "overlay" ? <Overlay /> : <Settings />;
}
