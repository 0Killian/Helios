import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { ApplicationProtocol, Service, TransportProtocol } from "@/models";
import { apiClient, ApiError } from "@/api/apiClient";
import { SliceError } from ".";

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
  error: SliceError | null;
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
  { rejectValue: SliceError }
>("services/createService", async (payload, thunkAPI) => {
  // TODO: BASE URL should be configurable
  try {
    return await apiClient<Service>("/api/v1/services", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(payload),
    });
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
