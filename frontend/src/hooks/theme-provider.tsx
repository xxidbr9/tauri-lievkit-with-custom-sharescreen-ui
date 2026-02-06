import React, { useMemo } from "react";
import { DeviceThemeProvider, useDeviceTheme } from "./device-theme-provider";
import type { UseDeviceThemeProps } from "./device-theme-provider.d";

const ThemeContext = React.createContext<{
  dark: boolean;
  setTheme: UseDeviceThemeProps["setTheme"];
  theme?: string;
}>({
  dark: true,
  setTheme: () => {},
  theme: "",
});

type ThemeProviderProps = {
  children: React.ReactNode;
  dark?: boolean;
};

const InnerProvider = (props: ThemeProviderProps) => {
  const { setTheme, resolvedTheme: deviceTheme, theme } = useDeviceTheme();

  const isDark = useMemo(() => deviceTheme === "dark", [deviceTheme]);
  return (
    <ThemeContext.Provider value={{ dark: isDark, setTheme, theme }}>
      {props.children}
    </ThemeContext.Provider>
  );
};

const ThemeProvider = (props: ThemeProviderProps) => {
  return (
    <DeviceThemeProvider defaultTheme="system" attribute="class">
      <InnerProvider {...props} />
    </DeviceThemeProvider>
  );
};

export { ThemeContext, ThemeProvider };

export default ThemeProvider;
