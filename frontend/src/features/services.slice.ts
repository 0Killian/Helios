import { RootState } from "@/store";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { ApplicationProtocol, Service, TransportProtocol } from "@/models";

export interface CreateService {
  deviceMac: string;
  displayName: string;
  kind: string;
  ports: CreateServicePort[];
}

export interface CreateServicePort {
  port: number;
  name: string;
  transportProtocol: TransportProtocol;
  applicationProtocol: ApplicationProtocol;
}

interface ServicesState {
  service: Service | null;
  status: "idle" | "loading" | "succeeded" | "failed";
  error: string | null;
}

const initialState: ServicesState = {
  service: null,
  status: "idle",
  error: null,
};

/**
 * Create a new service
 */
export const createService = createAsyncThunk<
  Service,
  CreateService,
  { state: RootState }
>("services/createService", async (payload) => {
  // TODO: BASE URL should be configurable
  const response = await fetch("http://127.0.0.1:3000/api/v1/services", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    throw new Error("Failed to create service");
  }
  const data = await response.json();
  return data;
});

/**
 * Slice
 */
export const servicesSlice = createSlice({
  name: "services",
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(createService.pending, (state) => {
        state.status = "loading";
      })
      .addCase(createService.fulfilled, (state, action) => {
        state.service = action.payload;
        state.status = "succeeded";
      })
      .addCase(createService.rejected, (state, action) => {
        state.status = "failed";
        state.error = action.error.message ?? "Unknown error";
      });
  },
});
