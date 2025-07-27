import { Card } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { Button } from "@/components/ui/Button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";
import { Monitor, ChevronDown, AlertTriangle } from "lucide-react";
import { Device, DeviceFull } from "@/models";
import { cn } from "@/lib";
import { Skeleton } from "./ui/Skeleton";
import { RootState } from "@/store";

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
  const getDeviceIcon = () => {
    // TODO: device type
    return Monitor;
  };

  const getStatusColor = (device: Device) => {
    return device.isOnline
      ? "bg-success text-success-foreground"
      : "bg-muted text-muted-foreground";
  };

  /*const getServiceStatusColor = (status: string) => {
    switch (status) {
      case "running":
        return "text-success";
      case "stopped":
        return "text-destructive";
      default:
        return "text-muted-foreground";
    }
  };*/

  // const handleServiceControl = (macAddress: string, port: number) => {
  //   // In real app, this would navigate to service control panel
  //   console.log(
  //     `Opening control panel for service on port ${port} on device ${macAddress}`,
  //   );
  // };

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

      <div className="space-y-4">
        {devices
          // Sort them by connected status and mac address
          .toSorted((a, b) => {
            if (a.device.isOnline !== b.device.isOnline) {
              return a.device.isOnline ? -1 : 1;
            }
            return a.device.macAddress.localeCompare(b.device.macAddress);
          })
          .map(({ device }) => {
            const DeviceIcon = getDeviceIcon();

            return (
              <div
                key={device.macAddress}
                className="p-4 rounded-lg border border-border bg-secondary/30 hover:bg-secondary/50 transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-3 flex-1">
                    <div className="p-2 rounded-lg bg-primary/10 mt-1">
                      <DeviceIcon className="h-4 w-4 text-primary" />
                    </div>

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <h4
                          className={cn(
                            "font-medium text-card-foreground truncate",
                            device.displayName === "" ? "italic" : "",
                          )}
                        >
                          {device.displayName === ""
                            ? "(Unknown)"
                            : device.displayName}
                        </h4>
                        <Badge className={getStatusColor(device)}>
                          {device.isOnline ? "Connected" : "Disconnected"}
                        </Badge>
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-sm text-muted-foreground mb-3">
                        <div>
                          <span className="font-medium">MAC:</span>{" "}
                          <code className="text-primary">
                            {device.macAddress}
                          </code>
                        </div>
                        <div>
                          <span className="font-medium">IP:</span>{" "}
                          <code className="text-primary">
                            {device.lastKnownIp}
                          </code>
                        </div>
                      </div>

                      <div className="text-xs text-muted-foreground">
                        Last seen: {new Date(device.lastSeen).toLocaleString()}
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 ml-4">
                    <DropdownMenu modal={false}>
                      <DropdownMenuTrigger asChild>
                        <Button variant="outline" size="sm">
                          Services
                          <ChevronDown className="h-3 w-3 ml-1" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end" className="w-64">
                        <DropdownMenuItem
                          className="flex items-center justify-between p-3"
                          disabled
                        >
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <span className="font-medium">
                                No services found
                              </span>
                            </div>
                          </div>
                        </DropdownMenuItem>
                        {/*{device.services.map((service, index) => (
                        <div key={index}>
                          <DropdownMenuItem className="flex items-center justify-between p-3">
                            <div className="flex-1">
                              <div className="flex items-center gap-2">
                                <span className="font-medium">
                                  {service.name}
                                </span>
                                <span
                                  className={`text-xs ${getServiceStatusColor(service.status)}`}
                                >
                                  ●
                                </span>
                              </div>
                              <div className="text-xs text-muted-foreground">
                                Port {service.port} • {service.status}
                              </div>
                            </div>

                            {service.controllable && (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() =>
                                  handleServiceControl(device.id, service)
                                }
                                className="ml-2"
                              >
                                {service.name === "DNS" && (
                                  <Shield className="h-3 w-3" />
                                )}
                                {service.name === "Database" && (
                                  <Database className="h-3 w-3" />
                                )}
                                {!["DNS", "Database"].includes(
                                  service.name,
                                ) && <Settings className="h-3 w-3" />}
                              </Button>
                            )}
                          </DropdownMenuItem>
                          {index < device.services.length - 1 && (
                            <DropdownMenuSeparator />
                          )}
                        </div>
                      ))}*/}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              </div>
            );
          })}
      </div>
    </Card>
  );
};
