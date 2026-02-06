"use client";

import { Tooltip as TooltipPrimitive } from "@base-ui/react/tooltip";
import { HTMLMotionProps, motion } from "motion/react";

import { cn } from "@/lib/utils";

const TooltipCreateHandle = TooltipPrimitive.createHandle;

const TooltipProvider = TooltipPrimitive.Provider;

const Tooltip = TooltipPrimitive.Root;

function TooltipTrigger(props: TooltipPrimitive.Trigger.Props) {
  return <TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...props} />;
}

type PopupProps = TooltipPrimitive.Popup.Props;

type MotionPopupProps = PopupProps & HTMLMotionProps<"div">;

const MotionPopup = motion.create(
  TooltipPrimitive.Popup as any,
) as React.FC<MotionPopupProps>;

type Side = "top" | "bottom" | "left" | "right" | "inline-start" | "inline-end";

function resolveInline(side: Side): "left" | "right" | Side {
  if (side !== "inline-start" && side !== "inline-end") return side;

  const dir =
    typeof document !== "undefined"
      ? document.documentElement.dir || "ltr"
      : "ltr";

  if (dir === "rtl") {
    return side === "inline-start" ? "right" : "left";
  } else {
    return side === "inline-start" ? "left" : "right";
  }
}

function getMotion(rawSide: Side) {
  const side = resolveInline(rawSide);
  const d = 6;
  const axis = side === "top" || side === "bottom" ? "y" : "x";
  const dir = side === "bottom" || side === "right" ? -d : d;

  return side === "top" ||
    side === "bottom" ||
    side === "left" ||
    side === "right"
    ? {
        initial: { [axis]: dir, opacity: 0 },
        animate: { [axis]: 0, opacity: 1 },
        exit: { [axis]: dir, opacity: 0 },
      }
    : {
        initial: { opacity: 0 },
        animate: { opacity: 1 },
        exit: { opacity: 0 },
      };
}

function TooltipPopup({
  className,
  align = "center",
  sideOffset = 4,
  side = "top",
  children,
  ...props
}: MotionPopupProps & {
  align?: TooltipPrimitive.Positioner.Props["align"];
  side?: TooltipPrimitive.Positioner.Props["side"];
  sideOffset?: TooltipPrimitive.Positioner.Props["sideOffset"];
}) {
  const motionCfg = getMotion(side);
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Positioner
        align={align}
        className="z-50 h-(--positioner-height) w-(--positioner-width) max-w-(--available-width) transition-[top,left,right,bottom,transform] data-instant:transition-none"
        data-slot="tooltip-positioner"
        side={side}
        sideOffset={sideOffset}
      >
        <MotionPopup
          initial={motionCfg.initial}
          animate={motionCfg.animate}
          transition={{ duration: 0.14, ease: "easeOut" }}
          className={cn(
            "relative flex h-(--popup-height,auto) w-(--popup-width,auto) origin-(--transform-origin) text-balance rounded-md border bg-popover not-dark:bg-clip-padding text-popover-foreground text-xs shadow-md/5 transition-[width,height,scale,opacity] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-md)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] data-ending-style:scale-98 data-starting-style:scale-98 data-ending-style:opacity-0 data-starting-style:opacity-0 data-instant:duration-0 dark:before:shadow-[0_-1px_--theme(--color-white/6%)]",
            className,
          )}
          data-slot="tooltip-popup"
          {...props}
        >
          <TooltipPrimitive.Viewport
            className="relative size-full overflow-clip px-(--viewport-inline-padding) py-1 [--viewport-inline-padding:--spacing(2)] data-instant:transition-none **:data-current:data-ending-style:opacity-0 **:data-current:data-starting-style:opacity-0 **:data-previous:data-ending-style:opacity-0 **:data-previous:data-starting-style:opacity-0 **:data-current:w-[calc(var(--popup-width)-2*var(--viewport-inline-padding)-2px)] **:data-previous:w-[calc(var(--popup-width)-2*var(--viewport-inline-padding)-2px)] **:data-previous:truncate **:data-current:opacity-100 **:data-previous:opacity-100 **:data-current:transition-opacity **:data-previous:transition-opacity"
            data-slot="tooltip-viewport"
          >
            {children}
          </TooltipPrimitive.Viewport>
        </MotionPopup>
      </TooltipPrimitive.Positioner>
    </TooltipPrimitive.Portal>
  );
}

export {
  TooltipCreateHandle,
  TooltipProvider,
  Tooltip,
  TooltipTrigger,
  TooltipPopup,
  TooltipPopup as TooltipContent,
};
