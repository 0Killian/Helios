export * from "./network.slice";
export * from "./devices.slice";
export * from "./service-templates.slice";
export * from "./services.slice";

export interface SliceError {
  message: string;
  code: string;
}
