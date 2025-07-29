import { configureStore } from "@reduxjs/toolkit";

import {
  networkSlice,
  devicesSlice,
  serviceTemplatesSlice,
  servicesSlice,
} from "@/features";

export const store = configureStore({
  reducer: {
    [networkSlice.name]: networkSlice.reducer,
    [devicesSlice.name]: devicesSlice.reducer,
    [serviceTemplatesSlice.name]: serviceTemplatesSlice.reducer,
    [servicesSlice.name]: servicesSlice.reducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
