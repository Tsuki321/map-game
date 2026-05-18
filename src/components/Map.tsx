import { useEffect, useRef } from 'react';
import maplibregl from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import { useGameStore } from '../stores/gameStore';
import {
  INITIAL_VIEW,
  MAP_STYLE,
  getFillColor,
  getBorderStyle,
  SELECTED_BORDER_STYLE,
  getFeatureCountryId,
} from '../utils/mapUtils';
import type { Country } from '../types';

const GEOJSON_URL = '/geodata/ne_110m_admin_0_countries.geojson';

const Map: React.FC = () => {
  const mapContainerRef = useRef<HTMLDivElement>(null);
  const mapRef = useRef<maplibregl.Map | null>(null);
  const geoJsonDataRef = useRef<GeoJSON.FeatureCollection | null>(null);

  const gameState = useGameStore((s) => s.gameState);
  const selectedCountryId = useGameStore((s) => s.selectedCountryId);
  const setSelectedCountryId = useGameStore((s) => s.setSelectedCountryId);

  // Create the map instance on mount
  useEffect(() => {
    if (!mapContainerRef.current || mapRef.current) return;

    const map = new maplibregl.Map({
      container: mapContainerRef.current,
      style: MAP_STYLE,
      center: INITIAL_VIEW.center,
      zoom: INITIAL_VIEW.zoom,
      attributionControl: false,
    });

    map.addControl(new maplibregl.NavigationControl(), 'top-left');

    map.on('load', async () => {
      try {
        const resp = await fetch(GEOJSON_URL);
        const data: GeoJSON.FeatureCollection = await resp.json();
        geoJsonDataRef.current = data;

        // Add source
        map.addSource('countries', {
          type: 'geojson',
          data,
        });

        // Add fill layer
        map.addLayer({
          id: 'countries-fill',
          type: 'fill',
          source: 'countries',
          paint: {
            'fill-color': '#374151',
            'fill-opacity': 0.5,
            'fill-outline-color': '#1e293b',
          },
        });

        // Add border layer
        map.addLayer({
          id: 'countries-border',
          type: 'line',
          source: 'countries',
          paint: {
            'line-color': '#1e293b',
            'line-width': 0.75,
            'line-opacity': 0.7,
          },
        });

        // Add highlight layer (hidden by default)
        map.addLayer({
          id: 'countries-highlight',
          type: 'line',
          source: 'countries',
          paint: {
            'line-color': SELECTED_BORDER_STYLE.color,
            'line-width': SELECTED_BORDER_STYLE.width,
            'line-opacity': 0, // hidden initially
            'line-dasharray': SELECTED_BORDER_STYLE.dasharray,
          },
        });

        // Apply initial colors from game state
        applyCountryColors(map);
      } catch (err) {
        console.error('Failed to load GeoJSON:', err);
      }
    });

    // Click handler: identify clicked country
    map.on('click', 'countries-fill', (e) => {
      if (!e.features || e.features.length === 0) return;
      const feature = e.features[0];
      const countryId = getFeatureCountryId(feature);
      if (countryId) {
        setSelectedCountryId(countryId);
        highlightCountry(map, countryId);
      }
    });

    // Hover cursor
    map.on('mouseenter', 'countries-fill', () => {
      map.getCanvas().style.cursor = 'pointer';
    });
    map.on('mouseleave', 'countries-fill', () => {
      map.getCanvas().style.cursor = '';
    });

    mapRef.current = map;

    return () => {
      map.remove();
      mapRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // React to selectedCountryId changes
  useEffect(() => {
    const map = mapRef.current;
    if (!map || !map.isStyleLoaded()) return;

    if (selectedCountryId) {
      highlightCountry(map, selectedCountryId);
    } else {
      // Remove highlight
      map.setPaintProperty(
        'countries-highlight',
        'line-opacity',
        0
      );
    }
  }, [selectedCountryId]);

  // React to gameState changes: update fill colors
  useEffect(() => {
    const map = mapRef.current;
    if (!map || !map.isStyleLoaded() || !geoJsonDataRef.current) return;
    applyCountryColors(map);
  }, [gameState]);

  return (
    <div style={{ flex: 1, position: 'relative', height: '100%' }}>
      <div ref={mapContainerRef} style={{ width: '100%', height: '100%' }} />
      {/* Scenario / turn overlay */}
      {gameState && (
        <div className="map-overlay">
          <span className="map-overlay-scenario">{gameState.scenario_name}</span>
          <span className="map-overlay-turn">
            Turn {gameState.turn_number} — {gameState.current_year}/{String(gameState.current_month).padStart(2, '0')}
          </span>
        </div>
      )}
    </div>
  );
};

/** Applies fill colors to all countries based on current game state */
function applyCountryColors(map: maplibregl.Map) {
  const data = geoJsonDataRef.current;
  if (!data) return;

  const state = useGameStore.getState().gameState;
  if (!state) {
    // No game state: neutral grey
    map.setPaintProperty('countries-fill', 'fill-color', '#374151');
    map.setPaintProperty('countries-fill', 'fill-opacity', 0.5);
    return;
  }

  // Use data-driven styling with a fallback expression
  const matchExpr: (string | number | string[])[] = ['match', ['get', 'ISO_A3']];

  // Build lookup from properties
  data.features.forEach((f) => {
    const props = f.properties;
    if (!props) return;
    const id = props.ISO_A3 || props.ISO_A2 || props.ADMIN || props.NAME;
    if (!id) return;

    const country = state.countries[id];
    if (country) {
      matchExpr.push(id, getFillColor(country));
    }
  });

  matchExpr.push('#374151'); // default

  map.setPaintProperty('countries-fill', 'fill-color', matchExpr as any);
  map.setPaintProperty('countries-fill', 'fill-opacity', 0.6);

  // Also style borders
  const borderMatch: (string | number | any[])[] = ['match', ['get', 'ISO_A3']];
  data.features.forEach((f) => {
    const props = f.properties;
    if (!props) return;
    const id = props.ISO_A3 || props.ISO_A2 || props.ADMIN || props.NAME;
    if (!id) return;

    const country = state.countries[id];
    if (country) {
      const bs = getBorderStyle(country);
      borderMatch.push(id, bs.color);
    }
  });
  borderMatch.push('#1e293b');

  map.setPaintProperty('countries-border', 'line-color', borderMatch as any);
}

/** Highlights a single country by filtering the highlight layer */
function highlightCountry(map: maplibregl.Map, countryId: string) {
  const data = geoJsonDataRef.current;
  if (!data) return;

  // Filter highlight layer to only show the selected country
  map.setFilter('countries-highlight', [
    'any',
    ['==', ['get', 'ISO_A3'], countryId],
    ['==', ['get', 'ISO_A2'], countryId],
    ['==', ['get', 'ADMIN'], countryId],
    ['==', ['get', 'NAME'], countryId],
  ]);

  map.setPaintProperty(
    'countries-highlight',
    'line-opacity',
    SELECTED_BORDER_STYLE.opacity
  );
}

export default Map;
