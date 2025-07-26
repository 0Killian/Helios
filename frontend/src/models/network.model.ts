// TODO: Should be auto-generated
export interface NetworkStats {
  download: NetworkStatsItem;
  upload: NetworkStatsItem;
  active_sessions: number;
}

export interface NetworkStatsItem {
  max_bandwidth: number;
  current_bandwidth: number;
  total_since_last_reboot: number;
  packets_lost: number;
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
