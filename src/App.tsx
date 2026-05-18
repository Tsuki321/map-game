import React, { useEffect, useState } from 'react';
import { useGameStore } from './stores/gameStore';
import Map from './components/Map';
import CountrySelector from './components/CountrySelector';
import Chatbox from './components/Chatbox';
import CommandInput from './components/CommandInput';
import AdvisorPanel from './components/AdvisorPanel';
import StatusBar from './components/StatusBar';
import WarPanel from './components/WarPanel';
import SettingsPanel from './components/SettingsPanel';
import { getScenarioList, startNewGame } from './utils/api';

const App: React.FC = () => {
  const { gameState, setGameState } = useGameStore();
  const [showSettings, setShowSettings] = useState(false);
  const [showScenarioSelect, setShowScenarioSelect] = useState(true);
  const [scenarios, setScenarios] = useState<string[]>([]);
  const [selectedScenario, setSelectedScenario] = useState('ww2_1939');

  useEffect(() => {
    getScenarioList().then(setScenarios).catch(() => setScenarios(['ww2_1939']));
  }, []);

  const handleStartGame = async () => {
    try {
      const state = await startNewGame(selectedScenario);
      setGameState(state);
      setShowScenarioSelect(false);
    } catch (e) {
      console.error('Failed to start game:', e);
    }
  };

  if (showScenarioSelect) {
    return (
      <div className="scenario-select">
        <div className="scenario-card">
          <h1>Map Game - Alternate History</h1>
          <p>Select a scenario to begin:</p>
          <select
            value={selectedScenario}
            onChange={(e) => setSelectedScenario(e.target.value)}
          >
            {scenarios.map((s) => (
              <option key={s} value={s}>
                {s.replace(/_/g, ' ')}
              </option>
            ))}
          </select>
          <button onClick={handleStartGame} className="btn-primary">
            Start Game
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <div className="top-bar">
        <CountrySelector />
        <div className="scenario-info">
          {gameState && (
            <span>
              {gameState.scenario_name.replace(/_/g, ' ')} | Turn{' '}
              {gameState.turn_number} | {gameState.current_month}/
              {gameState.current_year}
            </span>
          )}
        </div>
        <button onClick={() => setShowSettings(true)} className="btn-settings">
          Settings
        </button>
      </div>
      <div className="main-content">
        <div className="map-area">
          <Map />
        </div>
        <div className="side-panel">
          <WarPanel />
          <Chatbox />
          <AdvisorPanel />
          <CommandInput />
        </div>
      </div>
      <StatusBar />
      {showSettings && (
        <SettingsPanel onClose={() => setShowSettings(false)} />
      )}
    </div>
  );
};

export default App;
