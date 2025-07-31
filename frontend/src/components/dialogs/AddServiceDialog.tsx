import { Device, Service } from "@/models";
import { Step, StepperDialog } from "./StepperDialog";
import { ServiceSpecificationForm } from "../forms/ServiceSpecificationForm";
import { useState, useCallback, useRef, useEffect } from "react";
import { Button } from "../ui/Button";
import { useAppDispatch, useAppSelector, useToast } from "@/hooks";
import { Copy } from "lucide-react";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { atomDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { createService, CreateService } from "@/features/services.slice";

const AgentConfigurationForm = ({ service }: { service: Service }) => {
  const serviceJson = JSON.stringify(service, null, 2);
  const { toast } = useToast();

  const handleCopy = () => {
    navigator.clipboard.writeText(serviceJson);
    toast({
      title: "Copied to clipboard",
      description:
        "The service configuration has been copied to your clipboard.",
    });
  };

  return (
    service && (
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground text-center">
          On your device, run the following script to install and configure the
          Helios agent.
        </p>
        <div className="relative">
          <pre className="p-4 rounded-md bg-background border border-border text-sm overflow-x-auto">
            <SyntaxHighlighter
              language="bash"
              style={atomDark}
              customStyle={{
                borderRadius: "var(--radius)",
                border: "1px solid hsl(var(--color-border))",
                backgroundColor: "hsl(var(--color-background))",
                padding: "1rem",
              }}
              codeTagProps={{
                className: "text-sm",
              }}
            >
              {`$ curl "http://localhost:3000/api/v1/services/${service.serviceId}/install-script?os=linux" | sudo bash`}
            </SyntaxHighlighter>
          </pre>
          <Button
            variant="ghost"
            size="icon"
            className="absolute top-2 right-2 h-7 w-7"
            onClick={handleCopy}
          >
            <Copy className="h-4 w-4" />
          </Button>
        </div>
      </div>
    )
  );
};

export const AddServiceDialog = ({
  device,
  onClose,
}: {
  device: Device;
  onClose: () => void;
}) => {
  const dispatch = useAppDispatch();
  const { toast } = useToast();
  const [createServicePayload, setCreateServicePayload] = useState<
    Partial<CreateService>
  >({ deviceMac: device.macAddress });
  const servicePayloadRef = useRef<Partial<CreateService>>({});
  const { service } = useAppSelector((state) => state.services);

  // Update ref whenever state changes
  useEffect(() => {
    servicePayloadRef.current = createServicePayload;
  }, [createServicePayload]);

  const handleCreateService = useCallback(async () => {
    try {
      const result = await dispatch(
        createService(servicePayloadRef.current as CreateService),
      );
      if (createService.fulfilled.match(result)) {
        return true;
      } else {
        toast({
          title: "Error creating service",
          description:
            result.payload?.message ||
            "An unexpected error occurred. Please try again.",
          variant: "destructive",
        });
        return false;
      }
    } catch (error) {
      console.error(error);
      toast({
        title: "Error creating service",
        description: "An unexpected error occurred. Please try again.",
        variant: "destructive",
      });
      return false;
    }
  }, [dispatch, toast]);

  const handleFinish = async () => {
    toast({
      title: "Service created",
      description: "The service has been successfully created.",
    });
    onClose();

    return true;
  };

  const steps: Step[] = [
    {
      id: 1,
      content: (
        <ServiceSpecificationForm
          service={createServicePayload}
          setService={setCreateServicePayload}
        />
      ),
      back: false,
      onNext: handleCreateService,
    },
    {
      id: 2,
      content: <AgentConfigurationForm service={service as Service} />,
      back: false,
    },
    {
      id: 3,
      content: (
        <div>
          <h3 className="text-lg font-semibold text-center mb-4">
            Verification
          </h3>
          <p className="text-2xl font-bold text-success">Done</p>
        </div>
      ),
      back: true,
      onNext: handleFinish,
    },
  ];

  return (
    <StepperDialog
      title={`Add a new service to ${device.displayName}`}
      description="Follow the steps to configure and register a new service."
      steps={steps}
    />
  );
};
