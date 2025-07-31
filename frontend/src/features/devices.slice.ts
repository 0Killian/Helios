import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { DeviceFull } from "@/models";
import { apiClient, ApiError } from "@/api/apiClient";
import { SliceError } from ".";

interface DevicesState {
  devices: DeviceFull[];
  status: "idle" | "loading" | "succeeded" | "failed";
  error: SliceError | null;
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
  { rejectValue: SliceError }
>("devices/fetchDevices", async (_, thunkAPI) => {
  try {
    return await apiClient<DeviceFull[]>("/api/v1/devices?full=true");
  } catch (error) {
    if (error instanceof ApiError) {
      return thunkAPI.rejectWithValue({
        code: error.code,
        message: error.message,
      });
    } else if (error instanceof Error) {
      return thunkAPI.rejectWithValue({
        code: "unknown-error",
        message: error.message,
      });
    }
    return thunkAPI.rejectWithValue({
      code: "unknown-error",
      message: "Unknown error",
    });
  }
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
        if (action.payload) {
          state.error = action.payload;
        } else {
          state.error = {
            code: "unknown-error",
            message: "Unknown error",
          };
        }
      });
  },
});
