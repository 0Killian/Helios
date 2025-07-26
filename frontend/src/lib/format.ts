/**
 * Converts a number of bytes into a human-readable string with units (KB, MB, GB, etc.).
 * @param bytes The number of bytes.
 * @param decimals The number of decimal places to display.
 * @returns A formatted string like "1.23 GB".
 */
export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

/**
 * Converts a speed in Kilobits per second (kbps) into a human-readable string with units (Kbps, Mbps, Gbps, etc.).
 * @param kbps The speed in kbps.
 * @param decimals The number of decimal places to display.
 * @returns A formatted string like "1.23 Mbps".
 */
export function formatBandwidth(kbps: number, decimals = 1): string {
  if (kbps === 0) return "0 Kbps";

  const k = 1000;
  const dm = decimals < 0 ? 0 : decimals;
  const units = [
    "Kbps",
    "Mbps",
    "Gbps",
    "Tbps",
    "Pbps",
    "Ebps",
    "Zbps",
    "Ybps",
  ];

  const i = Math.floor(Math.log(kbps) / Math.log(k));

  return `${parseFloat((kbps / Math.pow(k, i)).toFixed(dm))} ${units[i]}`;
}

/**
 * Converts a duration in seconds to a human-readable string (e.g., "7d 14h 32m").
 * Omits parts that are zero, and shows "0m" for durations less than a minute.
 * @param totalSeconds The total duration in seconds.
 * @returns A formatted string like "7d 14h 32m".
 */
export function formatDuration(totalSeconds: number): string {
  if (totalSeconds <= 0) {
    return "0m";
  }

  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);

  const parts: string[] = [];

  if (days > 0) {
    parts.push(`${days}d`);
  }
  if (hours > 0) {
    parts.push(`${hours}h`);
  }
  // Show minutes if it's non-zero, or if it's the only unit available
  if (minutes > 0 || parts.length === 0) {
    parts.push(`${minutes}m`);
  }

  return parts.join(" ");
}
