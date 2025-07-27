import { Activity, AlertTriangle, Download, Upload } from "lucide-react";
import { Card } from "./ui/Card";
import { Progress } from "./ui/Progress";
import { Skeleton } from "./ui/Skeleton";
import { cn, formatBandwidth, formatDuration } from "@/lib";
import { NetworkWan } from "@/models";

export function NetworkStatsSkeleton() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {/* Download Skeleton */}
      <Card className="p-4 border-border bg-card">
        <div className="flex items-center gap-3 mb-3">
          <Skeleton className="h-8 w-8 rounded-lg" />
          <div className="space-y-2">
            <Skeleton className="h-4 w-[70px]" />
            <Skeleton className="h-3 w-[90px]" />
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Skeleton className="h-8 w-[80px]" />
            <Skeleton className="h-4 w-[90px]" />
          </div>
          <Skeleton className="h-2 w-full" />
          <Skeleton className="h-3 w-[120px] mt-1" />
        </div>
      </Card>

      {/* Upload Skeleton */}
      <Card className="p-4 border-border bg-card">
        <div className="flex items-center gap-3 mb-3">
          <Skeleton className="h-8 w-8 rounded-lg" />
          <div className="space-y-2">
            <Skeleton className="h-4 w-[60px]" />
            <Skeleton className="h-3 w-[90px]" />
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Skeleton className="h-8 w-[80px]" />
            <Skeleton className="h-4 w-[90px]" />
          </div>
          <Skeleton className="h-2 w-full" />
          <Skeleton className="h-3 w-[120px] mt-1" />
        </div>
      </Card>

      {/* Status Skeleton */}
      <Card className="p-4 border-border bg-card">
        <div className="flex items-center gap-3 mb-3">
          <Skeleton className="h-8 w-8 rounded-lg" />
          <div className="space-y-2">
            <Skeleton className="h-4 w-[100px]" />
            <Skeleton className="h-3 w-[80px]" />
          </div>
        </div>
        <div className="space-y-3 pt-1">
          <div className="flex items-center justify-between">
            <Skeleton className="h-4 w-[120px]" />
            <Skeleton className="h-6 w-[40px]" />
          </div>
          <div className="flex items-center justify-between">
            <Skeleton className="h-4 w-[60px]" />
            <Skeleton className="h-6 w-[90px]" />
          </div>
        </div>
      </Card>
    </div>
  );
}

export function NetworkStats({
  hasFailed,
  wan,
}: {
  hasFailed: boolean;
  wan: NetworkWan | null;
}) {
  console.log(wan);
  if (!wan) {
    return (
      <div className="relative">
        <NetworkStatsSkeleton />
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <div
            className={cn(
              "text-center text-destructive bg-background/40 p-4 rounded-lg  border-destructive border transition-opacity duration-500 ease-in-out",
              hasFailed ? "opacity-100" : "opacity-0",
            )}
          >
            <AlertTriangle className="mx-auto h-8 w-8 mb-2" />
            <h3 className="text-lg font-semibold">Error</h3>
            <p className="text-sm text-destructive/80">
              Could not fetch network statistics.
            </p>
          </div>
        </div>
      </div>
    );
  }

  // Helper to calculate progress safely
  const getProgress = (current: number, max: number) => {
    if (!max || max === 0) return 0;
    return (current / max) * 100;
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {/* Download Card */}
      <Card className="p-4 border-border bg-card">
        <div className="flex items-center gap-3 mb-3">
          <div className="p-2 rounded-lg bg-primary/10 relative">
            <Download className="h-4 w-4 text-primary" />
          </div>
          <div>
            <p className="text-sm font-medium text-card-foreground">Download</p>
            <p className="text-xs text-muted-foreground">Current usage</p>
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-2xl font-bold text-primary">
              {formatBandwidth(wan.stats.download.currentBandwidth)}
            </span>
            <span className="text-sm text-muted-foreground">
              / {formatBandwidth(wan.stats.download.maxBandwidth)}
            </span>
          </div>
          <Progress
            value={getProgress(
              wan.stats.download.currentBandwidth,
              wan.stats.download.maxBandwidth,
            )}
            className="h-2"
          />
          <p className="text-xs text-muted-foreground/70 mt-1">
            {wan.stats.download.packetsLost} packets dropped
          </p>
        </div>
      </Card>

      {/* Upload Card */}
      <Card className="p-4 border-border bg-card">
        <div className="flex items-center gap-3 mb-3">
          <div className="p-2 rounded-lg bg-accent/10 relative">
            <Upload className="h-4 w-4 text-accent" />
          </div>
          <div>
            <p className="text-sm font-medium text-card-foreground">Upload</p>
            <p className="text-xs text-muted-foreground">Current usage</p>
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-2xl font-bold text-accent">
              {formatBandwidth(wan.stats.upload.currentBandwidth)}
            </span>
            <span className="text-sm text-muted-foreground">
              / {formatBandwidth(wan.stats.upload.maxBandwidth)}
            </span>
          </div>
          <Progress
            value={getProgress(
              wan.stats.upload.currentBandwidth,
              wan.stats.upload.maxBandwidth,
            )}
            className="h-2"
            indicatorClassName="bg-accent"
          />
          <p className="text-xs text-muted-foreground/70 mt-1">
            {wan.stats.upload.packetsLost} packets dropped
          </p>
        </div>
      </Card>

      {/* Network Status Card */}
      <Card className="p-4 border-border bg-card">
        <div className="flex items-center gap-3 mb-3">
          <div className="p-2 rounded-lg bg-success/10 relative">
            <Activity className="h-4 w-4 text-success" />
          </div>
          <div>
            <p className="text-sm font-medium text-card-foreground">
              Network Status
            </p>
            <p className="text-xs text-muted-foreground">System health</p>
          </div>
        </div>
        <div className="space-y-3 pt-1">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">
              Active Connections
            </span>
            <span className="text-lg font-semibold text-card-foreground">
              {wan.stats.activeSessions}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">Uptime</span>
            <span className="text-lg font-semibold text-success">
              {formatDuration(wan.connectivity.uptime)}
            </span>
          </div>
        </div>
      </Card>
    </div>
  );
}
