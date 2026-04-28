import { useLocalStorage } from "@vueuse/core";

export function useAppearance() {
  const dockIconSize = useLocalStorage("appearance-dock-icon-size", 48);
  const dockIconSizeLarge = useLocalStorage("appearance-dock-icon-size-large", 64); // Size on hover could be calculated, but base size is what we want to configure.

  return {
    dockIconSize,
  };
}
