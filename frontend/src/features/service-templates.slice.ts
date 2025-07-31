import { apiClient, ApiError } from "@/api/apiClient";
import { ServiceTemplate } from "@/models";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { SliceError } from ".";

interface ServiceTemplatesState {
  templates: ServiceTemplate[];
  status: "idle" | "loading" | "succeeded" | "failed";
  error: SliceError | null;
}

const initialState: ServiceTemplatesState = {
  templates: [],
  status: "idle",
  error: null,
};

export const fetchServiceTemplates = createAsyncThunk<
  ServiceTemplate[],
  void,
  { rejectValue: SliceError }
>("serviceTemplates/fetchServiceTemplates", async (_, thunkAPI) => {
  try {
    return await apiClient<ServiceTemplate[]>("/api/v1/service-templates");
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

export const serviceTemplatesSlice = createSlice({
  name: "serviceTemplates",
  initialState,
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchServiceTemplates.pending, (state) => {
        state.status = "loading";
      })
      .addCase(fetchServiceTemplates.fulfilled, (state, action) => {
        state.templates = action.payload;
        state.status = "succeeded";
      })
      .addCase(fetchServiceTemplates.rejected, (state, action) => {
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
