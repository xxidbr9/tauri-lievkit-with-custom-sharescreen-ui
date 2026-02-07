import {
  Frame,
  FrameDescription,
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
  DropdownMenuSeparator,
  DropdownMenuShortcut,
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
} from "@/components/ui/alert-dialog";
import { cn } from "@/lib/utils";
import React, { useEffect } from "react";
import { exit } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { listen } from "@tauri-apps/api/event";

// import { useDebugStreamFps } from "@/hooks/use-debug-stream-fps";

const triggerError = async () => {
  await invoke("risk_command");
};

const panicTest = async () => {
  await invoke("panic_test");
};

const startShareScreen = async () => {
  await invoke("start_share_screen");
};

const closeShareScreen = async () => {
  await invoke("close_share_screen");
};

const streamList = async () => {
  await invoke("stream_list");
};

const closeStreamList = async () => {
  await invoke("close_stream_list");
};

export const SmallSizeMeetingLayout = () => {
  useEffect(() => {
    let frameCount = 0;
    let lastTime = Date.now();

    const unlisten = listen("share-screen-list", (data: any) => {
      frameCount++;
      const currentTime = Date.now();
      const elapsed = currentTime - lastTime;

      // Calculate FPS every second
      if (elapsed >= 1000) {
        const actualFPS = Math.round((frameCount * 1000) / elapsed);
        console.log({
          actualFPS,
          backendReportedFPS: data.payload.fps,
          payload: data.payload.sources,
        });

        frameCount = 0;
        lastTime = currentTime;
      }
    });

    return () => {
      unlisten.then(() => {
        console.log("Unlisten successful");
      });
    };
  }, []);
  return (
    <Frame className="w-full h-full relative">
      <FrameHeader className="flex items-center flex-row justify-between">
        <div className="flex flex-col">
          <FrameTitle>Livekit Meeting</FrameTitle>
          <FrameDescription>#CODE</FrameDescription>
        </div>
        <DropdownProfile />
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
              render={<Button onClick={streamList}>Stream List</Button>}
            />
            <TooltipContent>Stream List</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger
              render={
                <Button onClick={closeStreamList}>Close Stream List</Button>
              }
            />
            <TooltipContent>Close Stream List</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger
              render={
                <Button onClick={startShareScreen}>Start Share Screen</Button>
              }
            />
            <TooltipContent>Start Share Screen</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger
              render={
                <Button onClick={closeShareScreen}>Close Screen Share</Button>
              }
            />
            <TooltipContent>Close Screen Share</TooltipContent>
          </Tooltip>
        </div>
      </FramePanel>
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
    </Frame>
  );
};

export function DropdownProfile() {
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
