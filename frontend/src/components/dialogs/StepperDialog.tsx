import { JSX, useState } from "react";
import { DialogDescription, DialogHeader, DialogTitle } from "../ui/Dialog";
import { cn } from "@/lib";
import { Button } from "../ui/Button";
import { Loader2 } from "lucide-react";

export interface Step {
  id: number;
  content: JSX.Element;
  back: boolean;
  onNext?: () => Promise<boolean>;
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
  const [isLoading, setIsLoading] = useState(false);

  const handleNext = async () => {
    if (steps[currentStep - 1].onNext) {
      setIsLoading(true);
      const success = await steps[currentStep - 1].onNext!();
      setIsLoading(false);
      if (success) {
        setCurrentStep((prev) => Math.min(prev + 1, steps.length));
      }
    } else {
      setCurrentStep((prev) => Math.min(prev + 1, steps.length));
    }
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
        <div className="mt-6 flex flex-col w-full">
          <div className="overflow-hidden">
            <div
              className="flex transition-transform duration-300 ease-in-out"
              style={{
                transform: `translateX(-${(currentStep - 1) * 100}%)`,
              }}
            >
              {steps.map((step) => (
                <div key={step.id} className="w-full flex-shrink-0">
                  {step.content}
                </div>
              ))}
            </div>
          </div>
          <div className="flex justify-between mt-8 w-full">
            {currentStep > 1 && steps[currentStep - 1].back ? (
              <Button variant="outline" onClick={handleBack}>
                Back
              </Button>
            ) : (
              <div /> // Empty div to keep "Next" button on the right
            )}
            {currentStep < steps.length && (
              <Button onClick={handleNext}>
                Next
                <Loader2
                  className={cn(
                    "ml-2 h-4 w-4 animate-spin",
                    !isLoading ? "hidden" : "",
                  )}
                />
              </Button>
            )}
            {currentStep === steps.length && (
              <Button onClick={handleNext}>
                Finish
                <Loader2
                  className={cn(
                    "ml-2 h-4 w-4 animate-spin",
                    !isLoading ? "hidden" : "",
                  )}
                />
              </Button>
            )}
          </div>
        </div>
      </div>
    </>
  );
};
