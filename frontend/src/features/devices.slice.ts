import { RootState } from "@/store";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { DeviceFull } from "@/models";

interface DevicesState {
  devices: DeviceFull[];
  status: "idle" | "loading" | "succeeded" | "failed";
  error: string | null;
}

const initialState: DevicesState = {
  devices: [],
  status: "idle",
  error: null,
};

/**
 * Fetch all network devices
 */
export const fetchDevices = createAsyncThunk<
  DeviceFull[],
  void,
  { state: RootState }
>("devices/fetchDevices", async () => {
  // TODO: BASE URL should be configurable
  const response = await fetch(
    "http://127.0.0.1:3000/api/v1/devices?full=true",
  );
  if (!response.ok) {
    throw new Error("Failed to fetch devices");
  }
  const data = await response.json();
  return data;
});

/**
 * Slice
 */
export const devicesSlice = createSlice({
  name: "devices",
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchDevices.pending, (state) => {
        state.status = "loading";
      })
      .addCase(fetchDevices.fulfilled, (state, action) => {
        state.devices = action.payload;
        state.status = "succeeded";
      })
      .addCase(fetchDevices.rejected, (state, action) => {
        state.status = "failed";
        state.error = action.error.message ?? "Unknown error";
      });
  },
});
