import { Button } from "@/components/ui/Button";
import { Settings } from "lucide-react";
import helios from "@/assets/helios.png";

const Header = () => {
  return (
    <header className="border-b border-border bg-card/50 backdrop-blur-sm sticky top-0 z-50">
      <div className="container mx-auto px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/10">
              <img src={helios} className="h-6 w-6 text-primary" />
            </div>
            <div>
              <h1 className="text-xl font-semibold text-foreground">Helios</h1>
              <p className="text-sm text-muted-foreground">
                Dashboard & Device Management
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 text-foreground">
            <Button variant="ghost" size="sm">
              <Settings className="h-4 w-4 mr-2" />
              Settings
            </Button>
          </div>
        </div>
      </div>
    </header>
  );
};

export default Header;
