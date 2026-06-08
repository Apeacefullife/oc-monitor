import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface WindowState {
  interactionLocked: boolean;
  setInteractionLocked: (locked: boolean) => void;
  syncInteractionLocked: () => Promise<void>;
}

export const useWindowStore = create<WindowState>((set) => ({
  interactionLocked: false,

  setInteractionLocked: (locked) => set({ interactionLocked: locked }),

  syncInteractionLocked: async () => {
    try {
      const locked = await invoke<boolean>("is_window_interaction_locked");
      set({ interactionLocked: locked });
    } catch {
      // ignore
    }
  },
}));
