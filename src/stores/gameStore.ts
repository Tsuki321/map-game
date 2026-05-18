import { create } from 'zustand';
import type { GameState, Country, AppSettings, TurnSummary } from '../types';

interface GameStore {
  gameState: GameState | null;
  playerCountry: Country | null;
  chatHistory: Array<{ role: string; content: string; timestamp: number }>;
  isLoading: boolean;
  advisorOutput: string | null;
  lastTurnResult: TurnSummary | null;
  settings: AppSettings;
  showAdvisor: boolean;
  selectedCountryId: string | null;

  setGameState: (state: GameState) => void;
  setPlayerCountry: (country: Country | null) => void;
  addChatMessage: (role: string, content: string) => void;
  setLoading: (loading: boolean) => void;
  setAdvisorOutput: (output: string | null) => void;
  setLastTurnResult: (result: TurnSummary | null) => void;
  setSettings: (settings: Partial<AppSettings>) => void;
  setShowAdvisor: (show: boolean) => void;
  setSelectedCountryId: (id: string | null) => void;
  clearChat: () => void;
}

export const useGameStore = create<GameStore>((set) => ({
  gameState: null,
  playerCountry: null,
  chatHistory: [],
  isLoading: false,
  advisorOutput: null,
  lastTurnResult: null,
  settings: {
    llmConfig: null,
    advisorEnabled: false,
  },
  showAdvisor: false,
  selectedCountryId: null,

  setGameState: (state) => set({ gameState: state }),

  setPlayerCountry: (country) => set({ playerCountry: country }),

  addChatMessage: (role, content) =>
    set((prev) => ({
      chatHistory: [
        ...prev.chatHistory,
        { role, content, timestamp: Date.now() },
      ],
    })),

  setLoading: (loading) => set({ isLoading: loading }),

  setAdvisorOutput: (output) => set({ advisorOutput: output }),

  setLastTurnResult: (result) => set({ lastTurnResult: result }),

  setSettings: (partial) =>
    set((prev) => ({
      settings: { ...prev.settings, ...partial },
    })),

  setShowAdvisor: (show) => set({ showAdvisor: show }),

  setSelectedCountryId: (id) => set({ selectedCountryId: id }),

  clearChat: () => set({ chatHistory: [] }),
}));
