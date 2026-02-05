import { useVh } from "./hooks/use-vh";
import { RouterProvider } from "@tanstack/react-router";
import { router } from "./router";
function App() {
  useVh();

  return <RouterProvider router={router}></RouterProvider>;
}

export default App;
