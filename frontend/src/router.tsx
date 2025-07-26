import { Suspense } from "react";
import { BrowserRouter, Route, Routes } from "react-router";
import Dashboard from "./pages/Dashboard";
import PageLayout from "./components/layout/PageLayout";

const Router = () => {
    return (
        <Suspense>
            <BrowserRouter>
                <PageLayout>
                    <Routes>
                        <Route path="/" element={<Dashboard />} />
                        <Route path="*" element={<div>404 Not Found</div>} />
                    </Routes>
                </PageLayout>
            </BrowserRouter>
        </Suspense>
    );
};

export default Router;