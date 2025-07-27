import { Service } from "./service.model";

export interface Device {
  macAddress: string;
  lastKnownIp: string;
  displayName: string;
  isNameCustom: boolean;
  notes: string;
  isOnline: boolean;
  lastSeen: string;
  lastScanned: string;
}

export interface DeviceFull {
  device: Device;
  services: Service[];
}
