import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./lib/theme-provider";
import { TooltipProvider } from "./components/ui/tooltip";
import "./index.css";

// Prevent flash of incorrect theme on initial load
const initTheme = () => {
  const stored = localStorage.getItem("mantra-theme");
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  
  if (stored === "dark" || (stored === "system" && prefersDark) || (!stored && prefersDark)) {
    document.documentElement.classList.add("dark");
  } else if (stored === "light" || (stored === "system" && !prefersDark)) {
    document.documentElement.classList.remove("dark");
  }
};

// Run before React hydration
initTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="system">
      <TooltipProvider delayDuration={300}>
        <App />
      </TooltipProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
