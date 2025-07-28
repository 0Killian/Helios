import { Device } from "@/models";
import { Step, StepperDialog } from "./StepperDialog";

const steps: Step[] = [
  {
    id: 1,
    title: "Service Specification",
    content: (step: Step) => (
      <div>
        <h3 className="text-lg font-semibold text-center mb-4">{step.title}</h3>
      </div>
    ),
  },
  {
    id: 2,
    title: "Agent Configuration",
    content: (step: Step) => (
      <div>
        <h3 className="text-lg font-semibold text-center mb-4">{step.title}</h3>
      </div>
    ),
  },
  {
    id: 3,
    title: "Verification",
    content: (step: Step) => (
      <div>
        <h3 className="text-lg font-semibold text-center mb-4">{step.title}</h3>
        <p className="text-2xl font-bold text-success">Done</p>
      </div>
    ),
  },
];

export const AddServiceDialog = ({ device }: { device: Device }) => {
  return (
    <StepperDialog
      title={`Add a new service to ${device.displayName}`}
      description="Follow the steps to configure and register a new service."
      steps={steps}
    />
  );
};
