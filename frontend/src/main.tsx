import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@/index.css";
import Router from "@/router";
import { Provider } from "react-redux";
import { store } from "./store";
import { Toaster } from "./components/ui/Toaster";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider store={store}>
      <Toaster />
      <Router />
    </Provider>
  </StrictMode>,
);
