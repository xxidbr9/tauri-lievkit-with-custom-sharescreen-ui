import { cn } from "@/lib/utils";
import { motion, Variants } from "motion/react";
import React, { useRef } from "react";

type DockItemProps = {
  className?: string;
  children: React.ReactNode;
  onClick?: () => void;
};

function DockItem({ children, className = "", onClick }: DockItemProps) {
  const ref = useRef<HTMLDivElement>(null);

  return (
    <div
      ref={ref}
      onClick={onClick}
      className={cn(
        `cursor-pointer relative inline-flex items-center justify-center rounded-full border shadow bg-muted/75 p-2`,
        className,
      )}
    >
      {/*{Children.map(children, (child) =>
        React.isValidElement(child)
          ? cloneElement(child as React.ReactElement)
          : child,
      )}*/}
      {children}
    </div>
  );
}

type DockIconProps = {
  className?: string;
  children: React.ReactNode;
};

function DockIcon({ children, className = "" }: DockIconProps) {
  return (
    <div className={cn(`flex items-center justify-center`, className)}>
      {children}
    </div>
  );
}

export type DockItemData = {
  icon: React.ReactNode;
  label: React.ReactNode;
  onClick: () => void;
  className?: string;
};

export type DockProps = {
  items: DockItemData[];
  className?: string;
};
export default function Dock({ items, className = "" }: DockProps) {
  const dockVariants: Variants = {
    hidden: {
      opacity: 0,
      y: 24,
      scale: 0.96,
    },
    visible: {
      opacity: 1,
      y: 0,
      scale: 1,
      transition: {
        delay: 0.2,
        duration: 0.22,
        ease: "easeOut",
        when: "beforeChildren",
        staggerChildren: 0.04,
      },
    },
  };

  return (
    <motion.div
      variants={dockVariants}
      initial="hidden"
      animate="visible"
      className={cn(
        className,
        `bg-muted/59 absolute bottom-3 h-md:bottom-4 left-1/2 transform -translate-x-1/2 flex items-end w-fit gap-1 h-md:gap-3 rounded-2xl border p-1 h-md:px-3 h-md:py-2 shadow-lg`,
      )}
      role="toolbar"
      aria-label="Application dock"
    >
      {items.map((item, index) => (
        <DockItem key={index} onClick={item.onClick} className={item.className}>
          <DockIcon>{item.icon}</DockIcon>
        </DockItem>
      ))}
    </motion.div>
  );
}
