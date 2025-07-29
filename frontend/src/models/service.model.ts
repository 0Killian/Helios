export interface Service {
  serviceId: string;
  deviceMac: string;
  displayName: string;
  kind: string;
  isManaged: boolean;
  ports: ServicePort[];
}

export interface ServicePort {
  port: number;
  transportProtocol: TransportProtocol;
  applicationProtocol: ApplicationProtocol;
  isOnline: boolean;
}

export type TransportProtocol = "TCP" | "UDP";
export type ApplicationProtocol = "HTTP";
