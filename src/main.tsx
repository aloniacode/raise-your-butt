import React from "react";
import ReactDOM from "react-dom/client";
import { addCollection } from "@iconify/react";
import tabler from "@iconify-json/tabler/icons.json";

import App from "./App";
import "./styles.css";

// Register the tabler icon set offline so <Icon icon="tabler:..." /> never
// hits the Iconify HTTP API at runtime (important for a desktop app).
addCollection(tabler);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
