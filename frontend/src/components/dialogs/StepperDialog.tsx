import { JSX, useState } from "react";
import { DialogDescription, DialogHeader, DialogTitle } from "../ui/Dialog";
import { cn } from "@/lib";
import { Button } from "../ui/Button";

export interface Step {
  id: number;
  title: string;
  content: (step: Step) => JSX.Element;
}

export const StepperDialog = ({
  title,
  description,
  steps,
}: {
  title: string;
  description: string;
  steps: Step[];
}) => {
  const [currentStep, setCurrentStep] = useState(1);

  const handleNext = () => {
    setCurrentStep((prev) => Math.min(prev + 1, steps.length));
  };

  const handleBack = () => {
    setCurrentStep((prev) => Math.max(prev - 1, 1));
  };

  return (
    <>
      <DialogHeader>
        <DialogTitle>{title}</DialogTitle>
        <DialogDescription>{description}</DialogDescription>
      </DialogHeader>

      <div className="flex flex-col items-center justify-center mt-8">
        <div className="flex flew-row">
          {steps.map((step, index) => (
            <div key={step.id} className="flex items-center">
              <div
                className={cn(
                  "flex items-center justify-center w-2 h-2 rounded-full transition-colors duration-300",
                  currentStep >= step.id ? "bg-primary" : "bg-white/50",
                )}
              ></div>
              {index < steps.length - 1 && (
                <div className="w-24 h-1 bg-white/50 overflow-hidden">
                  <div
                    className={cn(
                      "h-full bg-primary transition-all duration-200 ease-in-out",
                      currentStep > step.id ? "w-full" : "w-0",
                    )}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
        <div className="mt-6 min-h-[150px] flex flex-col w-full">
          <div className="flex-grow relative overflow-hidden">
            {steps.map((step) => (
              <div
                className={cn(
                  "absolute w-full h-full transition-transform duration-200 ease-in-out",
                  currentStep === step.id
                    ? "translate-x-0"
                    : step.id < currentStep
                      ? "-translate-x-full"
                      : "translate-x-full",
                )}
              >
                {step.content(step)}
              </div>
            ))}
          </div>
          <div className="flex justify-between mt-8 w-full">
            {currentStep > 1 ? (
              <Button variant="outline" onClick={handleBack}>
                Back
              </Button>
            ) : (
              <div /> // Empty div to keep "Next" button on the right
            )}
            {currentStep < steps.length && (
              <Button onClick={handleNext}>Next</Button>
            )}
          </div>
        </div>
      </div>
    </>
  );
};
