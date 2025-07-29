import { Device } from "@/models";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { Button } from "@/components/ui/Button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/Dialog";
import { formatDistanceToNow } from "date-fns";
import { Computer, MoreVertical } from "lucide-react";
import { AddServiceDialog } from "./dialogs/AddServiceDialog";
import { useState } from "react";

interface DeviceListItemProps {
  device: Device;
  onRefresh?: () => void;
}

export function DeviceListItem({ device, onRefresh }: DeviceListItemProps) {
  const [isAddServiceDialogOpen, setIsAddServiceDialogOpen] = useState(false);

  const onAddServiceComplete = () => {
    setIsAddServiceDialogOpen(false);
    onRefresh?.();
  };

  return (
    <Card className="border-border bg-secondary/30 hover:bg-secondary/50 transition-all duration-100">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="p-2 rounded-lg bg-primary/10 mt-1">
              <Computer className="h-6 w-6 text-primary" />
            </div>
            <div className="flex flex-col">
              <CardTitle className="text-base text-card-foreground">
                {device.displayName ? device.displayName : "Unknown"}
              </CardTitle>
              <CardDescription className="text-muted-foreground font-medium">
                MAC: <span className="text-primary">{device.macAddress}</span>
              </CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={device.isOnline ? "success" : "destructive"}>
              {device.isOnline ? "Online" : "Offline"}
            </Badge>
            <Dialog
              open={isAddServiceDialogOpen}
              onOpenChange={setIsAddServiceDialogOpen}
            >
              <DialogTrigger asChild>
                <Button variant="outline" size="sm">
                  Add Service
                </Button>
              </DialogTrigger>
              <DialogContent>
                <AddServiceDialog
                  device={device}
                  onClose={onAddServiceComplete}
                />
              </DialogContent>
            </Dialog>
            <DropdownMenu modal={false}>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon">
                  <MoreVertical className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuItem>Scan for services</DropdownMenuItem>
                <DropdownMenuItem>Edit device</DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </CardHeader>
      <CardContent className="text-muted-foreground text-base">
        <div className="flex justify-between">
          <span className="font-medium">
            IP: <span className="text-primary">{device.lastKnownIp}</span>
          </span>
          <span>
            Last seen:{" "}
            {formatDistanceToNow(new Date(device.lastSeen), {
              addSuffix: true,
            })}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}
