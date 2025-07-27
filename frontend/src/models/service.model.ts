export interface Service {
  serviceId: string;
  deviceMac: string;
  displayName: string;
  kind: ServiceKind;
  isManaged: boolean;
  ports: ServicePort[];
}

export interface ServicePort {
  port: number;
  transportProtocol: TransportProtocol;
  applicationProtocol: ApplicationProtocol;
  isOnline: boolean;
}

type ServiceKind = "hello-world";
type TransportProtocol = "TCP" | "UDP";
type ApplicationProtocol = "HTTP";
