import { configureStore } from "@reduxjs/toolkit";

import { networkSlice, devicesSlice } from "@/features";

export const store = configureStore({
  reducer: {
    [networkSlice.name]: networkSlice.reducer,
    [devicesSlice.name]: devicesSlice.reducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
