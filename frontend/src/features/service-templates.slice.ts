import { ServiceTemplate } from "@/models";
import { RootState } from "@/store";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";

interface ServiceTemplatesState {
  templates: ServiceTemplate[];
  status: "idle" | "loading" | "succeeded" | "failed";
  error: string | null;
}

const initialState: ServiceTemplatesState = {
  templates: [],
  status: "idle",
  error: null,
};

export const fetchServiceTemplates = createAsyncThunk<
  ServiceTemplate[],
  void,
  { state: RootState }
>("serviceTemplates/fetchServiceTemplates", async () => {
  // TODO: BASE URL should be configurable
  const response = await fetch(
    "http://127.0.0.1:3000/api/v1/service-templates",
  );
  if (!response.ok) {
    throw new Error("Failed to fetch service templates");
  }
  const data = await response.json();
  return data;
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
        state.error = action.error.message ?? "Unknown error";
      });
  },
});
