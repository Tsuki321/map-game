import { useState, useRef, useEffect, useMemo } from 'react';
import { useGameStore } from '../stores/gameStore';
import { selectCountry, getCountryInfo } from '../utils/api';
import type { Country } from '../types';

const CountrySelector: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const gameState = useGameStore((s) => s.gameState);
  const playerCountry = useGameStore((s) => s.playerCountry);
  const setGameState = useGameStore((s) => s.setGameState);
  const setPlayerCountry = useGameStore((s) => s.setPlayerCountry);
  const addChatMessage = useGameStore((s) => s.addChatMessage);
  const setLoadingGlobal = useGameStore((s) => s.setLoading);

  // Derive sorted country list
  const allCountries: Country[] = useMemo(() => {
    if (!gameState) return [];
    return Object.values(gameState.countries).sort((a, b) =>
      a.name.localeCompare(b.name)
    );
  }, [gameState]);

  // Filter by search term
  const filtered = useMemo(() => {
    if (!searchTerm.trim()) return allCountries;
    const term = searchTerm.toLowerCase();
    return allCountries.filter(
      (c) =>
        c.name.toLowerCase().includes(term) ||
        c.id.toLowerCase().includes(term) ||
        c.continent.toLowerCase().includes(term)
    );
  }, [searchTerm, allCountries]);

  // Close dropdown on click outside
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  async function handleSelect(country: Country) {
    if (loading) return;
    setLoading(true);
    setLoadingGlobal(true);
    try {
      const updated = await selectCountry(country.id);
      setPlayerCountry(updated);
      // Also update in gameState
      if (gameState) {
        const newState = {
          ...gameState,
          player_country_id: country.id,
          countries: {
            ...gameState.countries,
            [country.id]: { ...gameState.countries[country.id], is_player_controlled: true },
          },
        };
        setGameState(newState);
      }
      addChatMessage('system', `Now controlling: ${country.name}`);
      setIsOpen(false);
      setSearchTerm(country.name);
    } catch (err: any) {
      addChatMessage('system', `Error selecting country: ${err}`);
    } finally {
      setLoading(false);
      setLoadingGlobal(false);
    }
  }

  return (
    <div className="country-selector" ref={containerRef}>
      {/* Current selection display */}
      <div
        className="country-selector-chip"
        onClick={() => !loading && setIsOpen(!isOpen)}
      >
        {playerCountry ? (
          <>
            <span
              className="country-color-dot"
              style={{ backgroundColor: playerCountry.color }}
            />
            <span>{playerCountry.name}</span>
          </>
        ) : (
          <span className="country-selector-placeholder">Select a country...</span>
        )}
        <span className={`country-selector-arrow ${isOpen ? 'open' : ''}`}>
          &#9662;
        </span>
      </div>

      {/* Dropdown */}
      {isOpen && (
        <div className="country-selector-dropdown">
          <input
            type="text"
            className="country-selector-search"
            placeholder="Search countries..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            autoFocus
          />
          <div className="country-selector-list">
            {filtered.length === 0 ? (
              <div className="country-selector-empty">No countries found</div>
            ) : (
              filtered.map((c) => (
                <div
                  key={c.id}
                  className={`country-selector-item ${
                    playerCountry?.id === c.id ? 'active' : ''
                  }`}
                  onClick={() => handleSelect(c)}
                >
                  <span
                    className="country-color-dot"
                    style={{ backgroundColor: c.color }}
                  />
                  <span className="country-selector-name">{c.name}</span>
                  <span className="country-selector-continent">
                    {c.continent}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {loading && <div className="country-selector-loading">Loading...</div>}
    </div>
  );
};

export default CountrySelector;
