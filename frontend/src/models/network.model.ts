// TODO: Should be auto-generated
export interface NetworkStats {
  download: NetworkStatsItem;
  upload: NetworkStatsItem;
  activeSessions: number;
}

export interface NetworkStatsItem {
  maxBandwidth: number;
  currentBandwidth: number;
  totalSinceLastReboot: number;
  packetsLost: number;
}

export interface NetworkConnectivity {
  ipv4: string;
  ipv6: string;
  gateway: string;
  status: "Up" | "Down";
  uptime: number;
}

export interface NetworkWan {
  connectivity: NetworkConnectivity;
  stats: NetworkStats;
}
