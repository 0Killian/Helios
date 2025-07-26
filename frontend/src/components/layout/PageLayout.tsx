import Header from "./Header";

const PageLayout = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="bg-background w-full">
      <Header />
      <main className="container mx-auto px-6 py-8">
        <div className="space-y-8">{children}</div>
      </main>
    </div>
  );
};

export default PageLayout;
