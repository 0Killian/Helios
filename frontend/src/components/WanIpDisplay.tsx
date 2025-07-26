import { Card } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { Globe, Copy, CheckCircle2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { useState } from "react";
import { useToast } from "@/hooks/use-toast";
import { Skeleton } from "./ui/Skeleton";
import { cn } from "@/lib/utils";
import { NetworkWan } from "@/models";

const WanIpDisplaySkeleton = () => {
  return (
    <Card className="p-6 border-border bg-card">
      {/* Header Placeholder */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <Skeleton className="h-9 w-9 rounded-lg" />
          <div className="space-y-2">
            <Skeleton className="h-4 w-[200px]" />
            <Skeleton className="h-4 w-[150px]" />
          </div>
        </div>
        <Skeleton className="h-6 w-[50px] rounded-full" />
      </div>

      <div className="space-y-4">
        {/* IPv4 Placeholder */}
        <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
          <div className="flex-1 space-y-2">
            <Skeleton className="h-4 w-[50px]" />
            <Skeleton className="h-6 w-[150px]" />
          </div>
          <Skeleton className="h-8 w-8" />
        </div>

        {/* IPv6 Placeholder */}
        <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
          <div className="flex-1 space-y-2">
            <Skeleton className="h-4 w-[50px]" />
            <Skeleton className="h-6 w-[250px]" />
          </div>
          <Skeleton className="h-8 w-8" />
        </div>
      </div>
    </Card>
  );
};

export function WanIpDisplay({
  hasFailed,
  wan,
}: {
  hasFailed: boolean;
  wan: NetworkWan | null;
}) {
  const { toast } = useToast();

  const [copiedIp, setCopiedIp] = useState<string | null>(null);

  const copyToClipboard = async (ip: string) => {
    try {
      await navigator.clipboard.writeText(ip);
      setCopiedIp(ip);
      toast({
        title: "IP Copied",
        description: `${ip} copied to clipboard`,
      });
      setTimeout(() => setCopiedIp(null), 2000);
    } catch (err) {
      toast({
        title: "Copy Failed",
        description: "Failed to copy IP address",
        variant: "destructive",
      });
      console.error(err);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "Up":
        return "bg-success text-success-foreground";
      case "Down":
        return "bg-destructive text-destructive-foreground";
      default:
        return "bg-muted text-muted-foreground";
    }
  };

  if (!wan) {
    return (
      <div className="relative">
        <WanIpDisplaySkeleton />
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
              Could not fetch WAN information.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <Card className="p-6 border-border bg-card">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-primary/10">
            <Globe className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-card-foreground">
              WAN IP Addresses
            </h3>
            <p className="text-sm text-muted-foreground">
              External network connectivity
            </p>
          </div>
        </div>
        <Badge className={getStatusColor(wan.connectivity!.status)}>
          {wan.connectivity!.status}
        </Badge>
      </div>

      <div className="space-y-4">
        {wan.connectivity!.ipv4 && (
          <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-secondary-foreground">
                  IPv4
                </span>
                <div className="w-2 h-2 rounded-full bg-success animate-pulse"></div>
              </div>
              <code className="text-lg font-mono text-primary">
                {wan.connectivity!.ipv4}
              </code>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => copyToClipboard(wan.connectivity!.ipv4!)}
              className="ml-2"
            >
              {copiedIp === wan.connectivity!.ipv4 ? (
                <CheckCircle2 className="h-4 w-4 text-success" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </Button>
          </div>
        )}

        {wan.connectivity!.ipv6 && (
          <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-sm font-medium text-secondary-foreground">
                  IPv6
                </span>
                <div className="w-2 h-2 rounded-full bg-success animate-pulse"></div>
              </div>
              <code className="text-lg font-mono text-primary break-all">
                {wan.connectivity!.ipv6}
              </code>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => copyToClipboard(wan.connectivity!.ipv6!)}
              className="ml-2"
            >
              {copiedIp === wan.connectivity!.ipv6 ? (
                <CheckCircle2 className="h-4 w-4 text-success" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </Button>
          </div>
        )}
      </div>
    </Card>
  );
}
