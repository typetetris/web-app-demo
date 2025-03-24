import { createBrowserRouter } from "react-router";
import App from "./App";
import { ErrorView } from "./components/ErrorView";

const router = createBrowserRouter([
  {
    path: "/",
    Component: App,
    ErrorBoundary: ErrorView,
  },
  {
    path: "/:chatName/:chatId",
    Component: App,
    ErrorBoundary: ErrorView,
  },
]);

export default router;
