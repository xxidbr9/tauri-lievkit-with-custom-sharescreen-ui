import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
} from "@tanstack/react-router";

import { SmallSizeMeetingLayout } from "@/features/meeting";

const rootRoute = createRootRoute({
  component: () => (
    <>
      <Outlet />
      {/*<TanStackRouterDevtools />*/}
    </>
  ),
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: () => <div>Index Page</div>,
});

const meetingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/meeting",
  component: () => <SmallSizeMeetingLayout />,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  meetingRoute,
  // ... other routes
]);

const hashHistory = createHashHistory();
export const router = createRouter({
  routeTree,
  history: hashHistory,
});
