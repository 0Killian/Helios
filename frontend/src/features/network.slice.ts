import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { NetworkWan } from "@/models";
import { apiClient, ApiError } from "@/api/apiClient";
import { SliceError } from ".";

interface NetworkState {
  wan: NetworkWan | null;
  status: "idle" | "loading" | "succeeded" | "failed";
  error: SliceError | null;
}

const initialState: NetworkState = {
  wan: null,
  status: "idle",
  error: null,
};

/**
 * Fetch current network WAN informations
 */
export const fetchNetworkWanInfo = createAsyncThunk<
  NetworkWan,
  void,
  { rejectValue: SliceError }
>("network/fetchNetworkWanInfo", async (_, thunkAPI) => {
  try {
    return await apiClient<NetworkWan>("/api/v1/network");
  } catch (error) {
    if (error instanceof ApiError) {
      return thunkAPI.rejectWithValue({
        code: error.code,
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
export const networkSlice = createSlice({
  name: "network",
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchNetworkWanInfo.pending, (state) => {
        state.status = "loading";
      })
      .addCase(fetchNetworkWanInfo.fulfilled, (state, action) => {
        state.wan = action.payload;
        state.status = "succeeded";
      })
      .addCase(fetchNetworkWanInfo.rejected, (state, action) => {
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
