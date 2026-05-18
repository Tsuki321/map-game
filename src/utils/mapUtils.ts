import type { Country } from '../types';

export const INITIAL_VIEW: { center: [number, number]; zoom: number } = {
  center: [20, 30],
  zoom: 2.5,
};

export const MAP_STYLE: string =
  'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json';

export const countryColors: Record<string, string> = {
  USA: '#2563eb',
  CHN: '#dc2626',
  RUS: '#7c3aed',
  GBR: '#0891b2',
  FRA: '#2563eb',
  DEU: '#ca8a04',
  JPN: '#ea580c',
  IND: '#16a34a',
  BRA: '#16a34a',
  ZAF: '#9333ea',
  AUS: '#db2777',
  CAN: '#f43f5e',
  ITA: '#14b8a6',
  ESP: '#f59e0b',
  KOR: '#6366f1',
  TUR: '#e11d48',
  SAU: '#15803d',
  MEX: '#15803d',
  IDN: '#f97316',
  NGA: '#84cc16',
  EGY: '#d97706',
  PAK: '#22d55e',
  IRN: '#10b981',
  ARG: '#06b6d4',
  default: '#374151',
};

/**
 * Returns a fill color for a country polygon.
 * Player-controlled countries get their assigned color;
 * unowned neutral countries appear greyish;
 * otherwise the country's defined color or a default muted tone.
 */
export function getFillColor(country: Country): string {
  if (country.is_player_controlled) {
    return country.color;
  }
  return countryColors[country.id] || country.color || countryColors.default;
}

/**
 * Returns a border / line style for a country polygon.
 * The player's own country receives a bright highlighted border,
 * other countries get a subtle dark border.
 */
export function getBorderStyle(country: Country): {
  color: string;
  width: number;
  opacity: number;
} {
  if (country.is_player_controlled) {
    return { color: '#facc15', width: 2.5, opacity: 1 };
  }
  return { color: '#1e293b', width: 0.75, opacity: 0.7 };
}

/**
 * Returns a border style for the currently selected (highlighted) country.
 */
export const SELECTED_BORDER_STYLE = {
  color: '#facc15',
  width: 3,
  opacity: 1,
  dasharray: [4, 2],
};

/**
 * Parses a country name/iso from the GeoJSON feature properties.
 * Tries several common property keys.
 */
export function getFeatureCountryId(feature: GeoJSON.Feature): string | null {
  const p = feature.properties;
  if (!p) return null;
  return (
    p.ISO_A3_EH ||
    p.ISO_A3 ||
    p.ISO_A2 ||
    p.ADMIN ||
    p.NAME ||
    p.name ||
    p.iso_3166_1 ||
    null
  );
}
