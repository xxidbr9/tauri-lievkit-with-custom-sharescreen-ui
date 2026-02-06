import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import {
  Frame,
  FrameDescription,
  FrameFooter,
  FrameHeader,
  FramePanel,
  FrameTitle,
} from "./components/ui/frame";
import { Button } from "./components/ui/button";
import {
  Avatar,
  AvatarBadge,
  AvatarFallback,
  AvatarImage,
} from "./components/ui/avatar";
import {
  Camera,
  Folder,
  Hand,
  LucidePhoneOff,
  Menu,
  MessageCircleIcon,
  Mic,
  MonitorUp,
  Presentation,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "./components/ui/menu";
import Dock from "./components/ui/dock";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./components/ui/alert-dialog";
import { cn } from "./lib/utils";
import React from "react";
import { exit } from "@tauri-apps/plugin-process";

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
  component: () => (
    <Frame className="w-full h-full relative">
      <FrameHeader className="flex items-center flex-row justify-between">
        <div className="flex flex-col">
          <FrameTitle>Livekit Meeting</FrameTitle>
          <FrameDescription>#CODE</FrameDescription>
        </div>
        <DropdownMenuDemo />
      </FrameHeader>
      <FramePanel className="flex-1">
        {/*<h2 className="font-semibold text-sm">Section title</h2>
        <p className="text-muted-foreground text-sm">Section description</p>*/}
      </FramePanel>
      {/*<div className="absolute bottom-2 left-1/2 transform -translate-x-1/2 flex items-end w-fit gap-4 rounded-2xl border-neutral-700 border-2 pb-2 px-4">
        Hello
      </div>*/}
      <Dock
        items={[
          {
            icon: <Mic className="size-4" strokeWidth={1.5} />,
            label: "Mic",
            onClick: () => alert("Mic!"),
          },
          {
            icon: <Camera className="size-4" strokeWidth={1.5} />,
            label: "Camera",
            onClick: () => alert("Camera!"),
          },
          {
            icon: <Hand className="size-4" strokeWidth={1.5} />,
            label: "Raise Hand",
            onClick: () => alert("Raise Hand!"),
          },
          {
            icon: <MonitorUp className="size-4" strokeWidth={1.5} />,
            label: "Share Screen",
            onClick: () => alert("Raise Hand!"),
          },
          {
            icon: <Presentation className="size-4" strokeWidth={1.5} />,
            label: "Board",
            onClick: () => alert("Raise Hand!"),
          },
          {
            icon: <MessageCircleIcon className="size-4" strokeWidth={1.5} />,
            label: "Chat",
            onClick: () => alert("Chat!"),
          },
          {
            icon: <Folder className="size-4" strokeWidth={1.5} />,
            label: "Files",
            onClick: () => alert("Files!"),
          },
        ]}
      />
      {/*<FrameFooter>
        <p className="text-muted-foreground text-sm">Footer</p>
      </FrameFooter>*/}
    </Frame>
  ),
});

export function DropdownMenuDemo() {
  const [open, setOpen] = React.useState(false);
  const handleExit = () => {
    exit();
  };
  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button
              variant={"outline"}
              size={"lg"}
              className="justify-between rounded-2xl gap-x-2 ps-1 border"
            >
              <Avatar>
                <AvatarImage src="https://github.com/xxidbr9.png" />
                <AvatarFallback>XX</AvatarFallback>
                <AvatarBadge className="bg-green-600 dark:bg-green-500" />
              </Avatar>
              <Menu className="size-4" />
            </Button>
          }
        />
        <DropdownMenuContent className="w-40 mt-1" align="start">
          <DropdownMenuGroup>
            <DropdownMenuLabel>My Account</DropdownMenuLabel>
            <DropdownMenuItem>
              Profile
              <DropdownMenuShortcut>Alt+P</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuItem>
              Settings
              <DropdownMenuShortcut>Alt+S</DropdownMenuShortcut>
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuGroup>
            <DropdownMenuItem
              render={(props) => (
                <button
                  {...props}
                  className={cn(props.className, "w-full")}
                  onClick={() => setOpen(true)}
                />
              )}
            >
              Log out
              <DropdownMenuShortcut>Ctrl+Q</DropdownMenuShortcut>
            </DropdownMenuItem>
          </DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
      <AlertDialog onOpenChange={setOpen} open={open}>
        <AlertDialogContent className={"max-w-sm"}>
          <AlertDialogHeader className="place-items-baseline">
            <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
            <AlertDialogDescription>
              Log out will remove all your data from this session.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleExit}>Continue</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

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
