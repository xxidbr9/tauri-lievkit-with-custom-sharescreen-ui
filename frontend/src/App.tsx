import { useVh } from "./hooks/use-vh";
import { RouterProvider } from "@tanstack/react-router";
import { router } from "./router";
import { useAppError } from "./hooks/use-app-error";
import { DirectionProvider } from "./components/ui/direction";
import { ShadcnProvider } from "./hooks/shadcn-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import ThemeProvider from "./hooks/theme-provider";

const queryClient = new QueryClient();

function App() {
  useVh();
  useAppError();
  return (
    <DirectionProvider direction="ltr">
      <ThemeProvider>
        <ShadcnProvider>
          <QueryClientProvider client={queryClient}>
            {/*<ReactQueryDevtools initialIsOpen={false} />*/}
            <RouterProvider router={router} />
          </QueryClientProvider>
        </ShadcnProvider>
      </ThemeProvider>
    </DirectionProvider>
  );
}

export default App;
