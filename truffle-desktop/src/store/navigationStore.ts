import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface NavigationState {
  isCollapsed: boolean;
  setCollapsed: (collapsed: boolean) => void;
  toggleCollapsed: () => void;
}

export const useNavigationStore = create<NavigationState>()(
  persist(
    (set) => ({
      isCollapsed: false,
      setCollapsed: (collapsed) => set({ isCollapsed: collapsed }),
      toggleCollapsed: () => set((state) => ({ isCollapsed: !state.isCollapsed })),
    }),
    {
      name: 'truffle-navigation-storage',
    }
  )
);

export default useNavigationStore;
