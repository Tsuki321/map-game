import { invoke } from '@tauri-apps/api/core';
import type { GameState, Country } from '../types';

async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    console.error(`Tauri invoke error [${command}]:`, error);
    throw error;
  }
}

export async function getGameState(): Promise<GameState> {
  return safeInvoke<GameState>('get_game_state');
}

export async function selectCountry(countryId: string): Promise<Country> {
  return safeInvoke<Country>('select_country', { countryId });
}

export async function getCountryInfo(countryId: string): Promise<Country> {
  return safeInvoke<Country>('get_country_info', { countryId });
}

export async function getWorldSituation(): Promise<string> {
  return safeInvoke<string>('get_world_situation');
}

export async function submitDirective(
  directive: string,
  useAdvisor: boolean
): Promise<string> {
  return safeInvoke<string>('submit_directive', { directive, useAdvisor });
}

export async function getAdvisorSuggestion(directive: string): Promise<string> {
  return safeInvoke<string>('get_advisor_suggestion', { directive });
}

export async function executeTurn(): Promise<string> {
  return safeInvoke<string>('execute_turn');
}

export async function saveGame(slot: string): Promise<void> {
  return safeInvoke<void>('save_game', { slot });
}

export async function loadGame(slot: string): Promise<GameState> {
  return safeInvoke<GameState>('load_game', { slot });
}

export async function listSaves(): Promise<string[]> {
  return safeInvoke<string[]>('list_saves');
}

export async function configureLlm(
  provider: string,
  apiKey: string,
  model: string,
  endpoint: string
): Promise<void> {
  return safeInvoke<void>('configure_llm', { provider, apiKey, model, endpoint });
}

export async function startNewGame(scenario: string): Promise<GameState> {
  return safeInvoke<GameState>('start_new_game', { scenario });
}

export async function getScenarioList(): Promise<string[]> {
  return safeInvoke<string[]>('get_scenario_list');
}
