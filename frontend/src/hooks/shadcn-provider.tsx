import React from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
// import { useTranslation } from "react-i18next";

type Props = {
  children: React.ReactNode;
};

const ShadcnProvider = ({ children }: Props) => {
  // const [_, i18n] = useTranslation();
  // const position = useMemo(
  //   () => (i18n.language === "ar" ? "bottom-left" : "bottom-right"),
  //   [i18n.language],
  // );
  return (
    <TooltipProvider>
      {children}
      {/* @ts-ignore */}
      <Toaster closeButton />
    </TooltipProvider>
  );
};

export { ShadcnProvider };
