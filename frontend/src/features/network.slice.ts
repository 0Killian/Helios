import { RootState } from "@/store";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { NetworkWan } from "@/models";

interface NetworkState {
  wan: NetworkWan | null;
  status: "idle" | "loading" | "succeeded" | "failed";
  error: string | null;
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
  { state: RootState }
>("network/fetchNetworkWanInfo", async () => {
  // TODO: BASE URL should be configurable
  const response = await fetch("http://127.0.0.1:3000/api/v1/network");
  if (!response.ok) {
    throw new Error("Failed to fetch network WAN info");
  }
  const data = await response.json();
  return data;
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
        state.error = action.error.message ?? "Unknown error";
      });
  },
});
