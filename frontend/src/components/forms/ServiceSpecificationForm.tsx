import { useEffect, useState } from "react";
import { useForm, useFieldArray, Control } from "react-hook-form";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/Form";
import { Input } from "@/components/ui/Input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { useAppDispatch, useAppSelector } from "@/hooks";
import { fetchServiceTemplates } from "@/features";
import { Skeleton } from "@/components/ui/Skeleton";
import { ServicePortTemplate, ServiceTemplate } from "@/models";
import { cn, formatKebabCaseString } from "@/lib";
import { AlertTriangle } from "lucide-react";
import { CreateService } from "@/features/services.slice";

interface FormValues extends ServiceTemplate {
  displayName: string;
}

interface ServiceSpecificationFormProps {
  service: Partial<CreateService>;
  setService: (service: Partial<CreateService>) => void;
}

const ServiceSpecificationFormSkeleton = () => (
  <div className="space-y-4">
    <Skeleton className="h-10 w-full" />
    <Skeleton className="h-10 w-full" />
    <Skeleton className="h-6 w-20 mt-2" />
    <div className="space-y-4 rounded-md border border-border p-4">
      <div className="grid grid-cols-3 gap-4">
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
      </div>
    </div>
  </div>
);

const PortField = ({
  index,
  control,
  portTemplate,
}: {
  index: number;
  control: Control<FormValues>;
  portTemplate: ServicePortTemplate;
}) => (
  <div className="grid grid-cols-3 gap-4">
    <FormField
      control={control}
      name={`ports.${index}.port`}
      render={({ field }) => (
        <FormItem>
          <FormLabel>
            {portTemplate.name}: {portTemplate.applicationProtocol} (
            {portTemplate.transportProtocol})
          </FormLabel>
          <FormControl>
            <Input
              type="number"
              min={1}
              max={65535}
              {...field}
              onChange={(e) => {
                const value = e.target.value;
                field.onChange(parseInt(value, 10));
              }}
            />
          </FormControl>
        </FormItem>
      )}
    ></FormField>
  </div>
);

export const ServiceSpecificationForm = (
  props: ServiceSpecificationFormProps,
) => {
  const dispatch = useAppDispatch();
  const { templates, status } = useAppSelector(
    (state) => state.serviceTemplates,
  );

  const [hasFailed, setHasFailed] = useState(false);

  // Fetch templates when the component mounts
  useEffect(() => {
    if (status === "idle") {
      dispatch(fetchServiceTemplates());
    } else if (status === "failed") {
      setHasFailed(true);
    } else if (status === "succeeded") {
      setHasFailed(false);
    }
  }, [status, dispatch]);

  // Display a loading state while fetching templates
  if (templates.length === 0 && status !== "succeeded") {
    return (
      <div className="relative">
        <ServiceSpecificationFormSkeleton />
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
              Could not fetch service templates.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return <RealServiceSpecificationForm templates={templates} {...props} />;
};

const RealServiceSpecificationForm = ({
  templates,
  service,
  setService,
}: ServiceSpecificationFormProps & {
  templates: ServiceTemplate[];
}) => {
  const [selectedTemplate, setSelectedTemplate] = useState<ServiceTemplate>(
    service.kind
      ? templates.find((t) => t.kind === service.kind) || templates[0]
      : templates[0],
  );

  const form = useForm<FormValues>({
    defaultValues: {
      kind: selectedTemplate.kind,
      displayName:
        service.displayName || formatKebabCaseString(selectedTemplate.kind),
      ports: selectedTemplate.ports,
    },
  });

  const { fields, replace } = useFieldArray({
    control: form.control,
    name: "ports",
  });

  useEffect(() => {
    const subscription = form.watch((value) => {
      const { kind, displayName, ports } = value;
      // Check that each port is complete
      if (!ports) {
        return;
      }

      for (const port of ports) {
        if (
          port === undefined ||
          port.port === undefined ||
          port.applicationProtocol === undefined ||
          port.transportProtocol === undefined
        ) {
          console.error(`Invalid port: ${JSON.stringify(port)}`);
          return;
        }
      }

      const fullPorts = ports.map((p) => {
        return {
          port: p!.port!,
          name: p!.name!,
          transportProtocol: p!.transportProtocol!,
          applicationProtocol: p!.applicationProtocol!,
        };
      });

      setService({
        deviceMac: service.deviceMac!,
        kind,
        displayName,
        ports: fullPorts,
      });

      return () => {
        subscription.unsubscribe();
      };
    });
  }, [form, setService, service.deviceMac]);

  // When the user selects a service kind, update the form's port fields
  const handleServiceKindChange = (kind: string) => {
    const template = templates.find((t) => t.kind === kind);

    if (template) {
      setSelectedTemplate(template);
      form.setValue("kind", kind);
      form.setValue("displayName", formatKebabCaseString(template.kind));
      replace(template.ports);
    } else {
      replace([]);
    }
  };

  return (
    <Form {...form}>
      <form className="space-y-4">
        <FormField
          control={form.control}
          name="kind"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Service Type</FormLabel>
              <Select
                onValueChange={handleServiceKindChange}
                defaultValue={field.value}
              >
                <FormControl>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a service type" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {templates.map((template) => (
                    <SelectItem key={template.kind} value={template.kind}>
                      {formatKebabCaseString(template.kind)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="displayName"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Display Name</FormLabel>
              <FormControl>
                <Input placeholder="My Awesome Service" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        {fields.length > 0 && (
          <>
            <h4 className="font-medium text-sm pt-2">Ports</h4>
            <div className="space-y-4 rounded-md border border-border p-4">
              {fields.map((field, index) => (
                <PortField
                  key={field.id}
                  index={index}
                  control={form.control}
                  portTemplate={field}
                />
              ))}
            </div>
          </>
        )}
      </form>
    </Form>
  );
};
