import { Card } from "@/components/ui/Card";

import { Monitor, AlertTriangle } from "lucide-react";
import { DeviceFull } from "@/models";
import { cn } from "@/lib";
import { Skeleton } from "./ui/Skeleton";
import { RootState } from "@/store";
import { DeviceListItem } from "./DeviceListItem";

const DeviceListSkeleton = () => {
  return (
    <Card className="p-6 border-border bg-card">
      <div className="flex items-center gap-3 mb-6">
        <Skeleton className="h-9 w-9 rounded-lg" />
        <div className="space-y-2">
          <Skeleton className="h-5 w-[140px]" />
          <Skeleton className="h-4 w-[120px]" />
        </div>
      </div>
      <div className="space-y-4">
        {Array.from({ length: 3 }).map((_, index) => (
          <div
            key={index}
            className="p-4 rounded-lg border border-border bg-secondary/30"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start gap-3 flex-1">
                <Skeleton className="h-8 w-8 rounded-lg mt-1" />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-2">
                    <Skeleton className="h-5 w-[120px]" />
                    <Skeleton className="h-5 w-[60px] rounded-full" />
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2 mb-3">
                    <Skeleton className="h-4 w-[180px]" />
                    <Skeleton className="h-4 w-[140px]" />
                  </div>
                  <Skeleton className="h-3 w-[100px]" />
                </div>
              </div>
              <Skeleton className="h-8 w-[80px] ml-4" />
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
};

export const DeviceList = ({
  devices,
  hasFailed,
  status,
}: {
  devices: DeviceFull[];
  hasFailed: boolean;
  status: RootState["devices"]["status"];
}) => {
  if (devices.length === 0 && status !== "succeeded") {
    return (
      <div className="relative">
        <DeviceListSkeleton />
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
              Could not fetch devices.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <Card className="p-6 border-border bg-card">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 rounded-lg bg-accent/10">
          <Monitor className="h-5 w-5 text-accent" />
        </div>
        <div>
          <h3 className="text-lg font-semibold text-card-foreground">
            Network Devices
          </h3>
          <p className="text-sm text-muted-foreground">
            {devices.length} devices discovered
          </p>
        </div>
      </div>

      <div className="space-y-4 max-h-[60vh] overflow-y-auto">
        {devices
          // Sort them by connected status and mac address
          .toSorted((a, b) => {
            if (a.device.isOnline !== b.device.isOnline) {
              return a.device.isOnline ? -1 : 1;
            }
            return a.device.macAddress.localeCompare(b.device.macAddress);
          })
          .map(({ device }) => (
            <DeviceListItem key={device.macAddress} device={device} />
          ))}
      </div>
    </Card>
  );
};
