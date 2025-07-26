import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@/index.css";
import Router from "@/router";
import { Provider } from "react-redux";
import { store } from "./store";
import "react-placeholder/lib/reactPlaceholder.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider store={store}>
      <Router />
    </Provider>
  </StrictMode>,
);
