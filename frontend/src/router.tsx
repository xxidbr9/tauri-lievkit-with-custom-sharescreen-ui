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
} from "@/components/ui/frame";
import { Button } from "@/components/ui/button";
import {
  Avatar,
  AvatarBadge,
  AvatarFallback,
  AvatarImage,
} from "@/components/ui/avatar";
import {
  VideoCameraIcon,
  FolderIcon,
  HandPalmIcon,
  PhoneSlashIcon,
  ListIcon,
  ChatCircleIcon,
  MicrophoneIcon,
  MonitorArrowUpIcon,
  ChalkboardSimpleIcon,
} from "@phosphor-icons/react";
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
} from "@/components/ui/menu";
import Dock from "@/components/ui/dock";
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
} from "@/components/ui/alert-dialog";
import { cn } from "@/lib/utils";
import React from "react";
import { exit } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

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

const triggerError = async () => {
  await invoke("risk_command");
};

const panicTest = async () => {
  await invoke("panic_test");
};

const testScreen = async () => {
  await invoke("get_list");
};

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
        <div className="flex flex-col gap-y-2">
          <Tooltip>
            <TooltipTrigger
              render={<Button onClick={triggerError}>Risky Error</Button>}
            />
            <TooltipContent>Risky Error</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger
              render={
                <Button variant={"destructive"} onClick={panicTest}>
                  Panic Test
                </Button>
              }
            />
            <TooltipContent>Panic Test</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger
              render={<Button onClick={testScreen}>Screen Share Test</Button>}
            />
            <TooltipContent>Screen Share Test</TooltipContent>
          </Tooltip>
        </div>
        {/*<h2 className="font-semibold text-sm">Section title</h2>
        <p className="text-muted-foreground text-sm">Section description</p>*/}
      </FramePanel>
      {/*<div className="absolute bottom-2 left-1/2 transform -translate-x-1/2 flex items-end w-fit gap-4 rounded-2xl border-neutral-700 border-2 pb-2 px-4">
        Hello
      </div>*/}
      <Dock
        items={[
          {
            icon: <MicrophoneIcon className="size-4" strokeWidth={1.5} />,
            label: "Mic",
            onClick: () => alert("Mic!"),
          },
          {
            icon: <VideoCameraIcon className="size-4" strokeWidth={1.5} />,
            label: "Camera",
            onClick: () => alert("Camera!"),
          },
          {
            icon: <HandPalmIcon className="size-4" strokeWidth={1.5} />,
            label: "Raise Hand",
            onClick: () => alert("Raise Hand!"),
          },
          {
            icon: <MonitorArrowUpIcon className="size-4" strokeWidth={1.5} />,
            label: "Share Screen",
            onClick: () => alert("Raise Hand!"),
          },
          {
            icon: <ChalkboardSimpleIcon className="size-4" strokeWidth={1.5} />,
            label: "Board",
            onClick: () => alert("Raise Hand!"),
          },
          {
            icon: <ChatCircleIcon className="size-4" strokeWidth={1.5} />,
            label: "Chat",
            onClick: () => alert("Chat!"),
          },
          {
            icon: <FolderIcon className="size-4" strokeWidth={1.5} />,
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
              <ListIcon className="size-4" />
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
              nativeButton
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
