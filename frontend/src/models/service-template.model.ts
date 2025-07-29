import { ApplicationProtocol, TransportProtocol } from "./service.model";

export interface ServiceTemplate {
  kind: string;
  ports: ServicePortTemplate[];
}

export interface ServicePortTemplate {
  name: string;
  transportProtocol: TransportProtocol;
  applicationProtocol: ApplicationProtocol;
  port: number;
}
